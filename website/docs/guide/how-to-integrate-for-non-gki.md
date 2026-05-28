# GKI support policy

Non-GKI devices are not supported by ApexSU.

- Installation is always blocked on non-GKI devices.
- There is no local LKM workaround for non-GKI devices.
- Repository LKM and Local LKM are only for supported GKI devices.

To check support, open ApexSU Manager and verify your kernel version includes `android`.

Example supported format:

`5.10.209-android12-9-00016-g7c6bbcca33e1`
