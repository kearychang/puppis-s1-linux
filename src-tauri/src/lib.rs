use puppis_core::{Application, ApplicationSnapshot, InMemoryEnvironment};

fn observed<T>(
    application: &Application,
    result: Result<T, puppis_core::OperationFailure>,
) -> Result<T, puppis_core::OperationFailure> {
    if let Err(failure) = &result {
        application.remember_failure(failure);
    }
    result
}

#[tauri::command]
fn get_snapshot(application: tauri::State<'_, Application>) -> ApplicationSnapshot {
    application.snapshot()
}

#[tauri::command]
fn refresh_candidates(
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.refresh_candidates())
}

#[tauri::command]
fn select_candidate(
    candidate_id: String,
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.select_candidate(&candidate_id))
}

#[tauri::command]
fn verify_selected_puppis(
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.verify_selected_puppis())
}

#[tauri::command]
fn preview_diagnostics(
    application: tauri::State<'_, Application>,
) -> Result<puppis_core::DiagnosticsBundle, puppis_core::OperationFailure> {
    observed(&application, application.preview_diagnostics())
}

#[tauri::command]
fn export_diagnostics(
    path: String,
    approved_preview: String,
    application: tauri::State<'_, Application>,
) -> Result<(), puppis_core::OperationFailure> {
    observed(
        &application,
        application.export_diagnostics(std::path::Path::new(&path), &approved_preview),
    )
}

#[tauri::command]
fn inspect_host_sharing(
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.inspect_host_sharing())
}

#[tauri::command]
fn enable_managed_sharing(
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.enable_managed_sharing())
}

#[tauri::command]
fn disable_managed_sharing(
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.disable_managed_sharing())
}

#[tauri::command]
fn remove_managed_sharing(
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.remove_managed_sharing())
}

#[tauri::command]
fn read_radio_settings(
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.read_radio_settings())
}

#[tauri::command]
fn apply_radio_settings(
    band: puppis_core::RadioBand,
    update: puppis_core::RadioUpdate,
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.apply_radio_settings(band, update))
}

#[tauri::command]
fn set_device_role(
    role: puppis_core::DeviceRole,
    confirmed: bool,
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.set_device_role(role, confirmed))
}

#[tauri::command]
fn refresh_client_evidence(
    telemetry_visible: bool,
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(
        &application,
        application.refresh_client_evidence(telemetry_visible),
    )
}

#[tauri::command]
fn refresh_operational_status(
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(&application, application.refresh_operational_status())
}

#[tauri::command]
fn accept_current_configuration_as_baseline(
    application: tauri::State<'_, Application>,
) -> Result<ApplicationSnapshot, puppis_core::OperationFailure> {
    observed(
        &application,
        application.accept_current_configuration_as_baseline(),
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Application::new(InMemoryEnvironment::linux_system()))
        .setup(|app| {
            use tauri::{Emitter, Manager};
            let handle = app.handle().clone();
            app.state::<Application>()
                .start_system_observers(std::sync::Arc::new(move |snapshot| {
                    let _ = handle.emit("application-snapshot", snapshot);
                }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            refresh_candidates,
            select_candidate,
            verify_selected_puppis,
            preview_diagnostics,
            export_diagnostics,
            inspect_host_sharing,
            enable_managed_sharing,
            disable_managed_sharing,
            remove_managed_sharing,
            read_radio_settings,
            apply_radio_settings,
            set_device_role,
            refresh_client_evidence,
            refresh_operational_status,
            accept_current_configuration_as_baseline
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                use tauri::Manager;
                if !window.state::<Application>().ordinary_close_allowed() {
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to run Puppis S1 Manager for Linux");
}
