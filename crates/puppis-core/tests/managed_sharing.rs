use puppis_core::{
    Application, AuthorizationState, DeviceIdentity, HostSharingObservation, InMemoryEnvironment,
    ProfileKind, PuppisCandidate, QUALIFIED_FIRMWARE, UsbLinkSpeed,
};

fn environment(
    profile_kind: ProfileKind,
    authorization: AuthorizationState,
) -> InMemoryEnvironment {
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
    .with_host_sharing(HostSharingObservation {
        upstream_available: true,
        upstream_description: Some("Host effective default route".into()),
        profile_kind,
        profile_name: (profile_kind != ProfileKind::None).then(|| "Existing profile".into()),
        active: profile_kind == ProfileKind::External,
        ipv4_shared: profile_kind == ProfileKind::External,
        downstream_address: (profile_kind == ProfileKind::External)
            .then(|| "192.168.137.1/24".into()),
        ipv6_disabled: false,
        autoconnect: false,
        subnet_conflict: false,
        authorization,
    })
}

#[test]
fn bootstrap_verifies_identity_then_promotes_only_the_owned_profile() {
    let environment = environment(ProfileKind::External, AuthorizationState::Prompt);
    let app = Application::new(environment.clone());
    app.refresh_candidates().unwrap();
    app.inspect_host_sharing().unwrap();

    let snapshot = app.enable_managed_sharing().unwrap();

    assert_eq!(snapshot.verified_puppis.unwrap().model, "P1411");
    let sharing = snapshot.host_sharing.unwrap();
    assert_eq!(sharing.profile_kind, ProfileKind::Managed);
    assert!(sharing.active && sharing.autoconnect && sharing.ipv4_shared && sharing.ipv6_disabled);
    assert_eq!(
        sharing.downstream_address.as_deref(),
        Some("192.168.137.1/24")
    );
    assert_eq!(
        environment.recorded_mutations(),
        vec![
            "checkpoint:create:enx001",
            "managed:reconcile:enx001",
            "checkpoint:commit:enx001",
        ]
    );
}

#[test]
fn authorization_denial_rolls_back_without_claiming_success() {
    let environment = environment(ProfileKind::None, AuthorizationState::Denied);
    let app = Application::new(environment.clone());
    app.refresh_candidates().unwrap();
    app.inspect_host_sharing().unwrap();

    let error = app.enable_managed_sharing().unwrap_err();

    assert_eq!(error.code, "network_authorization_denied");
    assert_eq!(
        environment.recorded_mutations(),
        vec!["checkpoint:create:enx001", "checkpoint:rollback:enx001",]
    );
}

#[test]
fn disable_and_remove_are_distinct_and_reject_external_profiles() {
    let environment = environment(ProfileKind::Managed, AuthorizationState::Allowed);
    let app = Application::new(environment.clone());
    app.refresh_candidates().unwrap();
    app.inspect_host_sharing().unwrap();
    app.disable_managed_sharing().unwrap();
    assert_eq!(
        app.snapshot().host_sharing.unwrap().profile_kind,
        ProfileKind::Managed
    );

    app.remove_managed_sharing().unwrap();
    assert_eq!(
        app.snapshot().host_sharing.unwrap().profile_kind,
        ProfileKind::None
    );
    assert_eq!(
        environment.recorded_mutations(),
        vec!["managed:disable:enx001", "managed:remove:enx001"]
    );
}

#[test]
fn unreachable_identity_is_bootstrapped_only_inside_the_checkpoint() {
    let environment =
        environment(ProfileKind::None, AuthorizationState::Allowed).with_identity_failures(1);
    let app = Application::new(environment.clone());
    app.refresh_candidates().unwrap();
    app.inspect_host_sharing().unwrap();

    app.enable_managed_sharing().unwrap();

    assert_eq!(
        environment.recorded_mutations(),
        vec![
            "checkpoint:create:enx001",
            "bootstrap:temporary:enx001",
            "managed:reconcile:enx001",
            "bootstrap:remove:enx001",
            "checkpoint:commit:enx001",
        ]
    );
}
