# shim -> scripts/setup_vbox.ps1 (keeps .\setup_vbox.ps1 working after move to scripts/)
param([string]$VmName="xOS", [string]$ImgPath="", [ValidateSet("auto","vm","usb","hdd","floppy","iso")][string]$Mode="auto")
$target = Join-Path $PSScriptRoot "scripts/setup_vbox.ps1"
if (Test-Path $target) {
    if ([string]::IsNullOrEmpty($ImgPath)) { & $target -VmName $VmName -Mode $Mode }
    else { & $target -VmName $VmName -ImgPath $ImgPath -Mode $Mode }
    exit $LASTEXITCODE
}
Write-Host "scripts/setup_vbox.ps1 not found"; exit 1
