; SaltyShade 3D installer (Inno Setup 6). Built by tools/make_installer.ps1, which passes:
;   /DAppVersion=2026.11.02.1200  /DAppExe=...\texturecraft.exe  /DLauncherExe=...\SaltyShade3D-Launcher.exe
;   /DUpstreamDir=...\upstream  /DRepo=owner/name  /DOutDir=...\dist  /DStageDir=...(version.txt, update.json)
; Installs per user (no admin prompt), so the launcher can update it silently.

#ifndef AppVersion
  #define AppVersion "0.0.0.0"
#endif
#ifndef Repo
  #define Repo "YOUR_GITHUB_NAME/SaltyShade3D"
#endif
#define Root ".."

[Setup]
AppId={{6B0E2C1A-5A7D-4E39-9C3E-5A17D3ADE3D0}
AppName=SaltyShade 3D
AppVersion={#AppVersion}
AppVerName=SaltyShade 3D {#AppVersion}
AppPublisher=SaltyShade
AppPublisherURL=https://github.com/{#Repo}
AppUpdatesURL=https://github.com/{#Repo}/releases
VersionInfoVersion={#AppVersion}
VersionInfoProductName=SaltyShade 3D
DefaultDirName={localappdata}\Programs\SaltyShade 3D
DefaultGroupName=SaltyShade 3D
DisableProgramGroupPage=yes
DisableDirPage=auto
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutDir}
OutputBaseFilename=SaltyShade3D-{#AppVersion}-Setup
SetupIconFile={#Root}\overlay\assets\saltyshade.ico
UninstallDisplayIcon={app}\SaltyShade3D.exe
UninstallDisplayName=SaltyShade 3D
WizardStyle=modern
WizardSmallImageFile={#Root}\overlay\assets\icons\128.png
WizardImageFile={#Root}\installer\wizard.png
Compression=lzma2/ultra64
SolidCompression=yes
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#AppExe}"; DestDir: "{app}"; DestName: "SaltyShade3D.exe"; Flags: ignoreversion
Source: "{#LauncherExe}"; DestDir: "{app}"; DestName: "SaltyShade3D-Launcher.exe"; Flags: ignoreversion
Source: "{#UpstreamDir}\LICENSE-MIT"; DestDir: "{app}\licenses"; Flags: ignoreversion
Source: "{#UpstreamDir}\LICENSE-APACHE"; DestDir: "{app}\licenses"; Flags: ignoreversion
Source: "{#UpstreamDir}\NOTICE"; DestDir: "{app}\licenses"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#UpstreamDir}\ATTRIBUTION.md"; DestDir: "{app}\licenses"; Flags: ignoreversion skipifsourcedoesntexist
; version.txt (this release) and update.json (where the launcher looks for newer ones).
Source: "{#StageDir}\version.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#StageDir}\update.json"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\SaltyShade 3D"; Filename: "{app}\SaltyShade3D-Launcher.exe"; IconFilename: "{app}\SaltyShade3D.exe"; WorkingDir: "{app}"
Name: "{group}\Uninstall SaltyShade 3D"; Filename: "{uninstallexe}"
Name: "{autodesktop}\SaltyShade 3D"; Filename: "{app}\SaltyShade3D-Launcher.exe"; IconFilename: "{app}\SaltyShade3D.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
; A normal install offers to open the app; a silent update (from the launcher) reopens it.
Filename: "{app}\SaltyShade3D-Launcher.exe"; Parameters: "--no-update"; Description: "{cm:LaunchProgram,SaltyShade 3D}"; Flags: nowait postinstall skipifsilent
Filename: "{app}\SaltyShade3D-Launcher.exe"; Parameters: "--no-update"; Flags: nowait skipifnotsilent
