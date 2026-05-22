# ApexSU Early Tester Program

## Status

Draft testing phase only. **ApexSU is not release-ready.**

## Safety Warning

Do not test on a daily-driver device. Use a spare device or disposable test environment.

## Who Should Test

- experienced Android/kernel users
- testers with a spare device or disposable environment
- users who can recover via fastboot/firmware restore
- users comfortable collecting logs and writing reproducible bug reports

## Preferred Targets

- Android/GKI target build environments
- disposable real devices
- Cuttlefish / Android-x86 / container-based Android only after kernel integration

## What to Test

- boot stability
- manager detection and connectivity
- `GET_INFO` behavior
- allowlist sync behavior
- grant/deny root flows
- manager identity freshness checks
- module ZIP rejection logic (invalid/special entries)
- diagnostics reporting
- recovery behavior after failure

## What Not to Test

- banking bypass
- anti-cheat bypass
- DRM bypass
- stealth/evasion behavior

## Report Template

- device/environment:
- Android version:
- kernel version:
- architecture:
- install method:
- ApexSU commit:
- test result (pass/fail):
- logs:
- screenshots:
- recovery required?:

## Contact

`cheansewmen@pm.me`
