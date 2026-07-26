# Remain a subproject of the parent repository

**Status:** Superseded by ADR 0025.

Puppis S1 Manager for Linux remains a self-contained subproject inside the existing `kearychang/project` repository rather than creating a nested standalone repository. All source, documentation, fixtures, packaging, and release automation for the application stay under `puppis_s1_linux` and must not depend on or modify sibling projects, accepting shared repository history in exchange for avoiding another repository.
