use puppis_core::{
    Application, AuthorizationState, HostSharingObservation, InMemoryEnvironment, ProfileKind,
    PuppisCandidate, UsbLinkSpeed,
};

fn app_with_host(observation: HostSharingObservation) -> (Application, InMemoryEnvironment) {
    let environment = InMemoryEnvironment::with_candidates(vec![PuppisCandidate {
        id: "candidate-1".into(),
        interface_name: "enx001".into(),
        display_name: "USB network adapter (enx001)".into(),
        link_speed: UsbLinkSpeed::SuperSpeed,
    }])
    .with_host_sharing(observation);
    let app = Application::new(environment.clone());
    app.refresh_candidates().unwrap();
    app.select_candidate("candidate-1").unwrap();
    (app, environment)
}

#[test]
fn working_external_sharing_is_reported_but_never_claimed() {
    let (app, environment) = app_with_host(HostSharingObservation {
        upstream_available: true,
        upstream_description: Some("Host default route".into()),
        profile_kind: ProfileKind::External,
        profile_name: Some("Wired connection 2".into()),
        active: true,
        ipv4_shared: true,
        downstream_address: Some("192.168.137.1/24".into()),
        ipv6_disabled: false,
        autoconnect: false,
        subnet_conflict: false,
        authorization: AuthorizationState::Prompt,
    });

    let snapshot = app.inspect_host_sharing().unwrap();

    assert_eq!(
        snapshot.sharing.summary,
        "Host sharing works through an external profile"
    );
    assert_eq!(
        snapshot.host_sharing.unwrap().profile_kind,
        ProfileKind::External
    );
    assert!(environment.recorded_mutations().is_empty());
}

#[test]
fn subnet_conflict_and_missing_upstream_are_independent_actionable_evidence() {
    let (app, _) = app_with_host(HostSharingObservation {
        upstream_available: false,
        upstream_description: None,
        profile_kind: ProfileKind::None,
        profile_name: None,
        active: false,
        ipv4_shared: false,
        downstream_address: None,
        ipv6_disabled: true,
        autoconnect: false,
        subnet_conflict: true,
        authorization: AuthorizationState::Denied,
    });

    let snapshot = app.inspect_host_sharing().unwrap();

    assert_eq!(snapshot.sharing.summary, "Puppis subnet conflict detected");
    let host = snapshot.host_sharing.unwrap();
    assert!(!host.upstream_available);
    assert!(host.subnet_conflict);
    assert_eq!(host.authorization, AuthorizationState::Denied);
    assert_eq!(snapshot.protocol.summary, "Candidate not yet verified");
}
