# Kernel Hooks Directory

This directory is reserved for kernel hook implementations and hook-related
helpers.

## Ownership

Subagent 3 owns this directory during the directory reorganization work.
Keep changes here focused on hook-domain organization notes unless a later
controller-approved migration moves source files.

## Planned Migration

Likely future candidates include syscall hook manager, setuid hook, sucompat,
supercalls, file wrapper, seccomp cache, mount namespace, and kernel unmount
code that currently live in the flat `kernel/` directory.

Do not move existing C or H files in this pass. These files are included by
flat quoted paths and are named directly in `kernel/Kbuild`.

## Build Impact

This first step intentionally adds documentation only. It does not change
compiled object lists, include paths, Kconfig symbols, or module output.
