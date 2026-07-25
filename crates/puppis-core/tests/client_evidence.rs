use puppis_core::{
    Application, ClientEvidenceKind, DeviceIdentity, InMemoryEnvironment, PuppisCandidate,
    QUALIFIED_FIRMWARE, RadioBand, RadioConfiguration, UsbLinkSpeed,
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
        RadioConfiguration::fixture("five", "secret-five", "0", "CA", "160"),
    )
    .with_radio(
        RadioBand::TwoPointFourGhz,
        RadioConfiguration::fixture("two", "secret-two", "0", "CA", "40"),
    )
    .with_client_observations(
        vec!["aa:bb:cc:11:22:33".into()],
        vec!["aa:bb:cc:11:22:33".into(), "de:ad:be:ef:00:01".into()],
    );
    let app = Application::new(environment.clone());
    app.refresh_candidates().unwrap();
    app.verify_selected_puppis().unwrap();
    (app, environment)
}

#[test]
fn protocol_activity_is_connected_while_host_neighbors_are_only_recent() {
    let (app, _) = ready();
    let snapshot = app.refresh_client_evidence(true).unwrap();

    assert_eq!(snapshot.client_evidence.len(), 2);
    assert_eq!(
        snapshot.client_evidence[0].kind,
        ClientEvidenceKind::Connected
    );
    assert_eq!(
        snapshot.client_evidence[1].kind,
        ClientEvidenceKind::RecentlyObserved
    );
    assert_eq!(snapshot.client_evidence[0].alias, "client-1");
    assert_eq!(snapshot.client_evidence[1].alias, "client-2");
    let serialized = serde_json::to_string(&snapshot).unwrap();
    assert!(!serialized.contains("aa:bb:cc"));
    assert!(!serialized.contains("de:ad:be"));
}

#[test]
fn hidden_telemetry_does_not_poll_and_configuration_reads_remain_explicit() {
    let (app, environment) = ready();

    app.refresh_client_evidence(false).unwrap();
    assert_eq!(environment.telemetry_reads(), 0);
    app.read_radio_settings().unwrap();
    assert_eq!(environment.telemetry_reads(), 0);
    app.refresh_client_evidence(true).unwrap();
    assert_eq!(environment.telemetry_reads(), 1);
}

#[test]
fn slow_operational_refresh_never_invokes_configuration_or_client_telemetry_getters() {
    let (app, environment) = ready();

    app.refresh_operational_status().unwrap();

    assert_eq!(environment.configuration_reads(), 0);
    assert_eq!(environment.telemetry_reads(), 0);
}
