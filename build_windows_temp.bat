@echo off

cd /d "%~dp0"
REM rd /s /q target\release\bundle\windows
cd server
cargo build --release
cd ..\client
cargo build --release
cd ..
md target\release\bundle\windows
move /Y target\release\client.exe target\release\bundle\windows\stub.dat
move /Y target\release\server.exe target\release\bundle\windows\Yosuke.exe
REM explorer target\release\bundle\windows
exit /b 0
