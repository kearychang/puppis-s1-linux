use puppis_core::{SavedClient, client_state};
use std::os::unix::fs::PermissionsExt;

fn temporary_state_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "puppis-saved-clients-{}-{name}.json",
        std::process::id()
    ))
}

#[test]
fn saved_client_file_is_versioned_private_and_round_trips() {
    let path = temporary_state_path("round-trip");
    let clients = vec![SavedClient {
        label: "Living Room Headset".into(),
        hardware_address: "02:00:00:00:02:01".into(),
        last_observed_at: 1_721_920_000,
        addresses: vec!["192.168.137.20".into()],
    }];

    client_state::save(&path, &clients).unwrap();

    assert_eq!(client_state::load(&path).unwrap(), clients);
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let serialized = std::fs::read_to_string(&path).unwrap();
    assert!(serialized.contains("\"schemaVersion\": 1"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn corrupt_or_future_state_is_reported_without_being_overwritten() {
    let path = temporary_state_path("corrupt");
    std::fs::write(&path, "{ not-json").unwrap();
    assert_eq!(
        client_state::load(&path).unwrap_err().code,
        "saved_clients_corrupt"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ not-json");

    std::fs::write(&path, r#"{"schemaVersion": 99, "clients": []}"#).unwrap();
    assert_eq!(
        client_state::load(&path).unwrap_err().code,
        "saved_clients_version_unsupported"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn semantically_invalid_v1_state_is_corrupt_and_left_untouched() {
    let path = temporary_state_path("invalid-v1");
    let invalid = r#"{
  "schemaVersion": 1,
  "clients": [
    {"label":"Headset","hardwareAddress":"not-a-mac","lastObservedAt":1,"addresses":[]},
    {"label":"HEADSET","hardwareAddress":"02:00:00:00:00:01","lastObservedAt":2,"addresses":[]}
  ]
}"#;
    std::fs::write(&path, invalid).unwrap();

    assert_eq!(
        client_state::load(&path).unwrap_err().code,
        "saved_clients_corrupt"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), invalid);
    let _ = std::fs::remove_file(path);
}

#[test]
fn loading_repairs_a_broadly_readable_state_file_to_user_only_permissions() {
    let path = temporary_state_path("permissions");
    client_state::save(&path, &[]).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    assert!(client_state::load(&path).unwrap().is_empty());
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn state_with_more_than_ten_clients_is_rejected() {
    let path = temporary_state_path("too-many");
    let clients: Vec<_> = (0..11)
        .map(|index| {
            serde_json::json!({
                "label": format!("Client {index}"),
                "hardwareAddress": format!("02:00:00:00:00:{index:02x}"),
                "lastObservedAt": index,
                "addresses": [],
            })
        })
        .collect();
    let state = serde_json::to_string(&serde_json::json!({
        "schemaVersion": 1,
        "clients": clients,
    }))
    .unwrap();
    std::fs::write(&path, state).unwrap();

    assert_eq!(
        client_state::load(&path).unwrap_err().code,
        "saved_clients_corrupt"
    );
    let _ = std::fs::remove_file(path);
}
