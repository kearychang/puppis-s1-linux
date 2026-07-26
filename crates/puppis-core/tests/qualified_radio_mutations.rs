use puppis_core::{
    Application, DeviceIdentity, InMemoryEnvironment, PuppisCandidate, QUALIFIED_FIRMWARE,
    RadioBand, RadioConfiguration, RadioUpdate, UsbLinkSpeed,
};

fn ready() -> (Application, InMemoryEnvironment) {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
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
        RadioConfiguration::fixture("five", "original-secret", "0", "CA", "160"),
    )
    .with_radio(
        RadioBand::TwoPointFourGhz,
        RadioConfiguration::fixture("two", "original-24", "0", "CA", "40"),
    );
    let app = Application::new(environment.clone());
    app.refresh_candidates().unwrap();
    app.select_candidate("candidate-1").unwrap();
    app.verify_selected_puppis().unwrap();
    app.read_radio_settings().unwrap();
    (app, environment)
}

#[test]
fn five_ghz_channel_and_write_only_password_use_the_verified_transaction() {
    let (app, environment) = ready();
    assert_eq!(
        app.set_radio_channel(RadioBand::FiveGhz, "40")
            .unwrap_err()
            .code,
        "channel_not_qualified"
    );

    let channel = app.set_radio_channel(RadioBand::FiveGhz, "36").unwrap();
    assert_eq!(
        channel
            .radios
            .iter()
            .find(|radio| radio.band == RadioBand::FiveGhz)
            .unwrap()
            .channel,
        "36"
    );
    let password = app
        .set_radio_password(RadioBand::FiveGhz, "NewSecret9!")
        .unwrap();

    assert_eq!(
        environment.radio(RadioBand::FiveGhz).password,
        "NewSecret9!"
    );
    assert!(
        !serde_json::to_string(&password)
            .unwrap()
            .contains("NewSecret9!")
    );
    assert!(
        environment
            .recorded_mutations()
            .iter()
            .all(|entry| !entry.contains("NewSecret9!"))
    );
}

#[test]
fn two_point_four_ghz_exposes_only_qualified_ssid_and_canadian_channel_paths() {
    let (app, environment) = ready();

    app.set_radio_ssid(RadioBand::TwoPointFourGhz, "TwoNew")
        .unwrap();
    app.set_radio_channel(RadioBand::TwoPointFourGhz, "6")
        .unwrap();

    assert_eq!(environment.radio(RadioBand::TwoPointFourGhz).ssid, "TwoNew");
    assert_eq!(environment.radio(RadioBand::TwoPointFourGhz).channel, "6");
    assert_eq!(
        app.set_radio_channel(RadioBand::TwoPointFourGhz, "11")
            .unwrap_err()
            .code,
        "channel_not_qualified"
    );
    assert_eq!(
        app.set_radio_password(RadioBand::TwoPointFourGhz, "Another9!")
            .unwrap_err()
            .code,
        "password_mutation_unavailable"
    );
}

#[test]
fn qualified_channels_are_bound_to_the_reported_country_for_both_radios() {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
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
        RadioConfiguration::fixture("five", "original-secret", "0", "US", "160"),
    )
    .with_radio(
        RadioBand::TwoPointFourGhz,
        RadioConfiguration::fixture("two", "original-24", "0", "US", "40"),
    );
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.select_candidate("candidate-1").unwrap();
    app.verify_selected_puppis().unwrap();

    let snapshot = app.read_radio_settings().unwrap();

    assert!(
        snapshot
            .radios
            .iter()
            .all(|radio| radio.qualified_channels.is_empty())
    );
    assert!(
        snapshot
            .radios
            .iter()
            .find(|radio| radio.band == RadioBand::FiveGhz)
            .unwrap()
            .password_mutation
    );
    assert!(
        !snapshot
            .radios
            .iter()
            .find(|radio| radio.band == RadioBand::TwoPointFourGhz)
            .unwrap()
            .password_mutation
    );
}

#[test]
fn one_radio_apply_changes_all_requested_exposed_fields_in_one_transaction() {
    let (app, environment) = ready();

    let snapshot = app
        .apply_radio_settings(
            RadioBand::FiveGhz,
            RadioUpdate {
                ssid: Some("FiveNew".into()),
                channel: Some("36".into()),
                password: Some("NewSecret9!".into()),
            },
        )
        .unwrap();

    let radio = environment.radio(RadioBand::FiveGhz);
    assert_eq!(radio.ssid, "FiveNew");
    assert_eq!(radio.channel, "36");
    assert_eq!(radio.password, "NewSecret9!");
    assert_eq!(
        environment
            .recorded_mutations()
            .iter()
            .filter(|entry| entry.starts_with("radio:set:"))
            .count(),
        1
    );
    assert!(
        !serde_json::to_string(&snapshot)
            .unwrap()
            .contains("NewSecret9!")
    );
}
