@echo off
REM Update the timestamp of the file
copy /b "build.rs" +,,

REM Wait for 1 second
timeout /t 1 /nobreak >nul
