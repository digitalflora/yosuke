@echo off

cd /d "%~dp0"
rd /s /q target\release\bundle\windows
cd server
cargo build --release
cd ..\client
cargo build --release
cd ..
md target\release\bundle\windows
move target\release\client.exe target\release\bundle\windows\stub.dat
move target\release\server.exe target\release\bundle\windows\Yosuke.exe
explorer target\release\bundle\windows
exit /b 0
