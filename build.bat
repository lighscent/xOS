@echo off
set NASM=C:\Users\x\AppData\Local\bin\NASM\nasm.exe
if not exist "%NASM%" set NASM=nasm
echo Using NASM: %NASM%
"%NASM%" -v || (echo NASM not found & exit /b 1)

if not exist build mkdir build

if exist src\boot\boot.asm (
  echo Assembling src\boot\boot.asm...
  "%NASM%" -f bin src\boot\boot.asm -o build\boot.bin || exit /b 1
) else (
  echo Assembling boot.asm...
  "%NASM%" -f bin boot.asm -o build\boot.bin || exit /b 1
)
if exist src\kernel\main.asm (
  echo Assembling src\kernel\main.asm...
  "%NASM%" -I src\kernel\ -f bin src\kernel\main.asm -o build\kernel.bin || exit /b 1
) else (
  echo Assembling kernel.asm...
  "%NASM%" -f bin kernel.asm -o build\kernel.bin || exit /b 1
)

echo Building os.img (1.44MB floppy)...
copy /b build\boot.bin+build\kernel.bin build\os.tmp >nul
:: pad to 1474560 bytes using python
python -c "import pathlib; p=pathlib.Path('build/os.tmp'); d=p.read_bytes(); need=1474560-len(d); p2=pathlib.Path('build/os.img'); p2.write_bytes(d + b'\x00'*need if need>0 else d); print(f'os.img: {p2.stat().st_size} bytes')"
del build\os.tmp

echo Done. Outputs in build\:
dir build
echo.
echo To test in VirtualBox: create VM Type=Other, Version=Other/Unknown, attach build\os.img as floppy or hard disk (or build\os.iso as CD)
