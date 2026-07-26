use puppis_core::{
    Application, ClientEvidenceKind, ClientNetworkObservation, DeviceIdentity, InMemoryEnvironment,
    PuppisCandidate, QUALIFIED_FIRMWARE, RadioBand, RadioConfiguration, UsbLinkSpeed,
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
    app.select_candidate("candidate-1").unwrap();
    app.verify_selected_puppis().unwrap();
    (app, environment)
}

#[test]
fn passive_observations_group_addresses_by_mac_without_inventing_a_name() {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_recent_client_observations(vec![
        ClientNetworkObservation::fixture("02:00:00:00:02:01", "192.168.137.20"),
        ClientNetworkObservation::fixture("02:00:00:00:02:01", "192.168.137.21"),
    ]);
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.select_candidate("candidate-1").unwrap();

    let snapshot = app.refresh_client_evidence(true).unwrap();

    assert_eq!(snapshot.client_evidence.len(), 1);
    assert_eq!(snapshot.client_evidence[0].display_name, "Unlabeled client");
    assert_eq!(
        snapshot.client_evidence[0].hardware_address,
        "02:00:00:00:02:01"
    );
    assert_eq!(
        snapshot.client_evidence[0].addresses,
        ["192.168.137.20", "192.168.137.21"]
    );
    assert_eq!(
        snapshot.client_evidence[0].kind,
        ClientEvidenceKind::RecentlyObserved
    );
}

#[test]
fn passive_observations_exclude_host_broadcast_and_every_puppis_address() {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_recent_client_observations(vec![
        ClientNetworkObservation::fixture("00:11:22:33:44:55", "192.168.137.1"),
        ClientNetworkObservation::fixture("00:11:22:33:44:66", "192.168.137.255"),
        ClientNetworkObservation::fixture("02:00:00:00:01:01", "192.168.137.254"),
        ClientNetworkObservation::fixture("02:00:00:00:01:01", "192.168.137.10"),
        ClientNetworkObservation::fixture("02:00:00:00:02:01", "192.168.137.20"),
    ]);
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.select_candidate("candidate-1").unwrap();

    let snapshot = app.refresh_client_evidence(true).unwrap();

    assert_eq!(snapshot.client_evidence.len(), 1);
    assert_eq!(
        snapshot.client_evidence[0].hardware_address,
        "02:00:00:00:02:01"
    );
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
    assert!(serialized.contains("aa:bb:cc"));
    assert!(serialized.contains("de:ad:be"));
    let diagnostics = app.preview_diagnostics().unwrap().preview;
    assert!(!diagnostics.contains("aa:bb:cc"));
    assert!(!diagnostics.contains("de:ad:be"));
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
