//! UI-independent application core for Puppis S1 Manager for Linux.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

pub mod client_state;
pub mod diagnostics;
pub mod log;
pub mod network;
mod protocol;
pub mod usb;
pub use diagnostics::DiagnosticsBundle;

pub const QUALIFIED_FIRMWARE: &str = "B-MD2FP1411V1.22-250108-r0e17";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusDimension {
    pub level: StatusLevel,
    pub summary: String,
    pub guidance: Option<String>,
}

impl StatusDimension {
    fn unavailable(summary: &str) -> Self {
        Self {
            level: StatusLevel::Unavailable,
            summary: summary.into(),
            guidance: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusLevel {
    Healthy,
    Attention,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsbLinkSpeed {
    SuperSpeed,
    Usb2,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PuppisCandidate {
    pub id: String,
    pub interface_name: String,
    pub display_name: String,
    pub link_speed: UsbLinkSpeed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceIdentity {
    pub model: String,
    pub firmware: String,
    pub role_code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceRole {
    PrismPulse,
    WifiHotspot,
    WifiAdapter,
}

impl DeviceRole {
    fn from_code(code: &str) -> Option<Self> {
        match code {
            "1" => Some(Self::PrismPulse),
            "2" => Some(Self::WifiHotspot),
            "3" => Some(Self::WifiAdapter),
            _ => None,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::PrismPulse => "1",
            Self::WifiHotspot => "2",
            Self::WifiAdapter => "3",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleWriteEffect {
    ApplyRequested,
    Apply(DeviceRole),
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientEvidenceKind {
    Connected,
    RecentlyObserved,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientNetworkObservation {
    pub hardware_address: String,
    pub address: Option<String>,
}

impl ClientNetworkObservation {
    pub fn fixture(hardware_address: &str, address: &str) -> Self {
        Self {
            hardware_address: hardware_address.to_ascii_lowercase(),
            address: Some(address.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientEvidence {
    pub alias: String,
    pub display_name: String,
    pub hardware_address: String,
    pub addresses: Vec<String>,
    pub kind: ClientEvidenceKind,
    pub saved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedClient {
    pub label: String,
    pub hardware_address: String,
    pub last_observed_at: u64,
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
    None,
    Managed,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationState {
    Allowed,
    Prompt,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostSharingObservation {
    pub upstream_available: bool,
    pub upstream_description: Option<String>,
    pub profile_kind: ProfileKind,
    pub profile_name: Option<String>,
    pub active: bool,
    pub ipv4_shared: bool,
    pub downstream_address: Option<String>,
    pub ipv6_disabled: bool,
    pub autoconnect: bool,
    pub subnet_conflict: bool,
    pub authorization: AuthorizationState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadioBand {
    FiveGhz,
    TwoPointFourGhz,
}

#[derive(Clone, PartialEq, Eq)]
pub struct RadioConfiguration {
    pub ssid: String,
    pub password: String,
    pub protection: String,
    pub channel: String,
    pub country: String,
    pub enabled: String,
    pub encryption: String,
    pub bandwidth: String,
}

impl std::fmt::Debug for RadioConfiguration {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RadioConfiguration")
            .field("ssid", &self.ssid)
            .field("password", &"[REDACTED]")
            .field("channel", &self.channel)
            .field("country", &self.country)
            .field("bandwidth", &self.bandwidth)
            .finish_non_exhaustive()
    }
}

impl RadioConfiguration {
    pub fn fixture(
        ssid: &str,
        password: &str,
        channel: &str,
        country: &str,
        bandwidth: &str,
    ) -> Self {
        Self {
            ssid: ssid.into(),
            password: password.into(),
            protection: "0".into(),
            channel: channel.into(),
            country: country.into(),
            enabled: "0".into(),
            encryption: "5".into(),
            bandwidth: bandwidth.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadioSnapshot {
    pub band: RadioBand,
    pub ssid: String,
    pub channel: String,
    pub country: String,
    pub enabled: bool,
    pub bandwidth: String,
    pub ssid_mutation: bool,
    pub password_mutation: bool,
    pub qualified_channels: Vec<String>,
    pub password_unavailable_reason: Option<String>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadioUpdate {
    pub ssid: Option<String>,
    pub channel: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadioWriteEffect {
    ApplyRequested,
    Apply(RadioConfiguration),
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSnapshot {
    pub revision: u64,
    pub observed_status_at: Option<u64>,
    pub candidates: Vec<PuppisCandidate>,
    pub selected_candidate_id: Option<String>,
    pub verified_puppis: Option<DeviceIdentity>,
    pub mutations_qualified: bool,
    pub host_sharing: Option<HostSharingObservation>,
    pub radios: Vec<RadioSnapshot>,
    pub recovery_required: bool,
    pub device_role: Option<DeviceRole>,
    pub client_evidence: Vec<ClientEvidence>,
    pub saved_clients: Vec<SavedClient>,
    pub saved_clients_available: bool,
    pub saved_clients_failure: Option<OperationFailure>,
    pub usb: StatusDimension,
    pub sharing: StatusDimension,
    pub protocol: StatusDimension,
    pub configuration: StatusDimension,
    pub clients: StatusDimension,
    pub active_operation: Option<String>,
    pub last_failure: Option<OperationFailure>,
}

impl Default for ApplicationSnapshot {
    fn default() -> Self {
        Self {
            revision: 1,
            observed_status_at: None,
            candidates: Vec::new(),
            selected_candidate_id: None,
            verified_puppis: None,
            mutations_qualified: false,
            host_sharing: None,
            radios: Vec::new(),
            recovery_required: false,
            device_role: None,
            client_evidence: Vec::new(),
            saved_clients: Vec::new(),
            saved_clients_available: true,
            saved_clients_failure: None,
            usb: StatusDimension::unavailable("No Puppis candidate detected"),
            sharing: StatusDimension::unavailable("Host sharing not inspected"),
            protocol: StatusDimension::unavailable("No verified Puppis"),
            configuration: StatusDimension::unavailable("Device configuration unavailable"),
            clients: StatusDimension::unavailable("Client evidence unavailable"),
            active_operation: None,
            last_failure: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationFailure {
    pub code: String,
    pub message: String,
    pub guidance: String,
    pub diagnostic_reference: Option<String>,
}

impl OperationFailure {
    pub fn safe(code: &str, message: &str, guidance: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            guidance: guidance.into(),
            diagnostic_reference: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryEnvironment {
    candidates: Arc<RwLock<Vec<PuppisCandidate>>>,
    discover_system_usb: bool,
    identity: Arc<RwLock<Option<DeviceIdentity>>>,
    use_system_protocol: bool,
    host_sharing: Arc<RwLock<Option<HostSharingObservation>>>,
    use_system_network: bool,
    mutations: Arc<RwLock<Vec<String>>>,
    radios: Arc<RwLock<HashMap<RadioBand, RadioConfiguration>>>,
    radio_write_effects: Arc<Mutex<VecDeque<RadioWriteEffect>>>,
    interrupted: Arc<RwLock<bool>>,
    role: Arc<RwLock<Option<DeviceRole>>>,
    role_write_effects: Arc<Mutex<VecDeque<RoleWriteEffect>>>,
    connected_clients: Arc<RwLock<Vec<String>>>,
    recent_clients: Arc<RwLock<Vec<ClientNetworkObservation>>>,
    telemetry_read_count: Arc<AtomicUsize>,
    configuration_read_count: Arc<AtomicUsize>,
    identity_failures_remaining: Arc<AtomicUsize>,
    bootstrap_active: Arc<RwLock<bool>>,
    current_time: Arc<AtomicU64>,
    saved_clients: Arc<RwLock<Vec<SavedClient>>>,
    saved_clients_path: Option<PathBuf>,
    saved_clients_failure: Arc<RwLock<Option<OperationFailure>>>,
}

impl InMemoryEnvironment {
    pub fn with_candidates(candidates: Vec<PuppisCandidate>) -> Self {
        Self {
            candidates: Arc::new(RwLock::new(candidates)),
            discover_system_usb: false,
            identity: Arc::new(RwLock::new(None)),
            use_system_protocol: false,
            host_sharing: Arc::new(RwLock::new(None)),
            use_system_network: false,
            mutations: Arc::new(RwLock::new(Vec::new())),
            radios: Arc::new(RwLock::new(HashMap::new())),
            radio_write_effects: Arc::new(Mutex::new(VecDeque::new())),
            interrupted: Arc::new(RwLock::new(false)),
            role: Arc::new(RwLock::new(None)),
            role_write_effects: Arc::new(Mutex::new(VecDeque::new())),
            connected_clients: Arc::new(RwLock::new(Vec::new())),
            recent_clients: Arc::new(RwLock::new(Vec::new())),
            telemetry_read_count: Arc::new(AtomicUsize::new(0)),
            configuration_read_count: Arc::new(AtomicUsize::new(0)),
            identity_failures_remaining: Arc::new(AtomicUsize::new(0)),
            bootstrap_active: Arc::new(RwLock::new(false)),
            current_time: Arc::new(AtomicU64::new(0)),
            saved_clients: Arc::new(RwLock::new(Vec::new())),
            saved_clients_path: None,
            saved_clients_failure: Arc::new(RwLock::new(None)),
        }
    }

    pub fn linux_system() -> Self {
        let saved_clients_path = client_state::system_path();
        let (saved_clients, saved_clients_failure) = match saved_clients_path.as_deref() {
            Some(path) => match client_state::load(path) {
                Ok(clients) => (clients, None),
                Err(failure) => (Vec::new(), Some(failure)),
            },
            None => (
                Vec::new(),
                Some(OperationFailure::safe(
                    "saved_clients_storage_failed",
                    "Saved client recognition data is unavailable.",
                    "Set HOME or XDG_STATE_HOME to a writable user state directory.",
                )),
            ),
        };
        Self {
            candidates: Arc::new(RwLock::new(Vec::new())),
            discover_system_usb: true,
            identity: Arc::new(RwLock::new(None)),
            use_system_protocol: true,
            host_sharing: Arc::new(RwLock::new(None)),
            use_system_network: true,
            mutations: Arc::new(RwLock::new(Vec::new())),
            radios: Arc::new(RwLock::new(HashMap::new())),
            radio_write_effects: Arc::new(Mutex::new(VecDeque::new())),
            interrupted: Arc::new(RwLock::new(system_marker_exists())),
            role: Arc::new(RwLock::new(None)),
            role_write_effects: Arc::new(Mutex::new(VecDeque::new())),
            connected_clients: Arc::new(RwLock::new(Vec::new())),
            recent_clients: Arc::new(RwLock::new(Vec::new())),
            telemetry_read_count: Arc::new(AtomicUsize::new(0)),
            configuration_read_count: Arc::new(AtomicUsize::new(0)),
            identity_failures_remaining: Arc::new(AtomicUsize::new(0)),
            bootstrap_active: Arc::new(RwLock::new(false)),
            current_time: Arc::new(AtomicU64::new(0)),
            saved_clients: Arc::new(RwLock::new(saved_clients)),
            saved_clients_path,
            saved_clients_failure: Arc::new(RwLock::new(saved_clients_failure)),
        }
    }

    pub fn with_identity(self, identity: DeviceIdentity) -> Self {
        *self.role.write().expect("role state poisoned") =
            DeviceRole::from_code(&identity.role_code);
        *self.identity.write().expect("identity state poisoned") = Some(identity);
        self
    }

    pub fn with_identity_failures(self, failures: usize) -> Self {
        self.identity_failures_remaining
            .store(failures, Ordering::Relaxed);
        self
    }

    pub fn with_role_write_effects(self, effects: Vec<RoleWriteEffect>) -> Self {
        *self
            .role_write_effects
            .lock()
            .expect("role script poisoned") = effects.into();
        self
    }

    pub fn role(&self) -> DeviceRole {
        self.read_role().expect("fake role configured")
    }

    pub fn with_client_observations(self, connected: Vec<String>, recent: Vec<String>) -> Self {
        *self
            .connected_clients
            .write()
            .expect("connected client state poisoned") = connected;
        *self
            .recent_clients
            .write()
            .expect("recent client state poisoned") = recent
            .into_iter()
            .map(|hardware_address| ClientNetworkObservation {
                hardware_address,
                address: None,
            })
            .collect();
        self
    }

    pub fn with_recent_client_observations(self, recent: Vec<ClientNetworkObservation>) -> Self {
        *self
            .recent_clients
            .write()
            .expect("recent client state poisoned") = recent;
        self
    }

    pub fn with_current_time(self, epoch_seconds: u64) -> Self {
        self.current_time.store(epoch_seconds, Ordering::Relaxed);
        self
    }

    pub fn with_saved_clients_path(mut self, path: PathBuf) -> Self {
        let (clients, failure) = match client_state::load(&path) {
            Ok(clients) => (clients, None),
            Err(failure) => (Vec::new(), Some(failure)),
        };
        *self
            .saved_clients
            .write()
            .expect("saved client state poisoned") = clients;
        *self
            .saved_clients_failure
            .write()
            .expect("saved client failure state poisoned") = failure;
        self.saved_clients_path = Some(path);
        self
    }

    fn current_time(&self) -> u64 {
        let fixed = self.current_time.load(Ordering::Relaxed);
        if fixed > 0 {
            fixed
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }
    }

    fn persist_saved_clients(&self, clients: &[SavedClient]) -> Result<(), OperationFailure> {
        if let Some(failure) = self
            .saved_clients_failure
            .read()
            .expect("saved client failure state poisoned")
            .clone()
        {
            return Err(failure);
        }
        if let Some(path) = &self.saved_clients_path {
            client_state::save(path, clients)?;
        }
        Ok(())
    }

    pub fn telemetry_reads(&self) -> usize {
        self.telemetry_read_count.load(Ordering::Relaxed)
    }

    pub fn configuration_reads(&self) -> usize {
        self.configuration_read_count.load(Ordering::Relaxed)
    }

    pub fn with_host_sharing(self, observation: HostSharingObservation) -> Self {
        *self
            .host_sharing
            .write()
            .expect("host sharing state poisoned") = Some(observation);
        self
    }

    pub fn with_radio(self, band: RadioBand, configuration: RadioConfiguration) -> Self {
        self.radios
            .write()
            .expect("radio state poisoned")
            .insert(band, configuration);
        self
    }

    pub fn with_radio_write_effects(self, effects: Vec<RadioWriteEffect>) -> Self {
        *self
            .radio_write_effects
            .lock()
            .expect("radio script poisoned") = effects.into();
        self
    }

    pub fn radio(&self, band: RadioBand) -> RadioConfiguration {
        self.radios
            .read()
            .expect("radio state poisoned")
            .get(&band)
            .expect("fake radio configured")
            .clone()
    }

    pub fn interrupted_marker(&self) -> bool {
        *self.interrupted.read().expect("interrupted state poisoned")
    }

    pub fn recorded_mutations(&self) -> Vec<String> {
        self.mutations
            .read()
            .expect("mutation history poisoned")
            .clone()
    }

    pub fn replace_candidates(&self, candidates: Vec<PuppisCandidate>) {
        *self.candidates.write().expect("candidate state poisoned") = candidates;
    }

    fn candidates(&self) -> Vec<PuppisCandidate> {
        if self.discover_system_usb {
            return usb::discover_linux_candidates().unwrap_or_default();
        }
        self.candidates
            .read()
            .expect("candidate state poisoned")
            .clone()
    }

    fn reset_protocol_session(&self) {
        if self.use_system_protocol {
            protocol::disconnect();
        }
    }

    fn identity(&self) -> Result<DeviceIdentity, OperationFailure> {
        if self.identity_failures_remaining.load(Ordering::Relaxed) > 0 {
            self.identity_failures_remaining
                .fetch_sub(1, Ordering::Relaxed);
            return Err(OperationFailure::safe(
                "protocol_unreachable",
                "The Puppis protocol is not reachable.",
                "Prepare temporary host addressing and try read-only identity again.",
            ));
        }
        if self.use_system_protocol {
            return protocol::read_device_identity();
        }
        self.identity
            .read()
            .expect("identity state poisoned")
            .clone()
            .ok_or_else(|| {
                OperationFailure::safe(
                    "protocol_unreachable",
                    "The selected candidate did not answer the Puppis protocol.",
                    "Check host addressing and try verification again.",
                )
            })
    }

    fn inspect_host_sharing(
        &self,
        interface_name: &str,
    ) -> Result<HostSharingObservation, OperationFailure> {
        if self.use_system_network {
            return network::inspect(interface_name);
        }
        self.host_sharing
            .read()
            .expect("host sharing state poisoned")
            .clone()
            .ok_or_else(|| {
                OperationFailure::safe(
                    "network_state_unavailable",
                    "Host sharing state is unavailable.",
                    "Inspect NetworkManager again.",
                )
            })
    }

    fn checkpoint(&self, interface_name: &str) -> Result<(), OperationFailure> {
        self.mutations
            .write()
            .expect("mutation history poisoned")
            .push(format!("checkpoint:create:{interface_name}"));
        if self.use_system_network {
            network::checkpoint(interface_name)
        } else {
            Ok(())
        }
    }

    fn finish_checkpoint(
        &self,
        interface_name: &str,
        commit: bool,
    ) -> Result<(), OperationFailure> {
        self.mutations
            .write()
            .expect("mutation history poisoned")
            .push(format!(
                "checkpoint:{}:{interface_name}",
                if commit { "commit" } else { "rollback" }
            ));
        if self.use_system_network {
            network::finish_checkpoint(commit)
        } else {
            Ok(())
        }
    }

    fn prepare_bootstrap(&self, interface_name: &str) -> Result<(), OperationFailure> {
        self.mutations
            .write()
            .expect("mutation history poisoned")
            .push(format!("bootstrap:temporary:{interface_name}"));
        let result = if self.use_system_network {
            network::prepare_bootstrap(interface_name)
        } else {
            Ok(())
        };
        if result.is_ok() {
            *self
                .bootstrap_active
                .write()
                .expect("bootstrap state poisoned") = true;
        }
        result
    }

    fn finish_bootstrap(&self, interface_name: &str, remove: bool) -> Result<(), OperationFailure> {
        let was_active = std::mem::take(
            &mut *self
                .bootstrap_active
                .write()
                .expect("bootstrap state poisoned"),
        );
        if remove && was_active {
            self.mutations
                .write()
                .expect("mutation history poisoned")
                .push(format!("bootstrap:remove:{interface_name}"));
        }
        if self.use_system_network {
            network::finish_bootstrap(remove)
        } else {
            Ok(())
        }
    }

    fn reconcile_managed(&self, interface_name: &str) -> Result<(), OperationFailure> {
        let authorization = self
            .host_sharing
            .read()
            .expect("host sharing state poisoned")
            .as_ref()
            .map(|state| state.authorization)
            .unwrap_or(AuthorizationState::Prompt);
        if authorization == AuthorizationState::Denied {
            return Err(OperationFailure::safe(
                "network_authorization_denied",
                "NetworkManager authorization was denied.",
                "Keep the current network state or try again when authorization is available.",
            ));
        }
        self.mutations
            .write()
            .expect("mutation history poisoned")
            .push(format!("managed:reconcile:{interface_name}"));
        if self.use_system_network {
            network::reconcile_managed(interface_name)
        } else {
            let mut state = self
                .host_sharing
                .write()
                .expect("host sharing state poisoned");
            let current = state.as_mut().expect("fake host state configured");
            current.profile_kind = ProfileKind::Managed;
            current.profile_name = Some("Puppis S1 Manager".into());
            current.active = true;
            current.ipv4_shared = true;
            current.downstream_address = Some("192.168.137.1/24".into());
            current.ipv6_disabled = true;
            current.autoconnect = true;
            Ok(())
        }
    }

    fn disable_managed(&self, interface_name: &str) -> Result<(), OperationFailure> {
        self.ensure_owned_profile()?;
        self.mutations
            .write()
            .expect("mutation history poisoned")
            .push(format!("managed:disable:{interface_name}"));
        if self.use_system_network {
            network::disable_managed(interface_name)
        } else {
            let mut state = self
                .host_sharing
                .write()
                .expect("host sharing state poisoned");
            let current = state.as_mut().expect("fake host state configured");
            current.active = false;
            current.autoconnect = false;
            Ok(())
        }
    }

    fn remove_managed(&self, interface_name: &str) -> Result<(), OperationFailure> {
        self.ensure_owned_profile()?;
        self.mutations
            .write()
            .expect("mutation history poisoned")
            .push(format!("managed:remove:{interface_name}"));
        if self.use_system_network {
            network::remove_managed(interface_name)
        } else {
            let mut state = self
                .host_sharing
                .write()
                .expect("host sharing state poisoned");
            let current = state.as_mut().expect("fake host state configured");
            current.profile_kind = ProfileKind::None;
            current.profile_name = None;
            current.active = false;
            current.ipv4_shared = false;
            current.downstream_address = None;
            current.autoconnect = false;
            Ok(())
        }
    }

    fn ensure_owned_profile(&self) -> Result<(), OperationFailure> {
        if !self.use_system_network
            && self
                .host_sharing
                .read()
                .expect("host sharing state poisoned")
                .as_ref()
                .is_some_and(|state| state.profile_kind != ProfileKind::Managed)
        {
            return Err(OperationFailure::safe(
                "external_profile_immutable",
                "The active sharing profile is not owned by this application.",
                "External profiles can be observed but never modified or removed.",
            ));
        }
        Ok(())
    }

    fn read_radio(&self, band: RadioBand) -> Result<RadioConfiguration, OperationFailure> {
        self.configuration_read_count
            .fetch_add(1, Ordering::Relaxed);
        if self.use_system_protocol {
            return protocol::read_radio(band);
        }
        self.radios
            .read()
            .expect("radio state poisoned")
            .get(&band)
            .cloned()
            .ok_or_else(|| {
                OperationFailure::safe(
                    "radio_unavailable",
                    "The requested radio settings are unavailable.",
                    "Verify the Puppis connection and request the settings again.",
                )
            })
    }

    fn write_radio(
        &self,
        band: RadioBand,
        requested: &RadioConfiguration,
    ) -> Result<(), OperationFailure> {
        self.mutations
            .write()
            .expect("mutation history poisoned")
            .push(format!("radio:set:{band:?}"));
        if self.use_system_protocol {
            return protocol::write_radio(band, requested);
        }
        let effect = self
            .radio_write_effects
            .lock()
            .expect("radio script poisoned")
            .pop_front()
            .unwrap_or(RadioWriteEffect::ApplyRequested);
        match effect {
            RadioWriteEffect::ApplyRequested => {
                self.radios
                    .write()
                    .expect("radio state poisoned")
                    .insert(band, requested.clone());
                Ok(())
            }
            RadioWriteEffect::Apply(configuration) => {
                self.radios
                    .write()
                    .expect("radio state poisoned")
                    .insert(band, configuration);
                Ok(())
            }
            RadioWriteEffect::Fail => Err(OperationFailure::safe(
                "device_mutation_failed",
                "The Puppis did not accept the requested settings transaction.",
                "The manager will reconcile observed state before allowing another change.",
            )),
        }
    }

    fn set_interrupted_marker(&self, present: bool) -> Result<(), OperationFailure> {
        *self
            .interrupted
            .write()
            .expect("interrupted state poisoned") = present;
        if !self.use_system_protocol {
            return Ok(());
        }
        write_system_marker(present)
    }

    fn read_role(&self) -> Result<DeviceRole, OperationFailure> {
        if self.use_system_protocol {
            let identity = protocol::read_device_identity()?;
            return DeviceRole::from_code(&identity.role_code).ok_or_else(role_unavailable);
        }
        self.role
            .read()
            .expect("role state poisoned")
            .ok_or_else(role_unavailable)
    }

    fn write_role(&self, requested: DeviceRole) -> Result<(), OperationFailure> {
        self.mutations
            .write()
            .expect("mutation history poisoned")
            .push("device:set-role".into());
        if self.use_system_protocol {
            return protocol::write_role(requested);
        }
        let effect = self
            .role_write_effects
            .lock()
            .expect("role script poisoned")
            .pop_front()
            .unwrap_or(RoleWriteEffect::ApplyRequested);
        match effect {
            RoleWriteEffect::ApplyRequested => {
                *self.role.write().expect("role state poisoned") = Some(requested);
                Ok(())
            }
            RoleWriteEffect::Apply(role) => {
                *self.role.write().expect("role state poisoned") = Some(role);
                Ok(())
            }
            RoleWriteEffect::Fail => Err(OperationFailure::safe(
                "device_mutation_failed",
                "The Puppis did not accept the role transition.",
                "The manager will reconcile observed state before another mutation.",
            )),
        }
    }

    fn read_client_evidence(
        &self,
        interface_name: &str,
    ) -> Result<(Vec<ClientNetworkObservation>, Vec<ClientNetworkObservation>), OperationFailure>
    {
        self.telemetry_read_count.fetch_add(1, Ordering::Relaxed);
        if self.use_system_protocol {
            // No validated P1411 station getter is currently qualified, so host evidence must never be promoted to connected.
            return Ok((Vec::new(), network::recent_clients(interface_name)));
        }
        let connected = self
            .connected_clients
            .read()
            .expect("connected client state poisoned")
            .iter()
            .cloned()
            .map(|hardware_address| ClientNetworkObservation {
                hardware_address,
                address: None,
            })
            .collect();
        let recent = self
            .recent_clients
            .read()
            .expect("recent client state poisoned")
            .clone();
        Ok((connected, recent))
    }
}

#[derive(Debug, Clone)]
pub struct Application {
    snapshot: Arc<RwLock<ApplicationSnapshot>>,
    environment: InMemoryEnvironment,
    mutation_lease: Arc<Mutex<()>>,
    log: Option<log::BoundedLog>,
}

struct ActiveOperationGuard {
    snapshot: Arc<RwLock<ApplicationSnapshot>>,
}

impl ActiveOperationGuard {
    fn new(snapshot: Arc<RwLock<ApplicationSnapshot>>, label: &str) -> Self {
        {
            let mut state = snapshot.write().expect("application state poisoned");
            state.revision += 1;
            state.active_operation = Some(label.into());
        }
        Self { snapshot }
    }
}

impl Drop for ActiveOperationGuard {
    fn drop(&mut self) {
        let mut state = self.snapshot.write().expect("application state poisoned");
        state.revision += 1;
        state.active_operation = None;
    }
}

impl Application {
    pub fn new(environment: InMemoryEnvironment) -> Self {
        let mut initial = ApplicationSnapshot::default();
        initial.saved_clients = environment
            .saved_clients
            .read()
            .expect("saved client state poisoned")
            .clone();
        initial.saved_clients_failure = environment
            .saved_clients_failure
            .read()
            .expect("saved client failure state poisoned")
            .clone();
        initial.saved_clients_available = initial.saved_clients_failure.is_none();
        if environment.interrupted_marker() {
            initial.recovery_required = true;
            initial.configuration = StatusDimension {
                level: StatusLevel::Attention,
                summary: "Recovery required".into(),
                guidance: Some(
                    "Reconcile the current device state before requesting another mutation.".into(),
                ),
            };
        }
        let log = environment
            .use_system_protocol
            .then(log::BoundedLog::system_default)
            .flatten();
        if let Some(log) = &log {
            let _ = log.record(log::SafeEvent::ApplicationStarted);
        }
        Self {
            snapshot: Arc::new(RwLock::new(initial)),
            environment,
            mutation_lease: Arc::new(Mutex::new(())),
            log,
        }
    }

    pub fn snapshot(&self) -> ApplicationSnapshot {
        self.snapshot
            .read()
            .expect("application state poisoned")
            .clone()
    }

    pub fn ordinary_close_allowed(&self) -> bool {
        self.snapshot
            .read()
            .expect("application state poisoned")
            .active_operation
            .is_none()
    }

    pub fn remember_failure(&self, failure: &OperationFailure) {
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.last_failure = Some(failure.clone());
    }

    pub fn refresh_operational_status(&self) -> Result<ApplicationSnapshot, OperationFailure> {
        let Ok(_lease) = self.mutation_lease.try_lock() else {
            return Ok(self.snapshot());
        };
        self.refresh_candidates()?;
        if self
            .snapshot
            .read()
            .expect("application state poisoned")
            .selected_candidate_id
            .is_none()
        {
            let mut snapshot = self.snapshot.write().expect("application state poisoned");
            snapshot.observed_status_at = Some(self.environment.current_time());
            return Ok(snapshot.clone());
        }
        if let Err(failure) = self.inspect_host_sharing() {
            self.remember_failure(&failure);
        }
        if let Err(failure) = self.verify_selected_puppis() {
            self.remember_failure(&failure);
        }
        self.snapshot
            .write()
            .expect("application state poisoned")
            .observed_status_at = Some(self.environment.current_time());
        Ok(self.snapshot())
    }

    pub fn start_system_observers(&self, publish: Arc<dyn Fn(ApplicationSnapshot) + Send + Sync>) {
        if !self.environment.discover_system_usb {
            return;
        }
        let application = self.clone();
        network::watch_changes(Arc::new(move || {
            if let Ok(snapshot) = application.refresh_operational_status() {
                publish(snapshot);
            }
        }));
    }

    pub fn refresh_candidates(&self) -> Result<ApplicationSnapshot, OperationFailure> {
        let candidates = self.environment.candidates();
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        let previous_selection = snapshot.selected_candidate_id.clone();
        snapshot.revision += 1;
        snapshot.candidates = candidates;
        if snapshot.selected_candidate_id.as_ref().is_some_and(|id| {
            !snapshot
                .candidates
                .iter()
                .any(|candidate| &candidate.id == id)
        }) {
            snapshot.selected_candidate_id = None;
            snapshot.verified_puppis = None;
            snapshot.mutations_qualified = false;
        }
        if snapshot.candidates.is_empty() {
            snapshot.selected_candidate_id = None;
            snapshot.usb = StatusDimension::unavailable("No Puppis candidate detected");
            snapshot.protocol = StatusDimension::unavailable("No verified Puppis");
        } else if snapshot.selected_candidate_id.is_none() {
            snapshot.usb = StatusDimension {
                level: StatusLevel::Attention,
                summary: "Select a Puppis candidate".into(),
                guidance: Some("Choose the USB network adapter connected to the Puppis.".into()),
            };
            snapshot.protocol = StatusDimension::unavailable("No verified Puppis");
        } else {
            apply_selected_candidate_status(&mut snapshot);
        }
        if snapshot.selected_candidate_id != previous_selection && previous_selection.is_some() {
            clear_selected_device_state(&mut snapshot);
            self.environment.reset_protocol_session();
        }
        Ok(snapshot.clone())
    }

    pub fn select_candidate(
        &self,
        candidate_id: &str,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        if !snapshot
            .candidates
            .iter()
            .any(|candidate| candidate.id == candidate_id)
        {
            return Err(OperationFailure::safe(
                "candidate_not_found",
                "That Puppis candidate is no longer available.",
                "Refresh device discovery and select an available candidate.",
            ));
        }
        snapshot.revision += 1;
        let changed = snapshot.selected_candidate_id.as_deref() != Some(candidate_id);
        snapshot.selected_candidate_id = Some(candidate_id.into());
        if changed {
            clear_selected_device_state(&mut snapshot);
            self.environment.reset_protocol_session();
        }
        apply_selected_candidate_status(&mut snapshot);
        Ok(snapshot.clone())
    }

    pub fn verify_selected_puppis(&self) -> Result<ApplicationSnapshot, OperationFailure> {
        if self
            .snapshot
            .read()
            .expect("application state poisoned")
            .selected_candidate_id
            .is_none()
        {
            return Err(OperationFailure::safe(
                "candidate_required",
                "Select a Puppis candidate before verification.",
                "Refresh discovery and select the USB network adapter to inspect.",
            ));
        }
        let identity = self.environment.identity()?;
        if identity.model != "P1411" {
            return Err(OperationFailure::safe(
                "identity_not_p1411",
                "The selected adapter did not identify as a P1411.",
                "Select another candidate. No device settings were changed.",
            ));
        }
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.mutations_qualified = identity.firmware == QUALIFIED_FIRMWARE;
        snapshot.protocol = if snapshot.mutations_qualified {
            StatusDimension {
                level: StatusLevel::Healthy,
                summary: "Verified P1411".into(),
                guidance: None,
            }
        } else {
            StatusDimension {
                level: StatusLevel::Attention,
                summary: "Verified P1411 · firmware not qualified".into(),
                guidance: Some(
                    "Read-only diagnostics remain available; device mutations are disabled.".into(),
                ),
            }
        };
        if !snapshot.mutations_qualified {
            snapshot.configuration = StatusDimension {
                level: StatusLevel::Attention,
                summary: "Read-only diagnostics available".into(),
                guidance: Some(
                    "This exact firmware has not completed reversible mutation qualification."
                        .into(),
                ),
            };
        }
        snapshot.verified_puppis = Some(identity);
        snapshot.device_role = DeviceRole::from_code(
            &snapshot
                .verified_puppis
                .as_ref()
                .expect("identity stored")
                .role_code,
        );
        if let Some(log) = &self.log {
            let _ = log.record(log::SafeEvent::PuppisVerified);
        }
        Ok(snapshot.clone())
    }

    pub fn inspect_host_sharing(&self) -> Result<ApplicationSnapshot, OperationFailure> {
        let interface_name = self.selected_interface()?;
        let observation = self.environment.inspect_host_sharing(&interface_name)?;
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.sharing = if observation.subnet_conflict {
            StatusDimension {
                level: StatusLevel::Attention,
                summary: "Puppis subnet conflict detected".into(),
                guidance: Some(
                    "Resolve the other 192.168.137.0/24 route before enabling managed sharing."
                        .into(),
                ),
            }
        } else if observation.active
            && observation.ipv4_shared
            && observation.profile_kind == ProfileKind::External
        {
            StatusDimension {
                level: StatusLevel::Healthy,
                summary: "Host sharing works through an external profile".into(),
                guidance: Some("The manager will observe this profile but never modify it.".into()),
            }
        } else if observation.active
            && observation.ipv4_shared
            && observation.profile_kind == ProfileKind::Managed
        {
            StatusDimension {
                level: StatusLevel::Healthy,
                summary: "Managed host sharing is active".into(),
                guidance: None,
            }
        } else if !observation.upstream_available {
            StatusDimension {
                level: StatusLevel::Attention,
                summary: "No upstream internet route".into(),
                guidance: Some(
                    "Connect the host to an upstream network before enabling sharing.".into(),
                ),
            }
        } else {
            StatusDimension {
                level: StatusLevel::Attention,
                summary: "Host sharing is not configured".into(),
                guidance: Some(
                    "Review the proposed NetworkManager profile before enabling it.".into(),
                ),
            }
        };
        snapshot.host_sharing = Some(observation);
        Ok(snapshot.clone())
    }

    pub fn enable_managed_sharing(&self) -> Result<ApplicationSnapshot, OperationFailure> {
        let _lease = self
            .mutation_lease
            .try_lock()
            .map_err(|_| operation_busy())?;
        let interface_name = self.selected_interface()?;
        let observed = self.environment.inspect_host_sharing(&interface_name)?;
        if observed.subnet_conflict {
            return Err(OperationFailure::safe(
                "puppis_subnet_conflict",
                "Managed sharing cannot start while another route uses the Puppis subnet.",
                "Resolve the 192.168.137.0/24 conflict and inspect again.",
            ));
        }
        if !observed.upstream_available {
            return Err(OperationFailure::safe(
                "upstream_unavailable",
                "The host has no effective upstream route to share.",
                "Connect the host to the intended upstream network and try again.",
            ));
        }
        let result = {
            let _active = ActiveOperationGuard::new(
                Arc::clone(&self.snapshot),
                "Enable managed host sharing",
            );
            (|| {
                let mut checkpoint_active = false;
                let identity_missing = self
                    .snapshot
                    .read()
                    .expect("application state poisoned")
                    .verified_puppis
                    .is_none();
                if identity_missing {
                    match self.verify_selected_puppis() {
                        Ok(_) => {}
                        Err(first_error) => {
                            if first_error.code != "protocol_unreachable" {
                                return Err(first_error);
                            }
                            self.environment.checkpoint(&interface_name)?;
                            checkpoint_active = true;
                            if let Err(error) = self
                                .environment
                                .prepare_bootstrap(&interface_name)
                                .and_then(|_| self.verify_selected_puppis().map(|_| ()))
                            {
                                let _ = self.environment.finish_bootstrap(&interface_name, false);
                                let _ = self.environment.finish_checkpoint(&interface_name, false);
                                return Err(error);
                            }
                        }
                    }
                }
                if !checkpoint_active {
                    self.environment.checkpoint(&interface_name)?;
                }
                let change = self
                    .environment
                    .reconcile_managed(&interface_name)
                    .and_then(|_| self.environment.finish_bootstrap(&interface_name, true));
                match change {
                    Ok(()) => self.environment.finish_checkpoint(&interface_name, true),
                    Err(error) => {
                        let _ = self.environment.finish_bootstrap(&interface_name, false);
                        let _ = self.environment.finish_checkpoint(&interface_name, false);
                        Err(error)
                    }
                }
            })()
        };
        result?;
        self.inspect_host_sharing()
    }

    pub fn read_radio_settings(&self) -> Result<ApplicationSnapshot, OperationFailure> {
        let (identity, qualified) = {
            let snapshot = self.snapshot.read().expect("application state poisoned");
            (
                snapshot.verified_puppis.clone().ok_or_else(|| {
                    OperationFailure::safe(
                        "verified_puppis_required",
                        "Verify a P1411 before reading its radio settings.",
                        "Select and verify a Puppis candidate first.",
                    )
                })?,
                snapshot.mutations_qualified,
            )
        };
        let five = self.environment.read_radio(RadioBand::FiveGhz)?;
        let two = self.environment.read_radio(RadioBand::TwoPointFourGhz)?;
        let make = |band: RadioBand, radio: RadioConfiguration| {
            let channels = if qualified && radio.country == "CA" {
                match band {
                    RadioBand::FiveGhz => vec!["0".into(), "36".into()],
                    RadioBand::TwoPointFourGhz => vec!["0".into(), "6".into()],
                }
            } else {
                Vec::new()
            };
            let password = qualified && band == RadioBand::FiveGhz;
            let reason = if !qualified {
                Some("This exact firmware is not qualified for device mutations.".into())
            } else if band == RadioBand::TwoPointFourGhz {
                Some("Password restoration has not completed reversible qualification.".into())
            } else {
                None
            };
            RadioSnapshot {
                band,
                ssid: radio.ssid,
                channel: radio.channel,
                country: radio.country,
                enabled: radio.enabled == "1",
                bandwidth: radio.bandwidth,
                ssid_mutation: qualified,
                password_mutation: password,
                qualified_channels: channels,
                password_unavailable_reason: reason,
            }
        };
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.radios = vec![
            make(RadioBand::FiveGhz, five),
            make(RadioBand::TwoPointFourGhz, two),
        ];
        snapshot.configuration = StatusDimension {
            level: if qualified {
                StatusLevel::Healthy
            } else {
                StatusLevel::Attention
            },
            summary: if qualified {
                "Qualified radio settings available"
            } else {
                "Radio settings available read-only"
            }
            .into(),
            guidance: (!qualified).then(|| {
                format!(
                    "Firmware {} is not in the exact mutation allowlist.",
                    identity.firmware
                )
            }),
        };
        Ok(snapshot.clone())
    }

    pub fn set_radio_ssid(
        &self,
        band: RadioBand,
        ssid: &str,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        if ssid.is_empty()
            || ssid.len() > 18
            || !ssid.bytes().all(|byte| byte.is_ascii_alphanumeric())
        {
            return Err(OperationFailure::safe(
                "invalid_ssid",
                "The network name must contain 1–18 ASCII letters or numbers.",
                "Choose a name matching the qualified P1411 format.",
            ));
        }
        self.apply_radio_settings(
            band,
            RadioUpdate {
                ssid: Some(ssid.into()),
                ..RadioUpdate::default()
            },
        )
    }

    pub fn set_radio_channel(
        &self,
        band: RadioBand,
        channel: &str,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        let qualified = self
            .snapshot
            .read()
            .expect("application state poisoned")
            .radios
            .iter()
            .find(|radio| radio.band == band)
            .is_some_and(|radio| {
                radio
                    .qualified_channels
                    .iter()
                    .any(|qualified| qualified == channel)
            });
        if !qualified {
            return Err(OperationFailure::safe(
                "channel_not_qualified",
                "That channel is not qualified for this radio, firmware, and country.",
                "Choose Automatic or another channel shown as qualified.",
            ));
        }
        self.apply_radio_settings(
            band,
            RadioUpdate {
                channel: Some(channel.into()),
                ..RadioUpdate::default()
            },
        )
    }

    pub fn set_radio_password(
        &self,
        band: RadioBand,
        password: &str,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        let available = self
            .snapshot
            .read()
            .expect("application state poisoned")
            .radios
            .iter()
            .find(|radio| radio.band == band)
            .is_some_and(|radio| radio.password_mutation);
        if !available {
            return Err(OperationFailure::safe(
                "password_mutation_unavailable",
                "Password replacement is not qualified for this radio.",
                "Keep the current password and review the capability explanation.",
            ));
        }
        let allowed =
            |byte: u8| byte.is_ascii_alphanumeric() || b"?!@&$%*_~^#-/.+:;=".contains(&byte);
        if password.len() < 8 || password.len() > 63 || !password.bytes().all(allowed) {
            return Err(OperationFailure::safe(
                "invalid_password",
                "The replacement password does not match the qualified P1411 format.",
                "Use 8–63 qualified ASCII letters, numbers, or symbols.",
            ));
        }
        self.apply_radio_settings(
            band,
            RadioUpdate {
                password: Some(password.into()),
                ..RadioUpdate::default()
            },
        )
    }

    pub fn apply_radio_settings(
        &self,
        band: RadioBand,
        update: RadioUpdate,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        if let Some(ssid) = update.ssid.as_deref()
            && (ssid.is_empty()
                || ssid.len() > 18
                || !ssid.bytes().all(|byte| byte.is_ascii_alphanumeric()))
        {
            return Err(OperationFailure::safe(
                "invalid_ssid",
                "The network name must contain 1–18 ASCII letters or numbers.",
                "Choose a name matching the qualified P1411 format.",
            ));
        }
        let radio = self
            .snapshot
            .read()
            .expect("application state poisoned")
            .radios
            .iter()
            .find(|radio| radio.band == band)
            .cloned()
            .ok_or_else(|| {
                OperationFailure::safe(
                    "radio_unavailable",
                    "The requested radio settings are unavailable.",
                    "Read current radio settings before applying changes.",
                )
            })?;
        if update.ssid.is_some() && !radio.ssid_mutation {
            return Err(OperationFailure::safe(
                "mutation_not_qualified",
                "Network-name changes are unavailable for this firmware.",
                "Use read-only settings and diagnostics.",
            ));
        }
        if let Some(channel) = update.channel.as_deref()
            && !radio.qualified_channels.iter().any(|item| item == channel)
        {
            return Err(OperationFailure::safe(
                "channel_not_qualified",
                "That channel is not qualified for this radio, firmware, and country.",
                "Choose Automatic or another channel shown as qualified.",
            ));
        }
        if let Some(password) = update.password.as_deref() {
            if !radio.password_mutation {
                return Err(OperationFailure::safe(
                    "password_mutation_unavailable",
                    "Password replacement is not qualified for this radio.",
                    "Keep the current password and review the capability explanation.",
                ));
            }
            let allowed =
                |byte: u8| byte.is_ascii_alphanumeric() || b"?!@&$%*_~^#-/.+:;=".contains(&byte);
            if password.len() < 8 || password.len() > 63 || !password.bytes().all(allowed) {
                return Err(OperationFailure::safe(
                    "invalid_password",
                    "The replacement password does not match the qualified P1411 format.",
                    "Use 8–63 qualified ASCII letters, numbers, or symbols.",
                ));
            }
        }
        if update.ssid.is_none() && update.channel.is_none() && update.password.is_none() {
            return Ok(self.snapshot());
        }
        self.transact_radio(
            band,
            format!("Apply {} settings", band_label(band)),
            |desired| {
                if let Some(ssid) = &update.ssid {
                    desired.ssid.clone_from(ssid);
                }
                if let Some(channel) = &update.channel {
                    desired.channel.clone_from(channel);
                }
                if let Some(password) = &update.password {
                    desired.password.clone_from(password);
                }
            },
        )
    }

    pub fn set_device_role(
        &self,
        requested: DeviceRole,
        confirmed: bool,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        if requested == DeviceRole::WifiAdapter {
            return Err(OperationFailure::safe(
                "role_not_supported",
                "Wi-Fi adapter mode is diagnostic-only.",
                "Choose PrismPulse mode or Wi-Fi hotspot mode.",
            ));
        }
        if !confirmed {
            return Err(OperationFailure::safe(
                "confirmation_required",
                "Confirm the disruptive device-role transition.",
                "Review both roles and the client-disconnection warning, then confirm.",
            ));
        }
        let _lease = self
            .mutation_lease
            .try_lock()
            .map_err(|_| operation_busy())?;
        {
            let snapshot = self.snapshot.read().expect("application state poisoned");
            if snapshot.recovery_required {
                return Err(recovery_failure());
            }
            if !snapshot.mutations_qualified {
                return Err(OperationFailure::safe(
                    "mutation_not_qualified",
                    "Device role changes are unavailable for this firmware.",
                    "Use read-only diagnostics.",
                ));
            }
        }
        let original = self.environment.read_role()?;
        if original == requested {
            return Ok(self.snapshot());
        }
        self.environment.set_interrupted_marker(true)?;
        if let Some(log) = &self.log {
            let _ = log.record(log::SafeEvent::SettingsTransactionStarted);
        }
        {
            let mut snapshot = self.snapshot.write().expect("application state poisoned");
            snapshot.revision += 1;
            snapshot.active_operation = Some(format!(
                "Switch from {} to {}",
                role_label(original),
                role_label(requested)
            ));
            snapshot.configuration = StatusDimension {
                level: StatusLevel::Attention,
                summary: "Device role transition in progress".into(),
                guidance: Some("Connected clients may disconnect.".into()),
            };
        }
        let write = self.environment.write_role(requested);
        let observed = self.environment.read_role();
        if write.is_ok() && observed.as_ref().is_ok_and(|role| role == &requested) {
            self.clear_transaction_state()?;
            let mut snapshot = self.snapshot.write().expect("application state poisoned");
            snapshot.revision += 1;
            snapshot.device_role = Some(requested);
            if let Some(identity) = snapshot.verified_puppis.as_mut() {
                identity.role_code = requested.code().into();
            }
            snapshot.configuration = StatusDimension {
                level: StatusLevel::Healthy,
                summary: format!("{} verified", role_label(requested)),
                guidance: None,
            };
            return Ok(snapshot.clone());
        }
        if observed.as_ref().is_ok_and(|role| role == &original) {
            self.clear_transaction_state()?;
            return Err(OperationFailure::safe(
                "transaction_not_applied",
                "The Puppis role did not change.",
                "Review the current role and try again if needed.",
            ));
        }
        let restored = self
            .environment
            .write_role(original)
            .and_then(|_| self.environment.read_role());
        if restored.as_ref().is_ok_and(|role| role == &original) {
            self.clear_transaction_state()?;
            return Err(OperationFailure::safe(
                "transaction_rolled_back",
                "The requested role could not be verified, so the original role was restored.",
                "Review diagnostics before trying again.",
            ));
        }
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.active_operation = None;
        snapshot.recovery_required = true;
        snapshot.configuration = StatusDimension {
            level: StatusLevel::Attention,
            summary: "Recovery required".into(),
            guidance: Some("The device role could not be verified or restored.".into()),
        };
        Err(recovery_failure())
    }

    pub fn refresh_client_evidence(
        &self,
        telemetry_visible: bool,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        if !telemetry_visible {
            return Ok(self.snapshot());
        }
        let interface_name = self.selected_interface()?;
        let (connected, recent) = self.environment.read_client_evidence(&interface_name)?;
        let puppis_hardware_addresses: HashSet<_> = connected
            .iter()
            .chain(&recent)
            .filter(|observation| observation.address.as_deref() == Some("192.168.137.254"))
            .map(|observation| observation.hardware_address.to_ascii_lowercase())
            .collect();
        let retain_client = |observation: &ClientNetworkObservation| {
            let hardware_address = observation.hardware_address.to_ascii_lowercase();
            !puppis_hardware_addresses.contains(&hardware_address)
                && !matches!(
                    observation.address.as_deref(),
                    Some("192.168.137.0" | "192.168.137.1" | "192.168.137.255")
                )
        };
        let connected: Vec<_> = connected.into_iter().filter(&retain_client).collect();
        let recent: Vec<_> = recent.into_iter().filter(retain_client).collect();
        let connected_set: HashSet<_> = connected
            .iter()
            .map(|observation| observation.hardware_address.to_ascii_lowercase())
            .collect();
        let mut grouped =
            std::collections::BTreeMap::<String, (Vec<String>, ClientEvidenceKind)>::new();
        for observation in connected.into_iter().chain(recent) {
            let hardware_address = observation.hardware_address.to_ascii_lowercase();
            let kind = if connected_set.contains(&hardware_address) {
                ClientEvidenceKind::Connected
            } else {
                ClientEvidenceKind::RecentlyObserved
            };
            let entry = grouped
                .entry(hardware_address)
                .or_insert_with(|| (Vec::new(), kind));
            if let Some(address) = observation.address
                && !entry.0.contains(&address)
            {
                entry.0.push(address);
                entry.0.sort();
            }
        }
        let mut evidence: Vec<_> = grouped
            .into_iter()
            .enumerate()
            .map(|(index, (hardware_address, (addresses, kind)))| {
                let saved = self
                    .environment
                    .saved_clients
                    .read()
                    .expect("saved client state poisoned")
                    .iter()
                    .find(|client| client.hardware_address == hardware_address)
                    .cloned();
                ClientEvidence {
                    alias: format!("client-{}", index + 1),
                    display_name: saved
                        .as_ref()
                        .map(|client| client.label.clone())
                        .unwrap_or_else(|| "Unlabeled client".into()),
                    hardware_address,
                    addresses,
                    kind,
                    saved: saved.is_some(),
                }
            })
            .collect();
        sort_client_evidence(&mut evidence);
        let connected_count = evidence
            .iter()
            .filter(|item| item.kind == ClientEvidenceKind::Connected)
            .count();
        let recent_count = evidence.len() - connected_count;
        let now = self.environment.current_time();
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.observed_status_at = Some(now);
        snapshot.client_evidence = evidence;
        {
            let mut saved_clients_guard = self
                .environment
                .saved_clients
                .write()
                .expect("saved client state poisoned");
            let mut saved_clients = saved_clients_guard.clone();
            let mut changed = false;
            for saved in saved_clients.iter_mut() {
                if let Some(observed) = snapshot
                    .client_evidence
                    .iter()
                    .find(|item| item.hardware_address == saved.hardware_address)
                    && (saved.last_observed_at != now || saved.addresses != observed.addresses)
                {
                    saved.last_observed_at = now;
                    saved.addresses.clone_from(&observed.addresses);
                    changed = true;
                }
            }
            if changed {
                if let Err(failure) = self.environment.persist_saved_clients(&saved_clients) {
                    *self
                        .environment
                        .saved_clients_failure
                        .write()
                        .expect("saved client failure state poisoned") = Some(failure.clone());
                    snapshot.saved_clients_available = false;
                    snapshot.saved_clients_failure = Some(failure);
                } else {
                    saved_clients_guard.clone_from(&saved_clients);
                }
            }
            snapshot.saved_clients.clone_from(&saved_clients_guard);
        }
        snapshot.clients = if connected_count > 0 {
            StatusDimension {
                level: StatusLevel::Healthy,
                summary: format!(
                    "{connected_count} connected client{}",
                    if connected_count == 1 { "" } else { "s" }
                ),
                guidance: (recent_count > 0).then(|| {
                    format!(
                        "{recent_count} additional client{} recently observed by the host.",
                        if recent_count == 1 { " was" } else { "s were" }
                    )
                }),
            }
        } else if recent_count > 0 {
            StatusDimension {
                level: StatusLevel::Unknown,
                summary: format!(
                    "{recent_count} recently observed client{}",
                    if recent_count == 1 { "" } else { "s" }
                ),
                guidance: Some(
                    "Host evidence does not prove a current wireless connection.".into(),
                ),
            }
        } else {
            StatusDimension {
                level: StatusLevel::Unavailable,
                summary: "No client evidence".into(),
                guidance: None,
            }
        };
        Ok(snapshot.clone())
    }

    pub fn save_client_label(
        &self,
        hardware_address: &str,
        label: &str,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        let label = label.trim();
        if label.is_empty() || label.chars().count() > 40 || label.chars().any(char::is_control) {
            return Err(OperationFailure::safe(
                "invalid_client_label",
                "Client labels must contain 1 to 40 printable characters.",
                "Choose a shorter label without control characters.",
            ));
        }
        let hardware_address = hardware_address.to_ascii_lowercase();
        let observed = self
            .snapshot
            .read()
            .expect("application state poisoned")
            .client_evidence
            .iter()
            .find(|client| client.hardware_address == hardware_address)
            .cloned()
            .ok_or_else(|| {
                OperationFailure::safe(
                    "observed_client_required",
                    "Only a currently observed client can be saved.",
                    "Wait for local traffic from the client, refresh, and try again.",
                )
            })?;
        let mut saved_clients_guard = self
            .environment
            .saved_clients
            .write()
            .expect("saved client state poisoned");
        let mut saved_clients = saved_clients_guard.clone();
        if saved_clients.iter().any(|client| {
            client.hardware_address != hardware_address
                && client.label.to_lowercase() == label.to_lowercase()
        }) {
            return Err(OperationFailure::safe(
                "client_label_not_unique",
                "That client label is already in use.",
                "Choose a unique label or rename the existing saved client.",
            ));
        }
        if saved_clients.len() >= 10
            && !saved_clients
                .iter()
                .any(|client| client.hardware_address == hardware_address)
        {
            return Err(OperationFailure::safe(
                "saved_client_limit_reached",
                "Up to 10 clients can be saved.",
                "Forget a saved client before adding another.",
            ));
        }
        let saved = SavedClient {
            label: label.into(),
            hardware_address: hardware_address.clone(),
            last_observed_at: self.environment.current_time(),
            addresses: observed.addresses,
        };
        if let Some(existing) = saved_clients
            .iter_mut()
            .find(|client| client.hardware_address == hardware_address)
        {
            *existing = saved;
        } else {
            saved_clients.push(saved);
        }
        saved_clients.sort_by_key(|client| client.label.to_lowercase());
        self.environment.persist_saved_clients(&saved_clients)?;
        saved_clients_guard.clone_from(&saved_clients);
        drop(saved_clients_guard);
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.saved_clients = saved_clients;
        if let Some(client) = snapshot
            .client_evidence
            .iter_mut()
            .find(|client| client.hardware_address == hardware_address)
        {
            client.display_name = label.into();
            client.saved = true;
        }
        sort_client_evidence(&mut snapshot.client_evidence);
        Ok(snapshot.clone())
    }

    pub fn rename_saved_client(
        &self,
        hardware_address: &str,
        label: &str,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        let label = label.trim();
        if label.is_empty() || label.chars().count() > 40 || label.chars().any(char::is_control) {
            return Err(OperationFailure::safe(
                "invalid_client_label",
                "Client labels must contain 1 to 40 printable characters.",
                "Choose a shorter label without control characters.",
            ));
        }
        let hardware_address = hardware_address.to_ascii_lowercase();
        let mut guard = self
            .environment
            .saved_clients
            .write()
            .expect("saved client state poisoned");
        let mut saved_clients = guard.clone();
        if saved_clients.iter().any(|client| {
            client.hardware_address != hardware_address
                && client.label.to_lowercase() == label.to_lowercase()
        }) {
            return Err(OperationFailure::safe(
                "client_label_not_unique",
                "That client label is already in use.",
                "Choose a unique label or rename the existing saved client.",
            ));
        }
        let Some(saved) = saved_clients
            .iter_mut()
            .find(|client| client.hardware_address == hardware_address)
        else {
            return Err(OperationFailure::safe(
                "saved_client_not_found",
                "That saved client no longer exists.",
                "Refresh saved clients and try again.",
            ));
        };
        saved.label = label.into();
        saved_clients.sort_by_key(|client| client.label.to_lowercase());
        self.environment.persist_saved_clients(&saved_clients)?;
        guard.clone_from(&saved_clients);
        drop(guard);
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.saved_clients = saved_clients;
        if let Some(client) = snapshot
            .client_evidence
            .iter_mut()
            .find(|client| client.hardware_address == hardware_address)
        {
            client.display_name = label.into();
        }
        sort_client_evidence(&mut snapshot.client_evidence);
        Ok(snapshot.clone())
    }

    pub fn reassociate_saved_client(
        &self,
        previous_hardware_address: &str,
        new_hardware_address: &str,
        confirmed: bool,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        if !confirmed {
            return Err(OperationFailure::safe(
                "confirmation_required",
                "Confirm the saved-client reassociation.",
                "Review the old and new hardware addresses before continuing.",
            ));
        }
        let previous_hardware_address = previous_hardware_address.to_ascii_lowercase();
        let new_hardware_address = new_hardware_address.to_ascii_lowercase();
        let observed = self
            .snapshot
            .read()
            .expect("application state poisoned")
            .client_evidence
            .iter()
            .find(|client| client.hardware_address == new_hardware_address)
            .cloned()
            .ok_or_else(|| {
                OperationFailure::safe(
                    "observed_client_required",
                    "The new hardware address is not currently observed.",
                    "Wait for local traffic from that client, refresh, and try again.",
                )
            })?;
        let mut guard = self
            .environment
            .saved_clients
            .write()
            .expect("saved client state poisoned");
        let mut saved_clients = guard.clone();
        if saved_clients
            .iter()
            .any(|client| client.hardware_address == new_hardware_address)
        {
            return Err(OperationFailure::safe(
                "saved_client_already_exists",
                "The new hardware address already belongs to a saved client.",
                "Forget or rename the existing saved client first.",
            ));
        }
        let saved = saved_clients
            .iter_mut()
            .find(|client| client.hardware_address == previous_hardware_address)
            .ok_or_else(|| {
                OperationFailure::safe(
                    "saved_client_not_found",
                    "That saved client no longer exists.",
                    "Refresh saved clients and try again.",
                )
            })?;
        saved.hardware_address = new_hardware_address.clone();
        saved.last_observed_at = self.environment.current_time();
        saved.addresses = observed.addresses;
        let label = saved.label.clone();
        self.environment.persist_saved_clients(&saved_clients)?;
        guard.clone_from(&saved_clients);
        drop(guard);
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.saved_clients = saved_clients;
        for client in &mut snapshot.client_evidence {
            if client.hardware_address == previous_hardware_address {
                client.display_name = "Unlabeled client".into();
                client.saved = false;
            } else if client.hardware_address == new_hardware_address {
                client.display_name.clone_from(&label);
                client.saved = true;
            }
        }
        sort_client_evidence(&mut snapshot.client_evidence);
        Ok(snapshot.clone())
    }

    pub fn forget_saved_client(
        &self,
        hardware_address: &str,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        let hardware_address = hardware_address.to_ascii_lowercase();
        let mut guard = self
            .environment
            .saved_clients
            .write()
            .expect("saved client state poisoned");
        let mut saved_clients = guard.clone();
        saved_clients.retain(|client| client.hardware_address != hardware_address);
        self.environment.persist_saved_clients(&saved_clients)?;
        guard.clone_from(&saved_clients);
        drop(guard);
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.saved_clients = saved_clients;
        if let Some(client) = snapshot
            .client_evidence
            .iter_mut()
            .find(|client| client.hardware_address == hardware_address)
        {
            client.display_name = "Unlabeled client".into();
            client.saved = false;
        }
        sort_client_evidence(&mut snapshot.client_evidence);
        Ok(snapshot.clone())
    }

    pub fn clear_saved_clients(
        &self,
        confirmed: bool,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        if !confirmed {
            return Err(OperationFailure::safe(
                "confirmation_required",
                "Confirm removal of all saved clients.",
                "This removes every saved label and recognition record.",
            ));
        }
        self.environment.persist_saved_clients(&[])?;
        self.environment
            .saved_clients
            .write()
            .expect("saved client state poisoned")
            .clear();
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.saved_clients.clear();
        for client in &mut snapshot.client_evidence {
            client.display_name = "Unlabeled client".into();
            client.saved = false;
        }
        sort_client_evidence(&mut snapshot.client_evidence);
        Ok(snapshot.clone())
    }

    pub fn reset_saved_clients(
        &self,
        confirmed: bool,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        if !confirmed {
            return Err(OperationFailure::safe(
                "confirmation_required",
                "Confirm reset of saved client recognition data.",
                "The unreadable or unsupported saved-client file will be replaced.",
            ));
        }
        let path = self
            .environment
            .saved_clients_path
            .as_ref()
            .ok_or_else(|| {
                OperationFailure::safe(
                    "saved_clients_storage_failed",
                    "Saved client recognition storage is unavailable.",
                    "Set HOME or XDG_STATE_HOME to a writable user state directory.",
                )
            })?;
        client_state::save(path, &[])?;
        self.environment
            .saved_clients
            .write()
            .expect("saved client state poisoned")
            .clear();
        *self
            .environment
            .saved_clients_failure
            .write()
            .expect("saved client failure state poisoned") = None;
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.saved_clients.clear();
        snapshot.saved_clients_available = true;
        snapshot.saved_clients_failure = None;
        for client in &mut snapshot.client_evidence {
            client.display_name = "Unlabeled client".into();
            client.saved = false;
        }
        sort_client_evidence(&mut snapshot.client_evidence);
        Ok(snapshot.clone())
    }

    pub fn accept_current_configuration_as_baseline(
        &self,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        let _lease = self
            .mutation_lease
            .try_lock()
            .map_err(|_| operation_busy())?;
        if !self
            .snapshot
            .read()
            .expect("application state poisoned")
            .recovery_required
        {
            return Err(OperationFailure::safe(
                "recovery_not_required",
                "The device does not currently require recovery reconciliation.",
                "Continue using the verified current state.",
            ));
        }
        if self
            .snapshot
            .read()
            .expect("application state poisoned")
            .verified_puppis
            .is_none()
        {
            return Err(OperationFailure::safe(
                "verified_puppis_required",
                "Reverify the P1411 before accepting a recovery baseline.",
                "Reconnect and verify the device first.",
            ));
        }
        let five = self.environment.read_radio(RadioBand::FiveGhz)?;
        let two = self.environment.read_radio(RadioBand::TwoPointFourGhz)?;
        let role = self.environment.read_role()?;
        self.refresh_radio_snapshot(RadioBand::FiveGhz, &five);
        self.refresh_radio_snapshot(RadioBand::TwoPointFourGhz, &two);
        self.environment.set_interrupted_marker(false)?;
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.device_role = Some(role);
        if let Some(identity) = snapshot.verified_puppis.as_mut() {
            identity.role_code = role.code().into();
        }
        snapshot.recovery_required = false;
        snapshot.active_operation = None;
        snapshot.configuration = StatusDimension {
            level: StatusLevel::Healthy,
            summary: "Current device state accepted as the recovery baseline".into(),
            guidance: None,
        };
        Ok(snapshot.clone())
    }

    fn transact_radio(
        &self,
        band: RadioBand,
        operation: String,
        change: impl FnOnce(&mut RadioConfiguration),
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        let _lease = self
            .mutation_lease
            .try_lock()
            .map_err(|_| operation_busy())?;
        {
            let snapshot = self.snapshot.read().expect("application state poisoned");
            if snapshot.recovery_required {
                return Err(recovery_failure());
            }
            if !snapshot.mutations_qualified {
                return Err(OperationFailure::safe(
                    "mutation_not_qualified",
                    "Device changes are unavailable for this firmware.",
                    "Use read-only diagnostics until this exact firmware is qualified.",
                ));
            }
        }
        let original = self.environment.read_radio(band)?;
        let mut desired = original.clone();
        change(&mut desired);
        if desired == original {
            return Ok(self.snapshot());
        }

        self.environment.set_interrupted_marker(true)?;
        {
            let mut snapshot = self.snapshot.write().expect("application state poisoned");
            snapshot.revision += 1;
            snapshot.active_operation = Some(operation);
            snapshot.configuration = StatusDimension {
                level: StatusLevel::Attention,
                summary: "Settings transaction in progress".into(),
                guidance: None,
            };
        }

        let write_result = self.environment.write_radio(band, &desired);
        let observed = self.environment.read_radio(band);
        if write_result.is_ok() && observed.as_ref().is_ok_and(|state| state == &desired) {
            return self.finish_radio_transaction(band, desired);
        }
        if observed.as_ref().is_ok_and(|state| state == &original) {
            self.clear_transaction_state()?;
            return Err(OperationFailure::safe(
                "transaction_not_applied",
                "The Puppis settings did not change.",
                "Review the current settings and try again if needed.",
            ));
        }

        let restored = self
            .environment
            .write_radio(band, &original)
            .and_then(|_| self.environment.read_radio(band));
        if restored.as_ref().is_ok_and(|state| state == &original) {
            self.clear_transaction_state()?;
            self.refresh_radio_snapshot(band, &original);
            return Err(OperationFailure::safe(
                "transaction_rolled_back",
                "The requested change could not be verified, so the original settings were restored.",
                "Review diagnostics before trying the change again.",
            ));
        }
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.active_operation = None;
        snapshot.recovery_required = true;
        snapshot.configuration = StatusDimension { level: StatusLevel::Attention, summary: "Recovery required".into(), guidance: Some("The device state could not be verified or restored. Reconcile it before any further mutation.".into()) };
        if let Some(log) = &self.log {
            let _ = log.record(log::SafeEvent::RecoveryRequired);
        }
        Err(recovery_failure())
    }

    fn finish_radio_transaction(
        &self,
        band: RadioBand,
        desired: RadioConfiguration,
    ) -> Result<ApplicationSnapshot, OperationFailure> {
        self.clear_transaction_state()?;
        self.refresh_radio_snapshot(band, &desired);
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.configuration = StatusDimension {
            level: StatusLevel::Healthy,
            summary: "Settings transaction verified".into(),
            guidance: None,
        };
        if let Some(log) = &self.log {
            let _ = log.record(log::SafeEvent::SettingsTransactionVerified);
        }
        Ok(snapshot.clone())
    }

    fn clear_transaction_state(&self) -> Result<(), OperationFailure> {
        self.environment.set_interrupted_marker(false)?;
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        snapshot.revision += 1;
        snapshot.active_operation = None;
        Ok(())
    }

    fn refresh_radio_snapshot(&self, band: RadioBand, configuration: &RadioConfiguration) {
        let mut snapshot = self.snapshot.write().expect("application state poisoned");
        if let Some(radio) = snapshot.radios.iter_mut().find(|radio| radio.band == band) {
            radio.ssid = configuration.ssid.clone();
            radio.channel = configuration.channel.clone();
            radio.country = configuration.country.clone();
            radio.enabled = configuration.enabled == "1";
            radio.bandwidth = configuration.bandwidth.clone();
        }
    }

    pub fn disable_managed_sharing(&self) -> Result<ApplicationSnapshot, OperationFailure> {
        let _lease = self
            .mutation_lease
            .try_lock()
            .map_err(|_| operation_busy())?;
        let interface_name = self.selected_interface()?;
        {
            let _active = ActiveOperationGuard::new(
                Arc::clone(&self.snapshot),
                "Disable managed host sharing",
            );
            self.environment.disable_managed(&interface_name)?;
        }
        self.inspect_host_sharing()
    }

    pub fn remove_managed_sharing(&self) -> Result<ApplicationSnapshot, OperationFailure> {
        let _lease = self
            .mutation_lease
            .try_lock()
            .map_err(|_| operation_busy())?;
        let interface_name = self.selected_interface()?;
        {
            let _active = ActiveOperationGuard::new(
                Arc::clone(&self.snapshot),
                "Remove managed host sharing",
            );
            self.environment.remove_managed(&interface_name)?;
        }
        self.inspect_host_sharing()
    }

    fn selected_interface(&self) -> Result<String, OperationFailure> {
        let snapshot = self.snapshot.read().expect("application state poisoned");
        let selected = snapshot.selected_candidate_id.as_deref().ok_or_else(|| {
            OperationFailure::safe(
                "candidate_required",
                "Select a Puppis candidate first.",
                "Refresh device discovery and select a candidate.",
            )
        })?;
        snapshot
            .candidates
            .iter()
            .find(|candidate| candidate.id == selected)
            .map(|candidate| candidate.interface_name.clone())
            .ok_or_else(|| {
                OperationFailure::safe(
                    "candidate_not_found",
                    "That Puppis candidate is no longer available.",
                    "Refresh device discovery.",
                )
            })
    }

    pub fn preview_diagnostics(&self) -> Result<DiagnosticsBundle, OperationFailure> {
        diagnostics::preview(&self.snapshot())
    }

    pub fn export_diagnostics(
        &self,
        path: &std::path::Path,
        approved_preview: &str,
    ) -> Result<(), OperationFailure> {
        let current = self.preview_diagnostics()?;
        if current.preview != approved_preview {
            return Err(OperationFailure::safe(
                "diagnostics_preview_stale",
                "The diagnostics preview changed before export.",
                "Review the refreshed preview before saving it.",
            ));
        }
        std::fs::write(path, approved_preview).map_err(|_| {
            OperationFailure::safe(
                "diagnostics_export_failed",
                "The diagnostics bundle could not be saved.",
                "Choose a writable destination and try again.",
            )
        })
    }
}

fn operation_busy() -> OperationFailure {
    OperationFailure::safe(
        "operation_in_progress",
        "Another host or device change is already in progress.",
        "Wait for the active operation to reach a verified terminal state.",
    )
}

fn recovery_failure() -> OperationFailure {
    OperationFailure::safe(
        "recovery_required",
        "The Puppis configuration requires reconciliation.",
        "Use read-only settings and diagnostics, then accept a verified baseline before another mutation.",
    )
}

fn band_label(band: RadioBand) -> &'static str {
    match band {
        RadioBand::FiveGhz => "5 GHz",
        RadioBand::TwoPointFourGhz => "2.4 GHz",
    }
}

fn role_label(role: DeviceRole) -> &'static str {
    match role {
        DeviceRole::PrismPulse => "PrismPulse mode",
        DeviceRole::WifiHotspot => "Wi-Fi hotspot mode",
        DeviceRole::WifiAdapter => "Wi-Fi adapter mode",
    }
}

fn sort_client_evidence(clients: &mut [ClientEvidence]) {
    clients.sort_by(|left, right| match (left.saved, right.saved) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (true, true) => left
            .display_name
            .to_lowercase()
            .cmp(&right.display_name.to_lowercase())
            .then_with(|| left.hardware_address.cmp(&right.hardware_address)),
        (false, false) => left.hardware_address.cmp(&right.hardware_address),
    });
}

fn role_unavailable() -> OperationFailure {
    OperationFailure::safe(
        "device_role_unavailable",
        "The current Puppis role is unavailable.",
        "Reconnect and verify the device before requesting a role transition.",
    )
}

fn marker_path() -> Option<std::path::PathBuf> {
    let root = std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".local/state"))
        })?;
    Some(root.join("puppis-s1-manager/interrupted-operation.json"))
}

fn system_marker_exists() -> bool {
    marker_path().is_some_and(|path| path.is_file())
}

fn write_system_marker(present: bool) -> Result<(), OperationFailure> {
    let path = marker_path().ok_or_else(|| {
        OperationFailure::safe(
            "state_directory_unavailable",
            "The recovery marker cannot be stored.",
            "Configure an XDG state directory before changing device settings.",
        )
    })?;
    if present {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| marker_failure())?;
        }
        std::fs::write(
            path,
            "{\"schemaVersion\":1,\"operation\":\"device-settings\"}\n",
        )
        .map_err(|_| marker_failure())
    } else if path.exists() {
        std::fs::remove_file(path).map_err(|_| marker_failure())
    } else {
        Ok(())
    }
}

fn marker_failure() -> OperationFailure {
    OperationFailure::safe(
        "recovery_marker_failed",
        "The interrupted-operation marker could not be updated.",
        "No further mutation is safe; inspect the state directory and diagnostics.",
    )
}

fn apply_selected_candidate_status(snapshot: &mut ApplicationSnapshot) {
    let Some(candidate) = snapshot
        .candidates
        .iter()
        .find(|candidate| snapshot.selected_candidate_id.as_deref() == Some(candidate.id.as_str()))
    else {
        return;
    };
    snapshot.usb = match candidate.link_speed {
        UsbLinkSpeed::SuperSpeed => StatusDimension {
            level: StatusLevel::Healthy,
            summary: "SuperSpeed USB connection".into(),
            guidance: None,
        },
        UsbLinkSpeed::Usb2 => StatusDimension {
            level: StatusLevel::Attention,
            summary: "USB 2 connection detected".into(),
            guidance: Some(
                "For best streaming performance, reconnect the Puppis through a SuperSpeed USB port."
                    .into(),
            ),
        },
        UsbLinkSpeed::Unknown => StatusDimension {
            level: StatusLevel::Unknown,
            summary: "USB link speed unavailable".into(),
            guidance: None,
        },
    };
    snapshot.protocol = StatusDimension {
        level: StatusLevel::Unknown,
        summary: "Candidate not yet verified".into(),
        guidance: Some("Verify its P1411 identity before changing device settings.".into()),
    };
}

fn clear_selected_device_state(snapshot: &mut ApplicationSnapshot) {
    snapshot.verified_puppis = None;
    snapshot.mutations_qualified = false;
    snapshot.host_sharing = None;
    snapshot.radios.clear();
    snapshot.device_role = None;
    snapshot.client_evidence.clear();
    snapshot.sharing = StatusDimension::unavailable("Host sharing not inspected");
    snapshot.configuration = StatusDimension::unavailable("Device configuration unavailable");
    snapshot.clients = StatusDimension::unavailable("Client evidence unavailable");
    snapshot.active_operation = None;
}
