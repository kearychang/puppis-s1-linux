use puppis_core::{
    Application, ClientNetworkObservation, InMemoryEnvironment, PuppisCandidate, UsbLinkSpeed,
};

fn observed_app() -> Application {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_current_time(1_721_920_000)
    .with_recent_client_observations(vec![ClientNetworkObservation::fixture(
        "02:00:00:00:02:01",
        "192.168.137.20",
    )]);
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.select_candidate("candidate-1").unwrap();
    app.refresh_client_evidence(true).unwrap();
    app
}

#[test]
fn saving_an_observed_client_reuses_its_label_and_latest_observation() {
    let app = observed_app();

    let snapshot = app
        .save_client_label("02:00:00:00:02:01", "  Living Room Headset  ")
        .unwrap();

    assert_eq!(
        snapshot.client_evidence[0].display_name,
        "Living Room Headset"
    );
    assert!(snapshot.client_evidence[0].saved);
    assert_eq!(snapshot.saved_clients.len(), 1);
    assert_eq!(snapshot.saved_clients[0].label, "Living Room Headset");
    assert_eq!(snapshot.saved_clients[0].last_observed_at, 1_721_920_000);
    assert_eq!(snapshot.saved_clients[0].addresses, ["192.168.137.20"]);
}

#[test]
fn reassociation_requires_confirmation_and_moves_the_saved_label_to_an_observed_mac() {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_current_time(1_721_920_000)
    .with_recent_client_observations(vec![
        ClientNetworkObservation::fixture("02:00:00:00:02:01", "192.168.137.20"),
        ClientNetworkObservation::fixture("02:00:00:00:02:02", "192.168.137.21"),
    ]);
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.select_candidate("candidate-1").unwrap();
    app.refresh_client_evidence(true).unwrap();
    app.save_client_label("02:00:00:00:02:01", "Living Room Headset")
        .unwrap();
    let environment_snapshot = app.snapshot();
    assert_eq!(environment_snapshot.saved_clients.len(), 1);

    let failure = app
        .reassociate_saved_client("02:00:00:00:02:01", "02:00:00:00:02:02", false)
        .unwrap_err();

    assert_eq!(failure.code, "confirmation_required");

    let snapshot = app
        .reassociate_saved_client("02:00:00:00:02:01", "02:00:00:00:02:02", true)
        .unwrap();
    assert_eq!(
        snapshot.saved_clients[0].hardware_address,
        "02:00:00:00:02:02"
    );
    assert_eq!(
        snapshot
            .client_evidence
            .iter()
            .find(|client| client.hardware_address == "02:00:00:00:02:02")
            .unwrap()
            .display_name,
        "Living Room Headset"
    );
}

#[test]
fn labels_are_printable_unique_and_limited_to_ten_saved_clients() {
    let observations: Vec<_> = (0..11)
        .map(|index| {
            ClientNetworkObservation::fixture(
                &format!("02:00:00:00:00:{index:02x}"),
                &format!("192.168.137.{}", index + 10),
            )
        })
        .collect();
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_recent_client_observations(observations);
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.select_candidate("candidate-1").unwrap();
    app.refresh_client_evidence(true).unwrap();
    assert_eq!(
        app.save_client_label("02:00:00:00:00:00", "bad\nlabel")
            .unwrap_err()
            .code,
        "invalid_client_label"
    );
    assert_eq!(
        app.save_client_label("02:00:00:00:00:00", "hidden\u{202e}text")
            .unwrap_err()
            .code,
        "invalid_client_label"
    );
    app.save_client_label("02:00:00:00:00:00", "Straße")
        .unwrap();
    assert_eq!(
        app.save_client_label("02:00:00:00:00:01", "STRASSE")
            .unwrap_err()
            .code,
        "client_label_not_unique"
    );
    app.save_client_label("02:00:00:00:00:01", "Client 1")
        .unwrap();
    assert_eq!(
        app.rename_saved_client("02:00:00:00:00:01", "STRASSE")
            .unwrap_err()
            .code,
        "client_label_not_unique"
    );
    assert_eq!(
        app.rename_saved_client("02:00:00:00:00:01", "hidden\u{200b}text")
            .unwrap_err()
            .code,
        "invalid_client_label"
    );
    for index in 1..10 {
        app.save_client_label(
            &format!("02:00:00:00:00:{index:02x}"),
            &format!("Client {index}"),
        )
        .unwrap();
    }
    assert_eq!(
        app.save_client_label("02:00:00:00:00:0a", "Client 10")
            .unwrap_err()
            .code,
        "saved_client_limit_reached"
    );
}

#[test]
fn a_runtime_storage_failure_disables_more_saved_client_mutations() {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_recent_client_observations(vec![ClientNetworkObservation::fixture(
        "02:00:00:00:00:01",
        "192.168.137.10",
    )])
    .with_saved_clients_path(std::path::PathBuf::from(
        "/proc/puppis-s1-manager/saved-clients.json",
    ));
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.select_candidate("candidate-1").unwrap();
    app.refresh_client_evidence(true).unwrap();

    assert_eq!(
        app.save_client_label("02:00:00:00:00:01", "Headset")
            .unwrap_err()
            .code,
        "saved_clients_storage_failed"
    );
    assert!(!app.snapshot().saved_clients_available);
    assert_eq!(
        app.save_client_label("02:00:00:00:00:01", "Headset")
            .unwrap_err()
            .code,
        "saved_clients_storage_failed"
    );
}

#[test]
fn forgetting_removes_saved_metadata_but_keeps_current_evidence_unlabeled() {
    let app = observed_app();
    app.save_client_label("02:00:00:00:02:01", "Living Room Headset")
        .unwrap();

    let snapshot = app.forget_saved_client("02:00:00:00:02:01").unwrap();

    assert!(snapshot.saved_clients.is_empty());
    assert_eq!(snapshot.client_evidence[0].display_name, "Unlabeled client");
    assert!(!snapshot.client_evidence[0].saved);
}

#[test]
fn clearing_all_saved_clients_requires_explicit_confirmation() {
    let app = observed_app();
    app.save_client_label("02:00:00:00:02:01", "Living Room Headset")
        .unwrap();

    assert_eq!(
        app.clear_saved_clients(false).unwrap_err().code,
        "confirmation_required"
    );
    assert!(
        app.clear_saved_clients(true)
            .unwrap()
            .saved_clients
            .is_empty()
    );
}

#[test]
fn corrupt_storage_disables_mutations_until_a_confirmed_reset() {
    let path = std::env::temp_dir().join(format!(
        "puppis-corrupt-saved-clients-{}.json",
        std::process::id()
    ));
    std::fs::write(&path, "broken").unwrap();
    let environment =
        InMemoryEnvironment::with_candidates(Vec::new()).with_saved_clients_path(path.clone());
    let app = Application::new(environment);

    assert!(!app.snapshot().saved_clients_available);
    assert_eq!(
        app.reset_saved_clients(false).unwrap_err().code,
        "confirmation_required"
    );
    let snapshot = app.reset_saved_clients(true).unwrap();
    assert!(snapshot.saved_clients_available);
    assert!(snapshot.saved_clients_failure.is_none());
    assert_eq!(puppis_core::client_state::load(&path).unwrap(), Vec::new());
    let _ = std::fs::remove_file(path);
}

#[test]
fn observed_saved_clients_sort_before_unlabeled_clients() {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_recent_client_observations(vec![
        ClientNetworkObservation::fixture("00:00:00:00:00:01", "192.168.137.10"),
        ClientNetworkObservation::fixture("ff:ff:ff:00:00:01", "192.168.137.20"),
    ]);
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.select_candidate("candidate-1").unwrap();
    app.refresh_client_evidence(true).unwrap();

    let snapshot = app
        .save_client_label("ff:ff:ff:00:00:01", "Headset")
        .unwrap();

    assert_eq!(snapshot.client_evidence[0].display_name, "Headset");
    assert_eq!(snapshot.client_evidence[1].display_name, "Unlabeled client");
}
