# Generate diagnostics from structured redacted state

A dedicated diagnostics module produces user-previewable support artifacts from structured state snapshots supplied by other modules. It does not inspect raw protocol frames, credentials, or NetworkManager directly; mandatory secret exclusion and bundle-local identifier aliasing are enforced at this seam so privacy behavior is independently testable and cannot depend on every log caller behaving correctly.
