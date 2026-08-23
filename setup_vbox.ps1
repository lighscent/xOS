# setup_vbox.ps1 - create VirtualBox VM for xOS (requires VirtualBox)
param([string]$VmName="xOS", [string]$ImgPath="", [ValidateSet("auto","vm","usb","hdd","floppy","iso")][string]$Mode="auto")

$VBox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
if (!(Test-Path $VBox)) { $VBox = "VBoxManage.exe" }

if (!(Get-Command VBoxManage -ErrorAction SilentlyContinue) -and !(Test-Path $VBox)) {
  Write-Host "VirtualBox not found. Install: winget install Oracle.VirtualBox"
  exit 1
}

# Auto-select image if not provided - split USB vs VM
if ($ImgPath -eq "") {
    if ($Mode -eq "usb") {
        $cands = @("$PSScriptRoot\build\xos-usb.img","$PSScriptRoot\build\xos-usb.iso","$PSScriptRoot\build\xos-hybrid.iso","$PSScriptRoot\build\xos.img")
    } elseif ($Mode -eq "vm" -or $Mode -eq "hdd") {
        $cands = @("$PSScriptRoot\build\xos-vm.vdi","$PSScriptRoot\build\xos-vm.vmdk","$PSScriptRoot\build\xos-vm.img","$PSScriptRoot\build\xos-hdd.img","$PSScriptRoot\build\xos.img","$PSScriptRoot\build\xos-usb.img")
    } elseif ($Mode -eq "iso") {
        $cands = @("$PSScriptRoot\build\xos-tiny.iso","$PSScriptRoot\build\xos-usb.iso","$PSScriptRoot\build\os.iso")
    } elseif ($Mode -eq "floppy") {
        $cands = @("$PSScriptRoot\build\xos-floppy.img","$PSScriptRoot\build\os.img")
    } else {
        # auto: prefer VM variant
        $cands = @("$PSScriptRoot\build\xos-vm.vdi","$PSScriptRoot\build\xos-vm.img","$PSScriptRoot\build\xos-vm.vmdk","$PSScriptRoot\build\xos-usb.img","$PSScriptRoot\build\xos-hdd.img","$PSScriptRoot\build\xos-hybrid.iso","$PSScriptRoot\build\xos.img","$PSScriptRoot\build\os.img")
    }
    foreach ($c in $cands) { if (Test-Path $c) { $ImgPath = $c; break } }
}
if (!(Test-Path $ImgPath)) { Write-Host "Image not found: $ImgPath (run build.bat)"; exit 1 }
$ImgPath = (Resolve-Path $ImgPath).Path
$ext = [IO.Path]::GetExtension($ImgPath).ToLower()
if ($Mode -eq "auto") {
    if ($ImgPath -like "*vm*") { $Mode = "vm" }
    elseif ($ImgPath -like "*usb*") { $Mode = "usb" }
    elseif ($ImgPath -like "*hdd*") { $Mode = "vm" }
    elseif ($ext -eq ".iso") { $Mode = "iso" }
    elseif ($ext -eq ".vdi" -or $ext -eq ".vmdk" -or $ext -eq ".qcow2") { $Mode = "vm" }
    else { $Mode = "floppy" }
}
# normalize hdd alias to vm
if ($Mode -eq "hdd") { $Mode = "vm" }
Write-Host "Using image: $ImgPath (mode=$Mode)"

$exists = & $VBox list vms 2>$null | Select-String -Pattern "`"$VmName`""
if ($exists) {
  Write-Host "VM $VmName already exists - reconfiguring (close VirtualBox GUI if locked)..."
  & $VBox controlvm $VmName poweroff 2>$null
  Start-Sleep -Seconds 1
  if ($Mode -eq "vm" -or $Mode -eq "usb") {
    & $VBox modifyvm $VmName --memory 64 --boot1 disk --boot2 none --boot3 none --firmware bios --chipset piix3 --cpus 1 2>&1 | Out-Null
  } elseif ($Mode -eq "iso") {
    & $VBox modifyvm $VmName --memory 64 --boot1 dvd --boot2 disk --boot3 none --firmware bios --chipset piix3 --cpus 1 2>&1 | Out-Null
  } else {
    & $VBox modifyvm $VmName --memory 64 --boot1 floppy --boot2 disk --boot3 none --firmware bios --chipset piix3 --cpus 1 2>&1 | Out-Null
  }
} else {
  Write-Host "Creating VM $VmName..."
  & $VBox createvm --name $VmName --ostype "Other" --register
  if ($Mode -eq "vm" -or $Mode -eq "usb") {
    & $VBox modifyvm $VmName --memory 64 --boot1 disk --boot2 none --boot3 none --firmware bios --chipset piix3 --cpus 1
  } elseif ($Mode -eq "iso") {
    & $VBox modifyvm $VmName --memory 64 --boot1 dvd --boot2 disk --boot3 none --firmware bios --chipset piix3 --cpus 1
  } else {
    & $VBox modifyvm $VmName --memory 64 --boot1 floppy --boot2 disk --boot3 none --firmware bios --chipset piix3 --cpus 1
  }
}

# detach any previous medium to free the file (must be before closemedium)
& $VBox storageattach $VmName --storagectl "SATA" --port 0 --device 0 --medium none 2>$null
& $VBox storageattach $VmName --storagectl "IDE" --port 0 --device 0 --medium none 2>$null
& $VBox storageattach $VmName --storagectl "Floppy" --port 0 --device 0 --medium none 2>$null

# fix stale medium type cache (VBox remembers hdd vs floppy by path)
& $VBox closemedium disk "$ImgPath" 2>$null
& $VBox closemedium floppy "$ImgPath" 2>$null
& $VBox closemedium dvd "$ImgPath" 2>$null

$attachOk = 1
if ($Mode -eq "vm" -or $Mode -eq "usb") {
    & $VBox storagectl $VmName --name "SATA" --add sata --controller IntelAhci 2>$null
    & $VBox storageattach $VmName --storagectl "SATA" --port 0 --device 0 --medium none 2>$null
    & $VBox storageattach $VmName --storagectl "SATA" --port 0 --device 0 --type hdd --medium $ImgPath
    $attachOk = $LASTEXITCODE
    & $VBox storagectl $VmName --name "Floppy" --remove 2>$null
    & $VBox storageattach $VmName --storagectl "IDE" --port 0 --device 0 --medium none 2>$null
    & $VBox storagectl $VmName --name "IDE" --remove 2>$null
    if ($Mode -eq "usb") { Write-Host "Note: testing USB image ($ImgPath) in VM as SATA HDD (same bytes, dd to real USB for hardware)" }
} elseif ($Mode -eq "iso") {
    & $VBox storagectl $VmName --name "IDE" --add ide 2>$null
    & $VBox storageattach $VmName --storagectl "IDE" --port 0 --device 0 --medium none 2>$null
    & $VBox storageattach $VmName --storagectl "IDE" --port 0 --device 0 --type dvddrive --medium $ImgPath
    $attachOk = $LASTEXITCODE
} else {
    & $VBox storagectl $VmName --name "Floppy" --add floppy --controller I82078 2>$null
    & $VBox storageattach $VmName --storagectl "Floppy" --port 0 --device 0 --medium none 2>$null
    & $VBox storageattach $VmName --storagectl "Floppy" --port 0 --device 0 --type fdd --medium $ImgPath
    $attachOk = $LASTEXITCODE
}
if ($attachOk -ne 0) {
  Write-Host "Attach failed - close VirtualBox GUI and retry, or run:"
  Write-Host "  & `"$VBox`" controlvm $VmName poweroff; & `"$VBox`" unregistervm $VmName --delete; .\setup_vbox.ps1 -Mode $Mode"
} else {
  Write-Host "VM $VmName reconfigured for $Mode."
}
Write-Host "Start with: VBoxManage startvm $VmName"
Write-Host "Or open VirtualBox GUI and start $VmName"
Write-Host ""
Write-Host "Modes: -Mode vm (xos-vm.vdi/img, for VirtualBox), -Mode usb (xos-usb.img in VM), -Mode floppy, -Mode iso"
