# Kernel Auth Directory

This directory is reserved for authentication, allowlist, package identity,
manager identity, and app-profile authorization code.

## Ownership

Subagent 3 owns this directory during the directory reorganization work.
Keep changes here scoped to auth-domain organization notes unless a later
controller-approved migration moves source files.

## Planned Migration

Likely future candidates include files such as allowlist, APK signature,
manager identity, app profile, package observer, and throne tracking code that
currently live in the flat `kernel/` directory.

Do not move existing C or H files in this pass. Several of those files are
already modified in the working tree, and the current build lists their object
files directly from `kernel/Kbuild`.

## Build Impact

This first step intentionally adds documentation only. It does not change
compiled object lists, include paths, Kconfig symbols, or module output.
