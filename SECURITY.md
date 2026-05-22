# ApexSU Security Policy

## Scope and Purpose

ApexSU is for legitimate device-owner root management, research, development, and transparent system control.

## How to Report a Vulnerability

Preferred channel:

- GitHub Security Advisories (private):
  `https://github.com/qrjhamron/ApexSU/security/advisories/new`

Fallback contact:

- `cheansewmen@pm.me`

Do not open public issues for unpatched vulnerabilities.

If a finding is confirmed to be purely upstream KernelSU behavior, we will coordinate upstream disclosure. Otherwise, report directly to ApexSU first.

## Supported Versions

Security triage is prioritized for:

- current `main` branch
- active security branches under review
- latest published release line (when available)

Older snapshots may receive limited or no security fixes.

## Severity Rubric

- **Critical**: privilege boundary break, kernel-level arbitrary code execution, or broad compromise.
- **High**: strong integrity/security control bypass with realistic exploit path.
- **Medium**: constrained bypass or denial of service in privileged components.
- **Low**: hardening gap or low-impact misconfiguration risk.

## Response Timeline (Target)

- Initial acknowledgement: within **72 hours**
- Triage + severity assignment: within **7 days**
- Fix/mitigation plan update: within **14 days**

Complex kernel/device-specific issues may take longer; status updates should still be provided.

## What to Include in a Report

- affected ApexSU commit/branch
- environment (device, Android version, kernel version, arch)
- reproduction steps
- observed vs expected behavior
- impact assessment
- logs/crash output and any PoC details

## Out of Scope Abuse Requests

The following are out of scope and will be rejected:

- banking bypass
- anti-cheat bypass
- DRM bypass
- stealth/evasion requests
- credential theft
- unauthorized access

## Disclosure and Coordination

We follow coordinated disclosure and will credit valid reports when possible. Public details are shared only after a fix or explicit mitigation is available.
