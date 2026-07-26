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
