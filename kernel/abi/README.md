# Kernel ABI Directory

This directory is reserved for kernel/userspace ABI definitions and
compatibility notes.

## Ownership

Subagent 3 owns this directory during the directory reorganization work.
Keep changes here focused on ABI contracts, syscall/supercall wire shapes,
and compatibility documentation.

## Planned Migration

Candidate files for a later reviewed migration include ABI-facing headers and
call-contract documentation currently kept in the flat `kernel/` directory.
Do not move existing C or H files into this directory until the matching
`kernel/Kbuild` object paths and every quoted include are updated together.

## Build Impact

This first step intentionally adds documentation only. It does not change
compiled object lists, include paths, Kconfig symbols, or module output.
