# Kernel Build Compatibility

`make -C /lib/modules/$(uname -r)/build M=$PWD/kernel modules` is only a
weak host-header smoke test unless `CONFIG_KSU` is forced or selected by the
target kernel configuration. With `CONFIG_KSU` unset, kbuild can parse this
directory and exit successfully without compiling the `apexsu` objects.

`CONFIG_KSU=m` is stronger evidence because it compiles the module object list
from `kernel/Kbuild`. On distro header-only kernels this can still fail before
it reaches Android compatibility checks. ApexSU includes code that uses private
SELinux internals such as `security/selinux/objsec.h`, `selinux_inode()`, and
`selinux_cred()`. Those headers are part of a full kernel source tree, not the
public UAPI/module header set commonly installed under `/lib/modules`.

Release evidence for the kernel module must come from the intended Android/GKI
kernel source and configuration, with the target architecture and toolchain
selected by that kernel tree. A successful Ubuntu x86_64 host-header build does
not prove arm64 Android/GKI module compatibility.
