import { mount } from "svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import App from "./App.svelte";
import "./styles.css";
import type { ApplicationSnapshot, DiagnosticsBundle, RadioUpdate } from "./lib/contracts";

const initialSnapshot: ApplicationSnapshot = {
  revision: 1,
  observedStatusAt: null,
  candidates: [],
  selectedCandidateId: null,
  verifiedPuppis: null,
  mutationsQualified: false,
  hostSharing: null,
  radios: [],
  recoveryRequired: false,
  deviceRole: null,
  clientEvidence: [],
  savedClients: [],
  savedClientsAvailable: true,
  savedClientsFailure: null,
  usb: { level: "unavailable", summary: "No Puppis candidate detected", guidance: null },
  sharing: { level: "unavailable", summary: "Host sharing not inspected", guidance: null },
  protocol: { level: "unavailable", summary: "No verified Puppis", guidance: null },
  configuration: { level: "unavailable", summary: "Device configuration unavailable", guidance: null },
  clients: { level: "unavailable", summary: "Client evidence unavailable", guidance: null },
  activeOperation: null,
  lastFailure: null,
};

const target = document.getElementById("app")!;
mount(App, {
  target,
  props: {
    snapshot: initialSnapshot,
    loadSnapshot: () => invoke<ApplicationSnapshot>("refresh_candidates"),
    subscribeSnapshots: (handler: (next: ApplicationSnapshot) => void) =>
      listen<ApplicationSnapshot>("application-snapshot", (event) => handler(event.payload)),
    onSelectCandidate: (candidateId: string) => invoke<ApplicationSnapshot>("select_candidate", { candidateId }),
    onVerify: () => invoke<ApplicationSnapshot>("verify_selected_puppis"),
    onPreviewDiagnostics: () => invoke<DiagnosticsBundle>("preview_diagnostics"),
    onExportDiagnostics: (path: string, approvedPreview: string) => invoke<void>("export_diagnostics", { path, approvedPreview }),
    onInspectNetwork: () => invoke<ApplicationSnapshot>("inspect_host_sharing"),
    onEnableManagedSharing: () => invoke<ApplicationSnapshot>("enable_managed_sharing"),
    onDisableManagedSharing: () => invoke<ApplicationSnapshot>("disable_managed_sharing"),
    onRemoveManagedSharing: () => invoke<ApplicationSnapshot>("remove_managed_sharing"),
    onReadRadios: () => invoke<ApplicationSnapshot>("read_radio_settings"),
    onApplyRadio: (band, update: RadioUpdate) => invoke<ApplicationSnapshot>("apply_radio_settings", { band, update }),
    onSetDeviceRole: (role, confirmed) => invoke<ApplicationSnapshot>("set_device_role", { role, confirmed }),
    onRefreshClientEvidence: (telemetryVisible) => invoke<ApplicationSnapshot>("refresh_client_evidence", { telemetryVisible }),
    onRefreshStatus: () => invoke<ApplicationSnapshot>("refresh_operational_status"),
    onSaveClientLabel: (hardwareAddress, label) => invoke<ApplicationSnapshot>("save_client_label", { hardwareAddress, label }),
    onRenameSavedClient: (hardwareAddress, label) => invoke<ApplicationSnapshot>("rename_saved_client", { hardwareAddress, label }),
    onForgetSavedClient: (hardwareAddress) => invoke<ApplicationSnapshot>("forget_saved_client", { hardwareAddress }),
    onClearSavedClients: (confirmed) => invoke<ApplicationSnapshot>("clear_saved_clients", { confirmed }),
    onResetSavedClients: (confirmed) => invoke<ApplicationSnapshot>("reset_saved_clients", { confirmed }),
    onReassociateSavedClient: (previousHardwareAddress, newHardwareAddress, confirmed) => invoke<ApplicationSnapshot>("reassociate_saved_client", { previousHardwareAddress, newHardwareAddress, confirmed }),
    onAcceptRecoveryBaseline: () => invoke<ApplicationSnapshot>("accept_current_configuration_as_baseline"),
  },
});
