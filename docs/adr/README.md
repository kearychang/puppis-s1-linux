# Architecture decision records

ADRs capture constraints that are costly or unsafe to rediscover. A superseded record remains visible so readers can understand why the direction changed.

1. [Allowlist firmware for device mutations](0001-allowlist-firmware-for-device-mutations.md)
2. [Use Rust and Tauri for the desktop application](0002-use-rust-and-tauri-for-the-desktop-application.md)
3. [Use NetworkManager and polkit without a root helper](0003-use-networkmanager-and-polkit-without-a-root-helper.md)
4. [Reconcile before recovering ambiguous mutations](0004-reconcile-before-recovering-ambiguous-mutations.md)
5. [Modify only owned network profiles](0005-modify-only-owned-network-profiles.md)
6. [Bootstrap device identity with a temporary network configuration](0006-bootstrap-device-identity-with-a-temporary-network-configuration.md)
7. [Qualify and separately confirm factory reset](0007-qualify-and-separately-confirm-factory-reset.md)
8. [Model operational status as independent dimensions](0008-model-operational-status-as-independent-dimensions.md)
9. [Respect the host's existing upstream routing](0009-respect-the-hosts-existing-upstream-routing.md)
10. [Validate the proprietary protocol strictly](0010-validate-the-proprietary-protocol-strictly.md)
11. [Serialize device protocol operations](0011-serialize-device-protocol-operations.md)
12. [Hide the proprietary protocol behind task-oriented operations](0012-hide-the-proprietary-protocol-behind-task-oriented-operations.md)
13. [Coordinate workflows behind one application interface](0013-coordinate-workflows-behind-one-application-interface.md)
14. [Separate USB discovery from host sharing](0014-separate-usb-discovery-from-host-sharing.md)
15. [Generate diagnostics from structured redacted state](0015-generate-diagnostics-from-structured-redacted-state.md)
16. [Expose typed safe operation failures](0016-expose-typed-safe-operation-failures.md)
17. [Publish revisioned redacted state snapshots](0017-publish-revisioned-redacted-state-snapshots.md)
18. [Serialize all host and device mutations](0018-serialize-all-host-and-device-mutations.md)
19. [Make no outbound internet requests](0019-make-no-outbound-internet-requests.md)
20. [Persist only minimal non-secret application state](0020-persist-only-minimal-non-secret-application-state.md)
21. [Separate hardware mutation tests from normal automation](0021-separate-hardware-mutation-tests-from-normal-automation.md)
22. [Remain a subproject of the parent repository](0022-remain-a-subproject-of-the-parent-repository.md) — superseded by ADR 0025
23. [Exclude vendor artifacts from source and releases](0023-exclude-vendor-artifacts-from-source-and-releases.md)
24. [Persist only explicitly saved client recognition](0024-persist-only-explicitly-saved-client-recognition.md)
25. [Publish with filtered standalone history](0025-publish-with-filtered-standalone-history.md)
