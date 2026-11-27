; Redis Manager Windows installer (Inno Setup 7; also works with 6)
; Build: cargo release   (or .\scripts\build-installer.ps1)
; Compile with /DMyArch=64, /DMyArch=86, or /DMyArch=arm64.
; Expects portable binaries:
;   dist\redis-manager-{version}_portable_windows_64.exe
;   dist\redis-manager-{version}_portable_windows_86.exe
;   dist\redis-manager-{version}_portable_windows_arm64.exe
; Installed process name stays redis-manager.exe (shortcuts / taskkill / autostart).

#define MyAppName "Redis Manager"
#ifndef MyAppVersion
  #define MyAppVersion "1.0.0"
#endif
#ifndef MyArch
  #define MyArch "64"
#endif
#define MyAppPublisher "Redis Manager"
#define MyAppExeName "redis-manager.exe"
#define MyPortableName "redis-manager-" + MyAppVersion + "_portable_windows_" + MyArch + ".exe"
#define MySetupName "redis-manager-" + MyAppVersion + "_installer_windows_" + MyArch
#if MyArch == "arm64"
  #define MyArchLabel "ARM64"
#elif MyArch == "86"
  #define MyArchLabel "32-bit"
#else
  #define MyArchLabel "64-bit"
#endif

[Setup]
#if MyArch == "arm64"
AppId={{E3F4A5B6-C7D8-4901-BCDE-F01234567890}
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#elif MyArch == "64"
AppId={{C1D2E3F4-A5B6-4789-9ABC-DEF012345678}
; Prefer native ARM64 installer on Arm PCs (x64compatible would also match Win11 ARM).
ArchitecturesAllowed=x64compatible and not arm64
ArchitecturesInstallIn64BitMode=x64compatible
#else
AppId={{D2E3F4A5-B6C7-4890-ABCD-EF0123456789}
ArchitecturesAllowed=x86compatible and not x64compatible
#endif
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion} ({#MyArchLabel})
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
; User picks the install folder on this page.
DisableDirPage=no
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\dist
OutputBaseFilename={#MySetupName}
SetupIconFile=app.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
MinVersion=10.0
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; One arch per installer; install as redis-manager.exe.
Source: "..\dist\{#MyPortableName}"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent runascurrentuser

[UninstallDelete]
; Runtime extracts (runtime/redis) and config live beside the exe.
Type: filesandordirs; Name: "{app}"

[Code]
function InitializeUninstall(): Boolean;
var
  ResultCode: Integer;
begin
  // Stop a running instance so files can be removed.
  Exec('taskkill.exe', '/F /IM {#MyAppExeName}', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Exec('taskkill.exe', '/F /IM redis-server.exe', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := True;
end;
