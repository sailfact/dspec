# Data Collected by Intune "Collect Diagnostics"

This is the canonical manifest of what the Microsoft Intune **Collect Diagnostics** remote
action captures from a Windows device. Use it to:

- Know what *should* be in a package, so a missing item is recognised as a failed collection
  (a zero-byte `No Results - Error [0x...]` file) rather than assumed absent.
- Map a registry key, command, event channel, or file path back to the investigation area it
  feeds (see SKILL.md Step 3).
- Identify additional data sources not yet called out as a dedicated area — e.g. `SetupDiag`
  results, `CBS.log`, measured boot, EPM Agent logs, Device Inventory — when a finding points
  outside the core areas.

Source: Microsoft Learn — *Collect diagnostics from a Windows device*. The simplified flattened
ZIP format is produced once **KB5011543** (Windows 10) or **KB5011563** (Windows 11) is installed:

- A flattened structure where each collected log is named to match the data collected.
- When a collection yields multiple files, a folder is created for them.

The lists below are in the **same order as the diagnostic ZIP**.

## Registry Keys

| Registry key |
|--------------|
| `HKLM\SOFTWARE\Microsoft\CloudManagedUpdate` |
| `HKLM\SOFTWARE\Microsoft\EPMAgent` |
| `HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\DeviceHealthMonitoring` |
| `HKLM\SOFTWARE\Microsoft\IntuneManagementExtension` |
| `HKLM\SOFTWARE\Microsoft\SystemCertificates\AuthRoot` |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Authentication\LogonUI` |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Internet Settings` |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall` |
| `HKLM\SOFTWARE\Microsoft\DeviceInventory` |
| `HKLM\SOFTWARE\Policies\Microsoft\Cryptography\Configuration\SSL` |
| `HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall` |
| `HKLM\SYSTEM\CurrentControlSet\Control\SecurityProviders\SCHANNEL` |
| `HKLM\SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy\Mdm` |
| `HKLM\SYSTEM\Setup\SetupDiag\Results` |

## Commands

| Command |
|---------|
| `%programfiles%\windows defender\mpcmdrun.exe -GetFiles` |
| `%windir%\system32\certutil.exe -store` |
| `%windir%\system32\certutil.exe -store -user my` |
| `%windir%\system32\Dsregcmd.exe /status` |
| `%windir%\system32\ipconfig.exe /all` |
| `%windir%\system32\mdmdiagnosticstool.exe` |
| `%windir%\system32\msinfo32.exe /report %temp%\MDMDiagnostics\msinfo32.log` |
| `%windir%\system32\netsh.exe advfirewall show allprofiles` |
| `%windir%\system32\netsh.exe advfirewall show global` |
| `%windir%\system32\netsh.exe lan show profiles` |
| `%windir%\system32\netsh.exe winhttp show proxy` |
| `%windir%\system32\netsh.exe wlan show profiles` |
| `%windir%\system32\netsh.exe wlan show wlanreport` |
| `%windir%\system32\ping.exe -n 50 localhost` |
| `%windir%\system32\pnputil.exe /enum-drivers` |
| `%windir%\system32\powercfg.exe /batteryreport /output - %temp%\MDMDiagnostics\battery-report.html` |
| `%windir%\system32\powercfg.exe /energy /output %temp%\MDMDiagnostics\energy-report.html` |

## Event Viewer

| Event |
|-------|
| `Application` |
| `Microsoft-Windows-AppLocker/EXE and DLL` |
| `Microsoft-Windows-AppLocker/MSI and Script` |
| `Microsoft-Windows-AppLocker/Packaged app-Deployment` |
| `Microsoft-Windows-AppLocker/Packaged app-Execution` |
| `Microsoft-Windows-AppxPackaging/Operational` |
| `Microsoft-Windows-Bitlocker/Bitlocker Management` |
| `Microsoft-Windows-HelloForBusiness/Operational` |
| `Microsoft-Windows-SENSE/Operational` |
| `Microsoft-Windows-SenseIR/Operational` |
| `Microsoft-Windows-Windows Firewall With Advanced Security/Firewall` |
| `Microsoft-Windows-WinRM/Operational` |
| `Microsoft-Windows-WMI-Activity/Operational` |
| `Microsoft-Windows-AppXDeployment/Operational` |
| `Microsoft-Windows-AppXDeploymentServer/Operational` |
| `Setup` |
| `System` |

All event channels are exported as binary `.evtx` and cannot be parsed in this environment.
Note their presence and recommend `Get-WinEvent` / Event Viewer for deep analysis (see the
report template reference for the binary-log handling guidance).

## Files

| Path |
|------|
| `%ProgramData%\Microsoft\DiagnosticLogCSP\Collectors\*.etl` |
| `%ProgramFiles%\Microsoft EPM Agent\Logs\*.*` |
| `%ProgramFiles%\Microsoft Device Inventory Agent\Logs` |
| `%ProgramData%\Microsoft\IntuneManagementExtension\Logs\*.*` |
| `%ProgramData%\Microsoft\Windows Defender\Support\MpSupportFiles.cab` |
| `%ProgramData%\Microsoft\Windows\WlanReport\wlan-report-latest.html` |
| `%ProgramData%\USOShared\logs\system\*.etl` |
| `%ProgramData%\Microsoft Update Health Tools\Logs\*.etl` |
| `%temp%\CloudDesktop\*.log` |
| `%temp%\MDMDiagnostics\battery-report.html` |
| `%temp%\MDMDiagnostics\energy-report.html` |
| `%temp%\MDMDiagnostics\mdmlogs-<Date/Time>.cab` |
| `%temp%\MDMDiagnostics\msinfo32.log` |
| `%windir%\ccm\logs\*.log` |
| `%windir%\ccmsetup\logs\*.log` |
| `%windir%\logs\CBS\cbs.log` |
| `%windir%\logs\measuredboot\*.*` |
| `%windir%\logs\Panther\unattendgc\setupact.log` |
| `%windir%\logs\SoftwareDistribution\ReportingEvents\*.log` |
| `%windir%\Logs\SetupDiag\SetupDiagResults.xml` |
| `%windir%\logs\WindowsUpdate\*.etl` |
| `%windir%\SensorFramework\*.etl` |
| `%windir%\system32\config\systemprofile\AppData\Local\mdm\*.log` |
| `%windir%\temp\%computername%*.log` |
| `%windir%\temp\officeclicktorun*.log` |
| `%TEMP%\winget\defaultstate*.log` |

### Notes on file captures

- `.etl` files (DiagnosticLogCSP, USOShared, Update Health Tools, WindowsUpdate, SensorFramework)
  are binary ETW traces — not readable here. Decode on-device with the relevant tool
  (`Get-WindowsUpdateLog` for WU traces).
- `MpSupportFiles.cab` and `mdmlogs-<Date/Time>.cab` are archives; note their presence rather
  than expanding them.
- `reportingevents.log` (under `SoftwareDistribution`) and `cbs.log` are plain text and useful
  for Windows Update / servicing history.
- `SetupDiagResults.xml` is the best first stop for feature-update/upgrade failures.
- EPM Agent and Device Inventory logs are present when those features are deployed; their absence
  (a `No Results - Error` entry) is expected on tenants not using them.

> While there's no intent to collect personal data, diagnostics may include user-identifiable
> information such as user or device name. Handle packages accordingly.