use puppis_core::{
    Application, DeviceIdentity, InMemoryEnvironment, PuppisCandidate, QUALIFIED_FIRMWARE,
    UsbLinkSpeed,
};

fn selected_environment(identity: DeviceIdentity) -> InMemoryEnvironment {
    InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_identity(identity)
}

#[test]
fn protocol_identity_promotes_a_candidate_to_a_verified_puppis() {
    let app = Application::new(selected_environment(DeviceIdentity {
        model: "P1411".into(),
        firmware: QUALIFIED_FIRMWARE.into(),
        role_code: "1".into(),
    }));
    app.refresh_candidates().unwrap();

    let snapshot = app.verify_selected_puppis().unwrap();

    assert_eq!(snapshot.verified_puppis.unwrap().model, "P1411");
    assert_eq!(snapshot.protocol.summary, "Verified P1411");
    assert_eq!(snapshot.protocol.level, puppis_core::StatusLevel::Healthy);
    assert!(snapshot.mutations_qualified);
}

#[test]
fn unknown_firmware_stays_usefully_read_only() {
    let app = Application::new(selected_environment(DeviceIdentity {
        model: "P1411".into(),
        firmware: "B-MD2FP1411V9.99-unknown".into(),
        role_code: "1".into(),
    }));
    app.refresh_candidates().unwrap();

    let snapshot = app.verify_selected_puppis().unwrap();

    assert_eq!(
        snapshot.protocol.summary,
        "Verified P1411 · firmware not qualified"
    );
    assert!(!snapshot.mutations_qualified);
    assert_eq!(
        snapshot.configuration.summary,
        "Read-only diagnostics available"
    );
}

#[test]
fn a_non_p1411_response_never_verifies_the_candidate() {
    let app = Application::new(selected_environment(DeviceIdentity {
        model: "Other".into(),
        firmware: QUALIFIED_FIRMWARE.into(),
        role_code: "1".into(),
    }));
    app.refresh_candidates().unwrap();

    let error = app.verify_selected_puppis().unwrap_err();

    assert_eq!(error.code, "identity_not_p1411");
    assert!(app.snapshot().verified_puppis.is_none());
}
