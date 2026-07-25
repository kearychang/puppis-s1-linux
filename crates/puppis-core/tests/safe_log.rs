use puppis_core::log::{BoundedLog, SafeEvent};

#[test]
fn structured_log_is_bounded_and_has_no_free_form_secret_channel() {
    let path = std::env::temp_dir().join(format!("puppis-safe-log-{}.jsonl", std::process::id()));
    let log = BoundedLog::with_limit(path.clone(), 512);

    for _ in 0..100 {
        log.record(SafeEvent::SettingsTransactionVerified).unwrap();
    }

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.len() <= 512);
    assert!(
        contents
            .lines()
            .all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok())
    );
    assert!(!contents.to_ascii_lowercase().contains("password"));
    std::fs::remove_file(path).unwrap();
}
