@if not defined _echo echo off

rem Find in the local directory the other environment files (e.g. env.[user].bat)
rem So that each developer can have their own environment settings
for %%f in (env.*.bat) do (
  if exist "%%f" call "%%f"
)

set VSWHERE="C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"

for /f "usebackq delims=" %%i in (`%VSWHERE% -prerelease -latest -property installationPath`) do (
  if exist "%%i\Common7\Tools\vsdevcmd.bat" (
    %comspec% /k "%%i\Common7\Tools\vsdevcmd.bat" -arch=x64 -host_arch=x64
    exit /b
  )
)

rem Instance or command prompt not found
exit /b 2
