use puppis_core::{
    Application, DeviceIdentity, InMemoryEnvironment, PuppisCandidate, QUALIFIED_FIRMWARE,
    RadioBand, RadioConfiguration, UsbLinkSpeed,
};

fn app(firmware: &str) -> Application {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_identity(DeviceIdentity {
        model: "P1411".into(),
        firmware: firmware.into(),
        role_code: "1".into(),
    })
    .with_radio(
        RadioBand::FiveGhz,
        RadioConfiguration::fixture("prismpulse", "private-5g", "36", "CA", "160"),
    )
    .with_radio(
        RadioBand::TwoPointFourGhz,
        RadioConfiguration::fixture("puppis24", "private-2g", "6", "CA", "40"),
    );
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.verify_selected_puppis().unwrap();
    app
}

#[test]
fn qualified_radios_publish_non_secret_state_and_independent_capabilities() {
    let snapshot = app(QUALIFIED_FIRMWARE).read_radio_settings().unwrap();

    let five = snapshot
        .radios
        .iter()
        .find(|radio| radio.band == RadioBand::FiveGhz)
        .unwrap();
    assert_eq!(five.ssid, "prismpulse");
    assert_eq!(five.qualified_channels, vec!["0", "36"]);
    assert!(five.ssid_mutation && five.password_mutation);
    let two = snapshot
        .radios
        .iter()
        .find(|radio| radio.band == RadioBand::TwoPointFourGhz)
        .unwrap();
    assert_eq!(two.qualified_channels, vec!["0", "6"]);
    assert!(two.ssid_mutation && !two.password_mutation);
    assert_eq!(
        two.password_unavailable_reason.as_deref(),
        Some("Password restoration has not completed reversible qualification.")
    );

    let serialized = serde_json::to_string(&snapshot).unwrap();
    assert!(!serialized.contains("private-5g"));
    assert!(!serialized.contains("private-2g"));
}

#[test]
fn unknown_firmware_reads_settings_but_exposes_no_mutations() {
    let snapshot = app("unknown-firmware").read_radio_settings().unwrap();

    assert!(snapshot.radios.iter().all(|radio| !radio.ssid_mutation
        && !radio.password_mutation
        && radio.qualified_channels.is_empty()));
    assert_eq!(
        snapshot.configuration.summary,
        "Radio settings available read-only"
    );
}

#[test]
fn partial_two_point_four_ghz_fixture_is_a_required_password_gate_regression() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../prototype/p1411_protocol/captures/partial-2g-roundtrip.json"
    ))
    .unwrap();
    assert_eq!(
        fixture["password_roundtrip"]["temporary_complete_object_verified"],
        true
    );
    assert_eq!(
        fixture["password_roundtrip"]["restoration_complete_object_verified"],
        false
    );
    assert!(
        fixture["qualification"]["password"]
            .as_str()
            .unwrap()
            .contains("not qualified")
    );

    let snapshot = app(QUALIFIED_FIRMWARE).read_radio_settings().unwrap();
    let two = snapshot
        .radios
        .iter()
        .find(|radio| radio.band == RadioBand::TwoPointFourGhz)
        .unwrap();
    assert!(!two.password_mutation);
}
