[Setup]
AppName = se (Save and Execute)
AppVersion = 1.2.0
DefaultDirName = {autopf}\se
OutputBaseFilename = se (Save and Execute) Installer
PrivilegesRequiredOverridesAllowed = Dialog
ArchitecturesInstallIn64BitMode = x64
DefaultGroupName = se
SetupIconFile = images\icon.ico
WizardSmallImageFile = images\wizard-small.bmp
UninstallDisplayName = Save and Execute
UninstallDisplayIcon = {app}\bin\se.exe,0
ChangesEnvironment = yes
OutputDir = target\release
AppPublisher = Brandon Fowler
WizardImageFile = images\wizard.bmp
WizardImageStretch = yes

[Types]
Name: "full"; Description: "Full installation"
Name: "basic"; Description: "Installation without any system integrations"
Name: "custom"; Description: "Custom installation"; Flags: iscustom

[Components]
Name: desktop; Description: Create a desktop icon; Types: full basic
Name: start; Description: Add to start menu; Types: full basic
Name: terminal; Description: Create a Windows Terminal profile; Types: full
Name: path; Description: Add to path; Types: full

[Files]
Source: "target\release\se.exe"; DestDir: "{app}\bin"; DestName: se.exe
Source: "images\icon.ico"; DestDir: "{app}"

[Dirs]
Name: "{app}\bin"
Name: "{commonappdata}\Microsoft\Windows Terminal\Fragments\se"; Components: terminal; Check: IsAdminLoggedOn
Name: "{localappdata}\Microsoft\Windows Terminal\Fragments\se"; Components: terminal; Check: not IsAdminLoggedOn

[Icons]
Name: "{commondesktop}\se (Save and Execute)"; Filename: "{app}\bin\se.exe"; Components: desktop; Check: IsAdminLoggedOn
Name: "{userdesktop}\se (Save and Execute)"; Filename: "{app}\bin\se.exe"; Components: desktop; Check: not IsAdminLoggedOn
Name: "{group}\se (Save and Execute)"; Filename: "{app}\bin\se.exe"; Components: start;

[Registry]
Root: HKCU; Subkey: "Environment"; \
	ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}\bin"; Components: path; \
	Check: not IsAdminLoggedOn and PathNeeded(false, 'Environment')

Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; \
	ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}\bin"; Components: path; \
	Check: IsAdminLoggedOn and PathNeeded(true, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment')

[Run]
Filename: "{app}\bin\se.exe"; Description: "Launch Save and Execute"; Flags: postinstall

[UninstallDelete]
// File must be deleted first, so Dirs entry does not delete folder
Type: files; Name: {commonappdata}\Microsoft\Windows Terminal\Fragments\se\se.json
Type: files; Name: {localappdata}\Microsoft\Windows Terminal\Fragments\se\se.json
Type: dirifempty; Name: {commonappdata}\Microsoft\Windows Terminal\Fragments\se
Type: dirifempty; Name: {localappdata}\Microsoft\Windows Terminal\Fragments\se

[Code]
function PathNeeded(UseSystem: Boolean; Key: String): Boolean;
	var
		Root: integer;
		Path: String;
	begin
	
    if UseSystem then begin
		Root := HKEY_LOCAL_MACHINE;
    end else begin
		Root := HKEY_CURRENT_USER;
    end;

	if not RegQueryStringValue(Root, Key, 'Path', Path) then begin
		Result := True;
		exit;
	end;
	
	Result := Pos(ExpandConstant(';{app}\bin;'), ';' + Path + ';') = 0;
end;

procedure RemovePath(UseSystem: Boolean; Key: String);
var
    Root: Integer;
	Path: String;
    Match: Integer;
begin
    if UseSystem then begin
		Root := HKEY_LOCAL_MACHINE;
    end else begin
		Root := HKEY_CURRENT_USER;
    end;

    if not RegQueryStringValue(Root, Key, 'Path', Path) then exit;

    Match := Pos(ExpandConstant(';{app}\bin;'), ';' + Path + ';');
    
	if Match = 0 then exit;

    Delete(Path, Match - 1, Length(ExpandConstant('{app}\bin')) + 1);
    RegWriteStringValue(Root, Key, 'Path', Path)
end;

procedure CurStepChanged(CurStep:TSetupStep);
	var JSONPath, AppPath, AppPathEsc: String;
	begin

	if CurStep<>ssPostInstall then Exit;
	
	AppPath := ExpandConstant('{app}');

	if IsComponentSelected('terminal') then begin
		if IsAdminInstallMode() then begin
			JSONPath := ExpandConstant('{commonappdata}\Microsoft\Windows Terminal\Fragments\se\se.json')
		end
		else begin
			JSONPath := ExpandConstant('{localappdata}\Microsoft\Windows Terminal\Fragments\se\se.json')
		end;		 

		AppPathEsc := AppPath;
		StringChangeEx(AppPathEsc, '\', '\\', True);

		SaveStringToFile(
			JSONPath,
			'{"profiles":[{"guid":"{ef539101-ae4c-5f93-91a7-967fe62af1ee}","name":"se (Save and Execute)","commandline":"' + AppPathEsc + '\\bin\\se.exe","icon":"' + AppPathEsc + '\\icon.ico"}]}',
			False
		);
	end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
    if CurUninstallStep<>usPostUninstall then Exit;

    RemovePath(false, 'Environment');
	RemovePath(true, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment');
end;