use puppis_core::{
    Application, DeviceIdentity, DeviceRole, InMemoryEnvironment, PuppisCandidate,
    QUALIFIED_FIRMWARE, RoleWriteEffect, UsbLinkSpeed,
};

fn ready(effects: Vec<RoleWriteEffect>) -> (Application, InMemoryEnvironment) {
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
    .with_role_write_effects(effects);
    let app = Application::new(environment.clone());
    app.refresh_candidates().unwrap();
    app.verify_selected_puppis().unwrap();
    (app, environment)
}

#[test]
fn confirmed_supported_role_change_is_independently_verified() {
    let (app, environment) = ready(vec![RoleWriteEffect::ApplyRequested]);

    assert_eq!(
        app.set_device_role(DeviceRole::WifiHotspot, false)
            .unwrap_err()
            .code,
        "confirmation_required"
    );
    let snapshot = app.set_device_role(DeviceRole::WifiHotspot, true).unwrap();

    assert_eq!(snapshot.device_role, Some(DeviceRole::WifiHotspot));
    assert_eq!(environment.role(), DeviceRole::WifiHotspot);
    assert!(!snapshot.recovery_required);
}

#[test]
fn wifi_adapter_role_is_diagnostic_only_and_mismatch_rolls_back() {
    let (app, _) = ready(Vec::new());
    assert_eq!(
        app.set_device_role(DeviceRole::WifiAdapter, true)
            .unwrap_err()
            .code,
        "role_not_supported"
    );

    let (app, environment) = ready(vec![
        RoleWriteEffect::Apply(DeviceRole::WifiAdapter),
        RoleWriteEffect::ApplyRequested,
    ]);
    assert_eq!(
        app.set_device_role(DeviceRole::WifiHotspot, true)
            .unwrap_err()
            .code,
        "transaction_rolled_back"
    );
    assert_eq!(environment.role(), DeviceRole::PrismPulse);
}
