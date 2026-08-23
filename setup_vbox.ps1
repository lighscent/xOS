# setup_vbox.ps1 - create VirtualBox VM for xOS (requires VirtualBox)
param([string]$VmName="xOS", [string]$ImgPath="$PSScriptRoot\build\xos.img")

$VBox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
if (!(Test-Path $VBox)) { $VBox = "VBoxManage.exe" }

if (!(Get-Command VBoxManage -ErrorAction SilentlyContinue) -and !(Test-Path $VBox)) {
  Write-Host "VirtualBox not found. Install: winget install Oracle.VirtualBox"
  exit 1
}
if (!(Test-Path $ImgPath)) { Write-Host "Image not found: $ImgPath"; exit 1 }
$ImgPath = (Resolve-Path $ImgPath).Path

$exists = & $VBox list vms 2>$null | Select-String -Pattern "`"$VmName`""
if ($exists) {
  Write-Host "VM $VmName already exists - reconfiguring (close VirtualBox GUI if locked)..."
  & $VBox controlvm $VmName poweroff 2>$null
  Start-Sleep -Seconds 1
  & $VBox modifyvm $VmName --memory 64 --boot1 floppy --boot2 disk --boot3 none --firmware bios --chipset piix3 --cpus 1 2>&1 | Out-Null
  # re-attach floppy (create controller if missing)
  & $VBox storagectl $VmName --name "Floppy" --add floppy --controller I82078 2>$null
  & $VBox storageattach $VmName --storagectl "Floppy" --port 0 --device 0 --type fdd --medium $ImgPath
  if ($LASTEXITCODE -ne 0) {
    Write-Host "Locked - close VirtualBox GUI and retry, or run:"
    Write-Host "  & `"$VBox`" controlvm $VmName poweroff; & `"$VBox`" unregistervm $VmName --delete; .\setup_vbox.ps1"
  } else {
    Write-Host "VM $VmName reconfigured."
  }
} else {
  Write-Host "Creating VM $VmName..."
  & $VBox createvm --name $VmName --ostype "Other" --register
  & $VBox modifyvm $VmName --memory 64 --boot1 floppy --boot2 disk --boot3 none --firmware bios --chipset piix3 --cpus 1
  & $VBox storagectl $VmName --name "Floppy" --add floppy --controller I82078
  & $VBox storageattach $VmName --storagectl "Floppy" --port 0 --device 0 --type fdd --medium $ImgPath
  Write-Host "VM created."
}
# also attach as ISO alternative if you have os.iso: --type dvddrive --medium build\os.iso

Write-Host "VM created. Start with: VBoxManage startvm $VmName"
Write-Host "Or open VirtualBox GUI and start $VmName"
