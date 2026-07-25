use puppis_core::{Application, ApplicationSnapshot, InMemoryEnvironment};

#[test]
fn launch_without_a_device_is_safe_and_explains_each_status_dimension() {
    let app = Application::new(InMemoryEnvironment::default());

    let snapshot: ApplicationSnapshot = app.snapshot();

    assert_eq!(snapshot.revision, 1);
    assert_eq!(snapshot.usb.summary, "No Puppis candidate detected");
    assert_eq!(snapshot.sharing.summary, "Host sharing not inspected");
    assert_eq!(snapshot.protocol.summary, "No verified Puppis");
    assert_eq!(
        snapshot.configuration.summary,
        "Device configuration unavailable"
    );
    assert_eq!(snapshot.clients.summary, "Client evidence unavailable");
    assert!(snapshot.active_operation.is_none());
}

#[test]
fn failures_crossing_the_application_seam_are_stable_and_safe() {
    let failure = Application::new(InMemoryEnvironment::default())
        .select_candidate("missing")
        .expect_err("an unknown candidate must be rejected");

    assert_eq!(failure.code, "candidate_not_found");
    assert_eq!(
        failure.message,
        "That Puppis candidate is no longer available."
    );
    assert_eq!(
        failure.guidance,
        "Refresh device discovery and select an available candidate."
    );
    assert!(failure.diagnostic_reference.is_none());
}
