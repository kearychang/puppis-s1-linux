use puppis_core::{
    Application, DeviceIdentity, InMemoryEnvironment, PuppisCandidate, QUALIFIED_FIRMWARE,
    RadioBand, RadioConfiguration, UsbLinkSpeed,
};

fn candidate(id: &str, interface: &str, speed: UsbLinkSpeed) -> PuppisCandidate {
    PuppisCandidate {
        id: id.into(),
        interface_name: interface.into(),
        display_name: format!("USB network adapter ({interface})"),
        link_speed: speed,
    }
}

#[test]
fn one_candidate_still_requires_explicit_selection() {
    let environment = InMemoryEnvironment::with_candidates(vec![candidate(
        "usb-a",
        "enx001",
        UsbLinkSpeed::SuperSpeed,
    )]);
    let app = Application::new(environment);

    let discovered = app.refresh_candidates().unwrap();

    assert_eq!(discovered.candidates.len(), 1);
    assert!(discovered.selected_candidate_id.is_none());
    assert_eq!(discovered.usb.summary, "Select a Puppis candidate");
}

#[test]
fn user_selects_one_ambiguous_candidate_without_claiming_it_is_a_puppis() {
    let environment = InMemoryEnvironment::with_candidates(vec![
        candidate("usb-a", "enx001", UsbLinkSpeed::SuperSpeed),
        candidate("usb-b", "enx002", UsbLinkSpeed::Usb2),
    ]);
    let app = Application::new(environment);

    let discovered = app.refresh_candidates().unwrap();
    assert_eq!(discovered.candidates.len(), 2);
    assert!(discovered.selected_candidate_id.is_none());
    assert_eq!(discovered.usb.summary, "Select a Puppis candidate");

    let selected = app.select_candidate("usb-b").unwrap();
    assert_eq!(selected.selected_candidate_id.as_deref(), Some("usb-b"));
    assert_eq!(selected.usb.summary, "USB 2 connection detected");
    assert_eq!(selected.protocol.summary, "Candidate not yet verified");
    assert_eq!(
        selected.usb.guidance.as_deref(),
        Some("For best streaming performance, reconnect the Puppis through a SuperSpeed USB port.")
    );
}

#[test]
fn a_disappearing_candidate_clears_stale_selection() {
    let environment = InMemoryEnvironment::with_candidates(vec![candidate(
        "usb-a",
        "enx001",
        UsbLinkSpeed::SuperSpeed,
    )]);
    let app = Application::new(environment.clone());
    app.refresh_candidates().unwrap();
    app.select_candidate("usb-a").unwrap();

    environment.replace_candidates(Vec::new());
    let snapshot = app.refresh_candidates().unwrap();

    assert!(snapshot.selected_candidate_id.is_none());
    assert_eq!(snapshot.usb.summary, "No Puppis candidate detected");
    assert_eq!(snapshot.protocol.summary, "No verified Puppis");
}

#[test]
fn switching_candidates_discards_the_previous_device_session_state() {
    let environment = InMemoryEnvironment::with_candidates(vec![
        candidate("usb-a", "enx001", UsbLinkSpeed::SuperSpeed),
        candidate("usb-b", "enx002", UsbLinkSpeed::SuperSpeed),
    ])
    .with_identity(DeviceIdentity {
        model: "P1411".into(),
        firmware: QUALIFIED_FIRMWARE.into(),
        role_code: "1".into(),
    })
    .with_radio(
        RadioBand::FiveGhz,
        RadioConfiguration::fixture("five", "private", "36", "CA", "160"),
    )
    .with_radio(
        RadioBand::TwoPointFourGhz,
        RadioConfiguration::fixture("two", "private", "6", "CA", "40"),
    );
    let app = Application::new(environment);
    app.refresh_candidates().unwrap();
    app.select_candidate("usb-a").unwrap();
    app.verify_selected_puppis().unwrap();
    app.read_radio_settings().unwrap();

    let switched = app.select_candidate("usb-b").unwrap();

    assert!(switched.verified_puppis.is_none());
    assert!(switched.radios.is_empty());
    assert!(switched.device_role.is_none());
    assert!(switched.client_evidence.is_empty());
    assert!(switched.host_sharing.is_none());
}
