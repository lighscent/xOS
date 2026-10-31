# xOS — bugs majeurs

Audit du 2026-10-31, vérifié par lecture du code, par `scripts/build.py` et par des essais VirtualBox 7.2.

## 1. Critique — l'effacement d'urgence ne peut pas fonctionner

**Statut : corrigé (2026-10-31).**

`kernel/src/usb.rs:71-109`

- `mov ax, 0xB800; mov es, ax` est du code real-mode. Le noyau tourne en mode protégé avec une GDT de 3 à 5 entrées : charger le sélecteur `0xB800` provoque un `#GP`, et sans IDT → triple fault. Le `vga::clear_screen()` juste avant suffisait.
- `write_bytes(0x600, 0x9F000 - 0x600)` couvre le noyau à `0x7E00` et la pile à `0x7C00` : la routine s'efface elle-même pendant son exécution, donc le `hlt` final n'est pas garanti.
- Le bug est latent tant que la détection ne se déclenche pas (voir 2), mais il se réveille dès qu'on répare la détection.

Correctif : l'asm VGA real-mode a été supprimé. Un stub de 19 octets est copié à `0x500` (sous la zone effacée), efface `0x600..0x9F000` avec `rep stosb`, puis boucle sur `hlt`. Testé via un déclenchement temporaire dans VBox 7.2 : l'écran reste sur le message après 12 s, sans reboot ; le build final démarre normalement. Le wipe reste inatteignable tant que le point 2 n'est pas corrigé.

## 2. Critique — le retrait USB n'est pas détecté

**Statut : ouvert.**

`kernel/src/bios.rs:15`, `kernel/src/usb.rs:53-69`

- `bios::drive_present()` renvoie `true` en dur.
- En mode BIOS (cas d'une vraie clé USB), `check()` voit toujours « présent ».
- En mode ATA, `ata_present()` surveille le disque ATA primaire, pas la clé USB. Débrancher la clé ne change pas l'état du disque interne.
- `panic_wipe()` n'est donc pas déclenché par le retrait de la clé ; la garantie d'effacement de la RAM n'existe pas en pratique.

## 3. Critique — `config::save()` peut écrire sur le disque hôte

**Statut : ouvert.**

`kernel/src/config.rs:69-83`

- `load()` et `save()` utilisent LBA33 sur le canal ATA primaire `0x1F0` sans vérifier `boot_drive`.
- `save()` est appelé par `layout azerty|qwerty`.
- Sur une machine avec contrôleur IDE/legacy, ou en mode IDE, cela peut écrire sur le disque interne. LBA33 est le dernier secteur des entrées GPT : corruption possible de la table de partitions, ou d'un bootloader dans le gap MBR.
- Si `ata_read_lba()` échoue, `save()` écrit quand même un secteur zéro avec le magic `xOSC` : le contenu précédent est perdu.
- Sur AHCI pur, `0x1F7` vaut `0xFF` et le code n'écrit pas ; le risque est conditionnel mais contraire à la contrainte « jamais le disque hôte ».

## 4. Élevé — dépassement des 16 Ko silencieux

**Statut : corrigé (2026-10-31).**

`scripts/build.py:192-193`, `scripts/test.py:22`

Correctif : `build.py` (`make_img`, `make_hdd_image`, pad final) sort en erreur (`sys.exit(1)`) au lieu de tronquer ; `test.py` exige chaque chaîne (`xOS`, `help`, `panic`) et vérifie le type de partition `0x0C`. Vérifié : build + tests OK, overflow simulé (16385 o) bloqué avec exit 1.

- Le noyau utilise environ 16,06 Ko sur 16 384 octets, soit ~300 octets de marge.
- En cas de dépassement, `build.py` tronque et affiche seulement `[!] kernel ... trunc` ; il ne échoue pas.
- `test.py` vérifie `len == 16384`, donc un noyau tronqué passe les tests. Il vérifie aussi les chaînes en s'arrêtant au premier motif trouvé (`xOS`), donc `help`/`panic` ne sont même pas garantis.
- Résultat possible : build « OK », tests « OK », image qui ne démarre pas ou plante après quelques instructions.

## 5. Moyens

**Statut : ouvert.**

- `ata_present()` considère tout statut différent de `0xFF` comme un disque présent. Un bus flottant à `0x00` fait choisir le mode ATA à tort.
- `FAIL_CNT >= 1` : un seul échec transitoire déclenche l'effacement. Si la détection est réparée, une lecture instable peut détruire la RAM.
- `shutdown` ne fait qu'un `out 0x604` (port QEMU) puis `hlt` ; sur matériel réel, ça ne coupe pas l'alimentation.
- `halt` affiche « Press ESC » puis imprime `^C` ; ce n'est pas un vrai halt.
- Pas de `FLUSH CACHE` (`0xE7`) après l'écriture ATA de la config : persistance non garantie en cas de coupure.
- `Ctrl-C` ne fonctionne pas vraiment : le scancode `0x1D` (Ctrl) est ignoré et aucun chemin ne renvoie l'octet 3.

## 6. Outillage (hors noyau)

**Statut : ouvert.**

Sous VirtualBox 7.2 :

- les ISO `xos-tiny.iso` et `xos-usb.iso` ne démarrent pas comme CD ;
- les images brutes `.img` sont rejetées comme HDD (« detected floppy »), donc `setup_vbox.ps1 -Mode usb` échoue ;
- `-Mode iso` est cassé aussi.

Le VDI et l'image disquette démarrent correctement.

## Priorités

1. Corriger l'effacement d'urgence (1) — corrigé.
2. Corriger la détection de retrait (2).
3. Empêcher toute écriture hors du disque de boot (3).
4. Rendre le dépassement 16 Ko bloquant et renforcer `test.py` (4).

Attention : corriger uniquement la détection USB sans corriger le wipe transforme un défaut de sécurité en crash/reboot.
