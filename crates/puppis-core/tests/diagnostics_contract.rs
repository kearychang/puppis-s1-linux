use puppis_core::{
    Application, DeviceIdentity, InMemoryEnvironment, OperationFailure, PuppisCandidate,
    QUALIFIED_FIRMWARE, UsbLinkSpeed,
};

fn verified_application() -> Application {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "usb:enx020000000101:1790".into(),
        interface_name: "enx020000000101".into(),
        display_name: "USB network adapter (enx020000000101)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_identity(DeviceIdentity {
        model: "P1411".into(),
        firmware: QUALIFIED_FIRMWARE.into(),
        role_code: "1".into(),
    });
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.verify_selected_puppis().unwrap();
    app
}

#[test]
fn diagnostics_retain_the_latest_typed_safe_failure() {
    let app = verified_application();
    app.remember_failure(&OperationFailure::safe(
        "network_authorization_denied",
        "NetworkManager authorization was denied.",
        "Keep the current network state or try again.",
    ));

    let bundle = app.preview_diagnostics().unwrap();

    assert!(bundle.preview.contains("network_authorization_denied"));
    assert!(
        bundle
            .preview
            .contains("NetworkManager authorization was denied.")
    );
    assert!(!bundle.preview.contains("org.freedesktop"));
}

#[test]
fn diagnostics_preview_retains_support_evidence_but_aliases_identifiers() {
    let bundle = verified_application().preview_diagnostics().unwrap();

    assert!(bundle.preview.contains(QUALIFIED_FIRMWARE));
    assert!(bundle.preview.contains("candidate-1"));
    assert!(!bundle.preview.contains("enx020000000101"));
    assert!(!bundle.preview.to_ascii_lowercase().contains("password"));
    assert!(!bundle.preview.to_ascii_lowercase().contains("ssid"));
    assert!(!bundle.preview.contains("raw_frame"));
}

#[test]
fn export_writes_exactly_the_preview_only_after_an_explicit_path() {
    let app = verified_application();
    let bundle = app.preview_diagnostics().unwrap();
    let path = std::env::temp_dir().join(format!("puppis-diagnostics-{}.json", std::process::id()));

    app.export_diagnostics(&path, &bundle.preview).unwrap();

    assert_eq!(std::fs::read_to_string(&path).unwrap(), bundle.preview);
    std::fs::remove_file(path).unwrap();
}
