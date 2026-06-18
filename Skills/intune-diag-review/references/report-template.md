# Report Template & Remediation Reference

The output structure for an Intune diagnostics review, plus the tailored
further-investigation tables used to populate it. SKILL.md Step 4 points here once analysis
is complete.

**Contents**
- [Delivery rules](#delivery-rules)
- [Summary report template](#summary-report-template)
- [Further investigation & remediation](#further-investigation--remediation)
  - [Device identity / Entra join](#device-identity--entra-join-issues)
  - [MDM enrollment](#mdm-enrollment-issues)
  - [App deployment](#app-deployment-issues)
  - [Windows Update](#windows-update-issues)
  - [Network / connectivity](#network--connectivity-issues)
  - [Certificates](#certificate-issues)
- [Binary log analysis](#binary-log-analysis-when-needed)

## Delivery rules

- Deliver the report directly in the chat. Do **not** generate a file unless the user asks.
- After presenting the report, offer: *"Want me to generate this as a Word document?"*
- If the user accepts, use the `docx` skill to produce a properly formatted `.docx` of the
  same report.
- In the Further Investigation section, only include subsections relevant to what was actually
  found — skip clean areas.

## Summary report template

ALWAYS follow this structure:

```markdown
## Intune Diagnostics Review: `<DeviceName>`
**Collection date:** `<timestamp from zip filename>`
**Tenant:** `<TenantName from dsregcmd>`

### Device Identity
- Entra ID Join: [status]
- TPM Protected: [yes/no]
- PRT Status: [present/missing + reason if missing]
- Windows Hello: [provisioned/not provisioned]

### MDM Enrollment
- Enrollment State: [enrolled/unenrolled]
- Last IME Sync: [timestamp if available]

### Issues Found
Numbered list of findings. For each: **severity** (`[CRITICAL]` / `[WARNING]` / `[INFO]`),
a plain-English description of what was found, and a concrete recommended action.

### Binary Logs Present (not analysed)
List of `.evtx` and `.etl` files by event channel name — note what each covers so the user
knows what to investigate manually if needed.

### Confirmed OK
Brief list of areas that checked out cleanly — reassures the user what isn't the problem.
```

## Further investigation & remediation

Append a tailored section after the summary, drawn from the tables below. Include only the
subsections matching the findings.

### Device identity / Entra join issues

| Finding | Next Step |
|---|---|
| `AzureAdJoined: NO` | Re-run `dsregcmd /join` or re-enrol the device via Intune |
| `DeviceAuthStatus` not SUCCESS | Check TPM health: `tpm.msc`, run `certlm.msc` and verify device cert under Personal |
| `AzureAdPrt: NO` (confirmed, not SYSTEM-context) | User should sign out and back in; check Conditional Access policies blocking token issuance; review `dsregcmd /refreshprt` |
| `WamDefaultSet ERROR` (user-session confirmed) | Run `dsregcmd /status` interactively as the user; check WAM logs via `mdmdiagnosticstool` |
| `NgcSet: NO` / `WillNotProvision` | Verify WHfB policy is assigned to device/user in Intune; confirm `IsUserAzureAD: YES` when run interactively |

### MDM enrollment issues

| Finding | Next Step |
|---|---|
| `EnrollmentState: 3` (unenrolled) | Re-enrol: Settings > Accounts > Access work or school > Enrol; or use Autopilot reset if applicable |
| Intune device cert expired | Check cert in `certlm.msc > Personal`; trigger re-enrolment or SCEP renewal policy; verify SCEP/PKCS profile is assigned |
| MDM Diagnostic Tool failed | Run `mdmdiagnosticstool.exe -out C:\Temp\diag` manually on the device; check MDM stack with `eventvwr > Microsoft-Windows-DeviceManagement-Enterprise-Diagnostics-Provider` |
| IME not syncing | Restart the `IntuneManagementExtension` service; check `intunemanagementextension.log` for repeated auth failures |

### App deployment issues

| Finding | Next Step |
|---|---|
| Win32 app install failure with non-zero exit code | Look up the exit code (Win32 error, MSI error, or app-specific); check `agentexecutor.log` for the full install command and output |
| Proactive Remediation script failure | Check `healthscripts.log` for the script output/error; re-run detection script manually on device as SYSTEM using `psexec -s powershell.exe` |
| App stuck in "Installing" | Check `appworkload.log` for timeout or content download errors; verify Delivery Optimization is not blocked |
| WinGet app not found | Verify the WinGet package ID is correct and available in WinGet source; check `agentexecutor.log` for `ApplicabilityCheck` results |

### Windows Update issues

| Finding | Next Step |
|---|---|
| `UsoLastScanResult` non-zero | Run `wuauclt /detectnow` or `UsoClient StartScan`; check WU service is running |
| `RebootRequired` set | Device needs a reboot to complete pending updates; schedule via Intune Update Ring or notify user |
| WU ETL analysis needed | On the device, run `Get-WindowsUpdateLog` (PowerShell) to decode `.etl` files into a readable `WindowsUpdate.log` |

### Network / connectivity issues

| Finding | Next Step |
|---|---|
| WinHTTP proxy missing (but environment uses proxy) | Set proxy: `netsh winhttp set proxy <proxy>:<port>`; or push via Intune Network/VPN profile |
| DNS resolution failures (visible in ipconfig) | Check DNS suffix search list; verify DNS server responds to Intune endpoints |
| Intune upload endpoint blocked | Verify the regional blob storage URL is reachable (see Microsoft docs for regional endpoints); check proxy/firewall allow-list |

### Certificate issues

| Finding | Next Step |
|---|---|
| Expired certificates in machine store | Remove via `certlm.msc`; or script removal: `Get-ChildItem Cert:\LocalMachine\My | Where-Object {$_.NotAfter -lt (Get-Date)} | Remove-Item` |
| Missing MS-Organization-Access cert | Device cert may have been deleted or corrupted; re-enrol to Entra ID |
| Intune Root CA cert expired | This is a Microsoft-managed cert — verify actual expiry by opening `certlm.msc`; if expired, raise with Microsoft Support |

## Binary log analysis (when needed)

For findings that need deeper event log investigation, use these commands on the device:

```powershell
# Read a specific event log by channel name
Get-WinEvent -LogName "Microsoft-Windows-HelloForBusiness/Operational" -MaxEvents 50

# Filter by error level
Get-WinEvent -LogName "System" | Where-Object {$_.Level -le 2} | Select-Object TimeCreated, Id, Message | Select-Object -First 20

# Decode Windows Update ETL traces (run on device)
Get-WindowsUpdateLog

# Export AppLocker events for review
Get-WinEvent -LogName "Microsoft-Windows-AppLocker/EXE and DLL" | Export-Csv C:\Temp\applocker.csv
```

- **`.evtx` event logs** (Application, System, Security, AppLocker, BitLocker, SENSE, WHfB, etc.)
  are binary and cannot be parsed here. Acknowledge their presence and recommend `Get-WinEvent`
  or Event Viewer for deep event log analysis.
- **`.etl` trace logs** (ETW traces from Autopilot, device provisioning, WU, SensorFramework)
  are also binary. For Autopilot ETLs specifically, recommend `TpmHliVerify` or the Autopilot
  Diagnostics script.
- **Sensitive data**: Diagnostic packages contain device IDs, tenant IDs, certificate
  thumbprints, IP addresses, and installed software lists. Handle accordingly.
- **Context matters**: Some findings (e.g. `WamDefaultSet ERROR`, missing user cert store) are
  expected when diagnostics are collected as SYSTEM and do not indicate real issues.