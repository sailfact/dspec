---
name: intune-diag-review
description: >
  Review and triage Windows device diagnostic packages collected via the Microsoft Intune
  "Collect Diagnostics" remote action. Use this skill whenever a user uploads or references
  an Intune diagnostic ZIP (named DiagLogs-*.zip or similar), asks to analyse device logs
  from Intune, troubleshoot Intune-managed Windows device issues, review Autopilot failures,
  check enrollment or compliance status, investigate app deployment failures, or diagnose
  Entra ID join / PRT / WHfB problems. Also triggers for queries like "what's wrong with
  this device", "why isn't this policy applying", "check the diag logs", or "review these
  Intune logs". Always use this skill before attempting to read individual files from a
  diagnostic ZIP — it defines the correct extraction and analysis workflow. The goal is
  general triage: surface all notable findings across all areas, not a single focused
  investigation.
---

# Intune Device Diagnostics Review

This skill guides analysis of Windows diagnostic packages collected via the Intune **Collect Diagnostics** remote action. Packages are ZIP files named `DiagLogs-<DeviceName>-<Timestamp>Z.zip`.

**Bundled resources:**
- `scripts/extract-diag.sh` — orient, extract readable files + IME logs, decode `.reg` to UTF-8.
- `scripts/scan-errors.sh` — first-pass error triage (failed collections + IME log errors).
- `references/data-collected.md` — canonical manifest of everything the action captures.
- `references/report-template.md` — output report template + per-area remediation tables.

## Package Structure

The ZIP uses a **flattened** layout with numbered file prefixes:

| Category | Naming Pattern | Format |
|---|---|---|
| Registry exports | `(N) RegistryKey HKLM_... export.reg` | Plain text `.reg` |
| Command outputs | `(N) Command windir_... output.log` | Plain text |
| Event logs | `(N) Events ... Events.evtx` | Binary `.evtx` (not directly readable) |
| Folder/file captures | `(N) FoldersFiles .../<filename>` | Various |
| Error entries | `(N) No Results - Error [0x...]` | Zero-byte files (important!) |
| Manifest | `results.xml` | XML |

**Key rule:** Zero-byte files named `No Results - Error [0x...]` are NOT missing files — they are meaningful failure indicators. `0x80070001` = operation not supported, `0x80070002` = file not found, `0x8000ffff` = unexpected error (often DISM on ARM/cloud PC).

**Full collection manifest:** For the complete, canonical list of every registry key, command, event channel, and file path the Collect Diagnostics action captures — in ZIP order — see `references/data-collected.md`. Consult it to confirm whether an item is genuinely missing (a failed collection) versus simply not part of the package, and to map a file back to its investigation area.

## Step 1: Extract & orient

Run the extraction script — it prints the `results.xml` manifest, extracts the text-readable
files (`.log`, `.reg`, `results.xml`) and IME logs to a working directory, and decodes every
`.reg` file from UTF-16LE to UTF-8 in place:

```bash
bash scripts/extract-diag.sh <zip> [workdir]   # workdir defaults to /tmp/diagreview
```

From the manifest it prints, identify:
- Items with `HRESULT="0"` → succeeded
- Items with non-zero HRESULT → failed collections (note which keys/commands failed)
- The `<OutputFileFormat>` element (should be `flattened`)

Why the script handles the `.reg` decode for you: **Windows registry exports are UTF-16LE.**
`cat`/`grep` on the raw file silently returns nothing, which is a common source of false
"no data found" results. Because the script normalises them to UTF-8 once, you can grep the
extracted `.reg` files directly afterwards — no per-file `iconv` needed.

Binary `.evtx` and `.etl` files cannot be parsed in this environment — note their presence (see
`references/data-collected.md` for the full list of what's in the package) but do not attempt
to read them.

## Step 2: Scan for errors

Run the error-scan script for a first pass — it surfaces failed collections (non-zero HRESULT
from `results.xml`) and error/warning lines from the IME logs:

```bash
bash scripts/scan-errors.sh [workdir]          # defaults to /tmp/diagreview
```

Use this to decide where to focus in Step 3, then dig into the relevant areas in detail.

## Step 3: Triage by investigation area

Work through areas in priority order based on what the user is investigating. If no specific focus was given, work through all areas and summarise findings.

---

### Area A: Device Identity & Entra Join Status

**File:** `(31) Command windir_system32_Dsregcmd_exe_status output.log`

Key fields to check:

| Field | Healthy Value | Problem if... |
|---|---|---|
| `AzureAdJoined` | `YES` | `NO` → not joined, re-enrol needed |
| `DomainJoined` | `NO` (cloud-only) | `YES` on Entra-only tenant = unexpected |
| `DeviceAuthStatus` | `SUCCESS` | Any error = cert/TPM issue |
| `TpmProtected` | `YES` | `NO` = device key not hardware-backed |
| `AzureAdPrt` | `YES` (when user logged in) | `NO` = PRT missing, causes SSO/Conditional Access failures |
| `WamDefaultSet` | no error | `ERROR 0x80070520` = WAM credential missing (common on SYSTEM-context runs) |
| `NgcSet` | `YES` (if WHfB deployed) | `NO` = Windows Hello not provisioned |
| `PreReqResult` | `WillProvision` or provisioned | `WillNotProvision` = check `IsUserAzureAD` and `PolicyEnabled` |

Note: `WamDefaultSet ERROR` is expected/benign when `dsregcmd` runs as SYSTEM (e.g. during diagnostics collection). It does not indicate a real problem unless the user is also reporting SSO failures.

---

### Area B: MDM Enrollment Status

**File:** `(4) RegistryKey HKLM_Software_Microsoft_Enrollments export.reg`

Look for enrollment entries. Each enrollment GUID subkey should have:
- `ProviderID` = `MS DM Server` (Intune)
- `EnrollmentState` = `1` (enrolled) — `3` = unenrolled
- `AADResourceID` containing `https://manage.microsoft.com`

**File:** `(6) RegistryKey HKLM_Software_Microsoft_IntuneManagementExtension export.reg`

Check for the `LastSuccessfulSync` value and any error codes under the IME key.

**File:** `(33) Command windir_system32_mdmdiagnosticstool... output.log`

Should end with `Succeeded to CollectLog at: ...mdmlogs-....cab`. Failure here = MDM stack issue.

---

### Area C: IME Logs (App Deployment, Scripts, Compliance)

**Files:** `(64) FoldersFiles ProgramData_Microsoft_IntuneManagementExtension_Logs/*`

IME log format uses SCCM-style log syntax:
```
<![LOG[<message>]LOG]!><time="HH:MM:SS" date="M-D-YYYY" component="..." type="N" ...>
```
- `type="1"` = Informational
- `type="2"` = Warning  
- `type="3"` = Error

**Priority logs to scan:**

| Log | What to look for |
|---|---|
| `intunemanagementextension.log` | Sync cycles, policy receipt, errors |
| `appworkload.log` | Win32 app / WinGet app installs, success/failure |
| `agentexecutor.log` | Script execution, applicability checks |
| `healthscripts.log` | Proactive Remediation detection/remediation results |
| `clienthealth.log` | IME self-health, restart/reinit events |
| `sensor.log` | Device health monitoring, compliance signals |

The Step 2 `scan-errors.sh` run already greps these logs for `error|failed|0x8|exception`. To
scan a single log in more detail, grep it directly (the IME logs are plain text):
```bash
grep -i "error\|failed\|0x8\|exception" /tmp/diagreview/ime/appworkload.log | head -50
```

Common IME error patterns:
- `Win32App install failed with exit code` → check the specific exit code
- `Remediation script failed` → script runtime error in Proactive Remediations
- `Agent executor timed out` → script exceeded time limit
- `PolicyManager` errors → policy conflict or CSP not supported

---

### Area D: Windows Update / Patch Status

**File:** `(19) RegistryKey HKLM_SOFTWARE_Microsoft_Windows_CurrentVersion_WindowsUpdate_Orchestrator export.reg`

Look for:
- `UsoLastScanResult` — `0x00000000` = success
- `RebootRequired` — presence/value indicates pending reboot
- `OobeInProgress` — should not be set post-enrolment

**File:** `(86) FoldersFiles windir_SoftwareDistribution_ReportingEvents_log/reportingevents.log`

Shows WU scan and install history. Look for failed update events.

Binary `.etl` files in `windir_Logs_WindowsUpdate_etl/` and `usoshared_logs/` cannot be read here — note their presence and recommend exporting with `Get-WindowsUpdateLog` if deeper analysis needed.

---

### Area E: Network & Proxy

**File:** `(32) Command windir_system32_ipconfig_exe_all output.log`

Check:
- DNS Suffix (should match tenant/domain if applicable)
- IP assignment (DHCP vs static)
- Adapter names and active connections

**File:** `(38) Command windir_system32_netsh_exe_winhttp_show_proxy output.log`

`Direct access (no proxy server)` = no WinHTTP proxy configured. If environment uses a proxy, this should show the proxy — missing proxy config here breaks Intune communication.

**File:** `(35) Command windir_system32_netsh_exe_advfirewall_show_allprofiles output.log`

All three profiles (Domain/Private/Public) should show:
- `State: ON`
- `FirewallPolicy: BlockInbound,AllowOutbound`
- `LocalFirewallRules: N/A (GPO-store only)` — confirms MDM firewall policy is applied (local rules blocked)

---

### Area F: Security / Certificate Store

**File:** `(27) Command windir_system32_certutil_exe_-store output.log`

Look for:
- Device certificate issued by `MS-Organization-Access` (Intune device cert)
- Any expired certificates
- `CertUtil: -store command completed successfully` at the end

**File:** `(26) Command windir_system32_certutil_exe_-store_-user_my output.log`

User certificate store — may be empty if run as SYSTEM.

---

### Area G: Defender / Security

**File:** `(26) Command programfiles_windows_defender_mpcmdrun_exe_-GetFiles output.log`

Check for collection success and any Defender service errors.

Binary `.evtx` files for AppLocker, SENSE, SenseIR, BitLocker, WHfB are present but not directly readable. Note their presence and advise using Event Viewer or `Get-WinEvent` if specific security event analysis is needed.

---

### Area H: Errors & Missing Data

Always check `results.xml` for failed collections. Notable error codes:

| HRESULT | Meaning | Action |
|---|---|---|
| `0x80070001` | Operation not supported | Often expected (e.g. EPMAgent not deployed) |
| `0x80070002` | File not found | Path doesn't exist on device |
| `0x80070003` | Path not found | Similar to above |
| `0x8000ffff` | Unexpected error | DISM failures common on ARM/Cloud PC — usually benign |
| `0x80070005` | Access denied | Permissions issue during collection |

---

## Step 4: Produce the summary report & remediation

Once triage is complete, produce the report and the tailored further-investigation section
following **`references/report-template.md`**. That reference defines:

- The delivery rules (deliver in chat; only generate a `.docx` via the `docx` skill if the user asks).
- The exact summary report template — device identity, MDM enrollment, issues found (with
  `[CRITICAL]` / `[WARNING]` / `[INFO]` severity), binary logs present, and confirmed-OK areas.
- The per-area remediation tables (Finding → Next Step). Include only the subsections relevant
  to what was actually found — skip clean areas.
- Guidance for deeper binary `.evtx` / `.etl` analysis on-device when a finding needs it.