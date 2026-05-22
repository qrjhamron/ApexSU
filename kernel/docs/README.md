# Kernel Reorganization Notes

This directory is reserved for kernel module architecture and migration
documentation.

## Current Layout

The kernel module is still built from a mostly flat source layout under
`kernel/`. `kernel/Kbuild` lists each object explicitly, for example
`ksu.o`, `allowlist.o`, `setuid_hook.o`, and `supercalls.o`. The existing
`selinux/` directory is the only source subdirectory currently represented in
the object list.

Local includes also assume the flat layout. Most source files include sibling
headers with quoted names such as `"allowlist.h"` or `"ksud.h"`, while
`kernel/selinux/*` uses relative includes such as `"../klog.h"`.

## Proposed Layout

- `kernel/abi/`: ABI and kernel/userspace contract documentation and, after
  review, ABI-facing headers.
- `kernel/auth/`: authorization, allowlist, package identity, manager
  identity, app profile, and trust-state code.
- `kernel/hooks/`: syscall, setuid, su compatibility, file wrapper, seccomp,
  namespace, and unmount hook code.
- `kernel/tests/`: kernel module test notes, fixtures, and future build-safe
  test entry points.
- `kernel/docs/`: architecture notes and migration records for the kernel
  module layout.

## Safe Migration Rule

Do not move existing C or H files until the change is reviewed as a single
build-preserving patch that updates:

- `kernel/Kbuild` object paths.
- All quoted local includes affected by the move.
- Any tooling assumptions in `kernel/Makefile`, `.clangd`, or compile database
  generation.

This pass only creates directories and README files so the build behavior stays
unchanged.
