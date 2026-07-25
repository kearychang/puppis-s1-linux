use puppis_core::{
    Application, DeviceIdentity, InMemoryEnvironment, PuppisCandidate, QUALIFIED_FIRMWARE,
    RadioBand, RadioConfiguration, RadioWriteEffect, UsbLinkSpeed,
};

fn environment(effects: Vec<RadioWriteEffect>) -> InMemoryEnvironment {
    InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_identity(DeviceIdentity {
        model: "P1411".into(),
        firmware: QUALIFIED_FIRMWARE.into(),
        role_code: "1".into(),
    })
    .with_radio(
        RadioBand::FiveGhz,
        RadioConfiguration::fixture("original", "private-value", "0", "CA", "160"),
    )
    .with_radio(
        RadioBand::TwoPointFourGhz,
        RadioConfiguration::fixture("original24", "private-24", "0", "CA", "40"),
    )
    .with_radio_write_effects(effects)
}

fn ready_app(environment: InMemoryEnvironment) -> Application {
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.verify_selected_puppis().unwrap();
    app.read_radio_settings().unwrap();
    app
}

#[test]
fn five_ghz_ssid_succeeds_only_after_complete_readback_verification() {
    let environment = environment(vec![RadioWriteEffect::ApplyRequested]);
    let app = ready_app(environment.clone());

    let snapshot = app
        .set_radio_ssid(RadioBand::FiveGhz, "NewNetwork")
        .unwrap();

    assert_eq!(
        snapshot
            .radios
            .iter()
            .find(|radio| radio.band == RadioBand::FiveGhz)
            .unwrap()
            .ssid,
        "NewNetwork"
    );
    assert!(!snapshot.recovery_required);
    assert!(snapshot.active_operation.is_none());
    assert!(!environment.interrupted_marker());
    assert!(
        environment
            .recorded_mutations()
            .iter()
            .all(|entry| !entry.contains("private-value"))
    );
}

#[test]
fn mismatched_readback_restores_and_verifies_the_complete_original_object() {
    let mismatch = RadioConfiguration::fixture("unexpected", "private-value", "36", "CA", "160");
    let environment = environment(vec![
        RadioWriteEffect::Apply(mismatch),
        RadioWriteEffect::ApplyRequested,
    ]);
    let app = ready_app(environment.clone());

    let error = app
        .set_radio_ssid(RadioBand::FiveGhz, "NewNetwork")
        .unwrap_err();

    assert_eq!(error.code, "transaction_rolled_back");
    assert_eq!(environment.radio(RadioBand::FiveGhz).ssid, "original");
    assert!(!app.snapshot().recovery_required);
    assert!(!environment.interrupted_marker());
}

#[test]
fn unverifiable_rollback_enters_recovery_required_and_blocks_more_mutations() {
    let mismatch = RadioConfiguration::fixture("unexpected", "private-value", "36", "CA", "160");
    let environment = environment(vec![
        RadioWriteEffect::Apply(mismatch),
        RadioWriteEffect::Fail,
    ]);
    let app = ready_app(environment.clone());

    let error = app
        .set_radio_ssid(RadioBand::FiveGhz, "NewNetwork")
        .unwrap_err();

    assert_eq!(error.code, "recovery_required");
    assert!(app.snapshot().recovery_required);
    assert!(environment.interrupted_marker());
    assert_eq!(
        app.set_radio_ssid(RadioBand::FiveGhz, "AnotherName")
            .unwrap_err()
            .code,
        "recovery_required"
    );
    assert!(
        app.preview_diagnostics()
            .unwrap()
            .preview
            .contains("Recovery required")
    );

    let reconciled = app.accept_current_configuration_as_baseline().unwrap();
    assert!(!reconciled.recovery_required);
    assert!(!environment.interrupted_marker());
    assert_eq!(
        reconciled.configuration.summary,
        "Current device state accepted as the recovery baseline"
    );
}
