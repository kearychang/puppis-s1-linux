use crate::{
    AuthorizationState, ClientNetworkObservation, HostSharingObservation, OperationFailure,
    ProfileKind,
};
use dbus::arg::{PropMap, RefArg, Variant};
use dbus::blocking::Connection;
use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

const NM: &str = "org.freedesktop.NetworkManager";
const ROOT: &str = "/org/freedesktop/NetworkManager";
const MANAGED_ID: &str = "Puppis S1 Manager";
const MANAGED_UUID: &str = "14e8cce1-e96d-4d55-b014-141100000001";
static CHECKPOINT: OnceLock<Mutex<Option<dbus::Path<'static>>>> = OnceLock::new();
static BOOTSTRAP: OnceLock<Mutex<Option<(dbus::Path<'static>, dbus::Path<'static>)>>> =
    OnceLock::new();

pub fn watch_changes(callback: Arc<dyn Fn() + Send + Sync>) {
    std::thread::spawn(move || {
        let Ok(connection) = Connection::new_system() else {
            return;
        };
        let mut rule = MatchRule::new();
        rule.msg_type = Some(dbus::MessageType::Signal);
        rule.sender = Some(NM.into());
        rule.path = Some(ROOT.into());
        rule.path_is_namespace = true;
        connection.start_receive(
            rule,
            Box::new(move |_, _| {
                callback();
                true
            }),
        );
        while connection.process(Duration::from_secs(30)).is_ok() {}
    });
}

pub fn inspect(interface_name: &str) -> Result<HostSharingObservation, OperationFailure> {
    let connection = Connection::new_system().map_err(|_| dbus_unavailable())?;
    let timeout = Duration::from_secs(4);
    let manager = connection.with_proxy(NM, ROOT, timeout);
    let primary: dbus::Path<'static> = manager
        .get("org.freedesktop.NetworkManager", "PrimaryConnection")
        .map_err(|_| dbus_unavailable())?;
    let device_path: dbus::Path<'static> = manager
        .method_call(
            "org.freedesktop.NetworkManager",
            "GetDeviceByIpIface",
            (interface_name,),
        )
        .map(|(path,): (dbus::Path<'static>,)| path)
        .map_err(|_| {
            OperationFailure::safe(
                "network_device_missing",
                "NetworkManager no longer sees the selected USB interface.",
                "Refresh USB discovery and select an available candidate.",
            )
        })?;
    let device = connection.with_proxy(NM, device_path, timeout);
    let active_path: dbus::Path<'static> = device
        .get("org.freedesktop.NetworkManager.Device", "ActiveConnection")
        .unwrap_or_else(|_| dbus::Path::from("/"));

    let mut profile_name = None;
    let mut profile_kind = ProfileKind::None;
    let mut active = false;
    let mut ipv4_shared = false;
    let mut ipv6_disabled = false;
    let mut autoconnect = false;
    let mut downstream_address = None;
    if active_path != "/" {
        active = true;
        profile_kind = ProfileKind::External;
        let active_proxy = connection.with_proxy(NM, active_path, timeout);
        let id: String = active_proxy
            .get("org.freedesktop.NetworkManager.Connection.Active", "Id")
            .unwrap_or_else(|_| "Unnamed profile".into());
        profile_name = Some(id);
        if let Ok(settings_path) = active_proxy.get::<dbus::Path<'static>>(
            "org.freedesktop.NetworkManager.Connection.Active",
            "Connection",
        ) {
            let settings_proxy = connection.with_proxy(NM, settings_path, timeout);
            if let Ok((settings,)) = settings_proxy
                .method_call::<(HashMap<String, PropMap>,), _, _, _>(
                    "org.freedesktop.NetworkManager.Settings.Connection",
                    "GetSettings",
                    (),
                )
            {
                profile_kind = profile_kind_for_uuid(
                    setting_string(&settings, "connection", "uuid").as_deref(),
                );
                ipv4_shared =
                    setting_string(&settings, "ipv4", "method").as_deref() == Some("shared");
                ipv6_disabled =
                    setting_string(&settings, "ipv6", "method").as_deref() == Some("disabled");
                autoconnect = setting_bool(&settings, "connection", "autoconnect").unwrap_or(false);
            }
        }
        let ip4_path: dbus::Path<'static> = device
            .get("org.freedesktop.NetworkManager.Device", "Ip4Config")
            .unwrap_or_else(|_| dbus::Path::from("/"));
        if ip4_path != "/" {
            let ip4 = connection.with_proxy(NM, ip4_path, timeout);
            if let Ok(addresses) =
                ip4.get::<Vec<PropMap>>("org.freedesktop.NetworkManager.IP4Config", "AddressData")
            {
                downstream_address = addresses.first().and_then(|entry| {
                    let address = entry.get("address")?.0.as_str()?;
                    let prefix = entry.get("prefix")?.0.as_u64()?;
                    Some(format!("{address}/{prefix}"))
                });
            }
        }
    }

    Ok(HostSharingObservation {
        upstream_available: primary != "/",
        upstream_description: (primary != "/").then(|| "Host effective default route".into()),
        profile_kind,
        profile_name,
        active,
        ipv4_shared,
        downstream_address,
        ipv6_disabled,
        autoconnect,
        subnet_conflict: has_subnet_conflict(interface_name),
        authorization: AuthorizationState::Prompt,
    })
}

fn setting_string(settings: &HashMap<String, PropMap>, group: &str, key: &str) -> Option<String> {
    settings.get(group)?.get(key)?.0.as_str().map(str::to_owned)
}

fn setting_bool(settings: &HashMap<String, PropMap>, group: &str, key: &str) -> Option<bool> {
    settings
        .get(group)?
        .get(key)?
        .0
        .as_i64()
        .map(|value| value != 0)
}

fn profile_kind_for_uuid(uuid: Option<&str>) -> ProfileKind {
    if uuid == Some(MANAGED_UUID) {
        ProfileKind::Managed
    } else {
        ProfileKind::External
    }
}

fn has_subnet_conflict(selected_interface: &str) -> bool {
    std::fs::read_to_string("/proc/net/route")
        .ok()
        .is_some_and(|routes| {
            routes.lines().skip(1).any(|line| {
                let fields: Vec<_> = line.split_whitespace().collect();
                fields.len() > 2
                    && fields[0] != selected_interface
                    && fields[1].eq_ignore_ascii_case("0089A8C0")
            })
        })
}

pub fn recent_clients(interface_name: &str) -> Vec<ClientNetworkObservation> {
    let mut clients: Vec<_> = std::fs::read_to_string("/proc/net/arp")
        .ok()
        .into_iter()
        .flat_map(|contents| {
            contents
                .lines()
                .skip(1)
                .filter_map(|line| {
                    let fields: Vec<_> = line.split_whitespace().collect();
                    (fields.len() >= 6 && fields[5] == interface_name && fields[2] == "0x2").then(
                        || ClientNetworkObservation {
                            hardware_address: fields[3].to_ascii_lowercase(),
                            address: Some(fields[0].into()),
                        },
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    clients.sort();
    clients.dedup();
    clients
}

fn dbus_unavailable() -> OperationFailure {
    OperationFailure::safe(
        "networkmanager_unavailable",
        "NetworkManager state is unavailable.",
        "Make sure NetworkManager is running, then inspect the network again.",
    )
}

pub fn checkpoint(interface_name: &str) -> Result<(), OperationFailure> {
    let connection = Connection::new_system().map_err(|_| dbus_unavailable())?;
    let manager = connection.with_proxy(NM, ROOT, Duration::from_secs(30));
    let device: dbus::Path<'static> = manager
        .method_call(
            "org.freedesktop.NetworkManager",
            "GetDeviceByIpIface",
            (interface_name,),
        )
        .map(|(path,): (dbus::Path<'static>,)| path)
        .map_err(network_change_failed)?;
    let checkpoint: dbus::Path<'static> = manager
        .method_call(
            "org.freedesktop.NetworkManager",
            "CheckpointCreate",
            (vec![device], 45u32, 0x02u32),
        )
        .map(|(path,): (dbus::Path<'static>,)| path)
        .map_err(network_change_failed)?;
    *CHECKPOINT
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("checkpoint lock poisoned") = Some(checkpoint);
    Ok(())
}

pub fn finish_checkpoint(commit: bool) -> Result<(), OperationFailure> {
    let checkpoint = CHECKPOINT
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("checkpoint lock poisoned")
        .take();
    let Some(checkpoint) = checkpoint else {
        return Ok(());
    };
    let connection = Connection::new_system().map_err(|_| dbus_unavailable())?;
    let manager = connection.with_proxy(NM, ROOT, Duration::from_secs(30));
    if commit {
        manager
            .method_call::<(), _, _, _>(
                "org.freedesktop.NetworkManager",
                "CheckpointDestroy",
                (checkpoint,),
            )
            .map_err(network_change_failed)
    } else {
        manager
            .method_call::<(HashMap<dbus::Path<'static>, u32>,), _, _, _>(
                "org.freedesktop.NetworkManager",
                "CheckpointRollback",
                (checkpoint,),
            )
            .map(|_| ())
            .map_err(network_change_failed)
    }
}

pub fn reconcile_managed(interface_name: &str) -> Result<(), OperationFailure> {
    let connection = Connection::new_system().map_err(|_| dbus_unavailable())?;
    let timeout = Duration::from_secs(30);
    let settings_proxy =
        connection.with_proxy(NM, "/org/freedesktop/NetworkManager/Settings", timeout);
    let settings = managed_settings(interface_name, true);
    let profile: dbus::Path<'static> = match settings_proxy
        .method_call(
            "org.freedesktop.NetworkManager.Settings",
            "GetConnectionByUuid",
            (MANAGED_UUID,),
        )
        .map(|(path,): (dbus::Path<'static>,)| path)
    {
        Ok(path) => {
            connection
                .with_proxy(NM, path.clone(), timeout)
                .method_call::<(), _, _, _>(
                    "org.freedesktop.NetworkManager.Settings.Connection",
                    "Update",
                    (settings,),
                )
                .map_err(network_change_failed)?;
            path
        }
        Err(_) => settings_proxy
            .method_call(
                "org.freedesktop.NetworkManager.Settings",
                "AddConnection",
                (settings,),
            )
            .map(|(path,): (dbus::Path<'static>,)| path)
            .map_err(network_change_failed)?,
    };
    let manager = connection.with_proxy(NM, ROOT, timeout);
    let device: dbus::Path<'static> = manager
        .method_call(
            "org.freedesktop.NetworkManager",
            "GetDeviceByIpIface",
            (interface_name,),
        )
        .map(|(path,): (dbus::Path<'static>,)| path)
        .map_err(network_change_failed)?;
    manager
        .method_call::<(dbus::Path<'static>,), _, _, _>(
            "org.freedesktop.NetworkManager",
            "ActivateConnection",
            (profile, device, dbus::Path::from("/")),
        )
        .map(|_| ())
        .map_err(network_change_failed)
}

pub fn prepare_bootstrap(interface_name: &str) -> Result<(), OperationFailure> {
    let connection = Connection::new_system().map_err(|_| dbus_unavailable())?;
    let timeout = Duration::from_secs(30);
    let manager = connection.with_proxy(NM, ROOT, timeout);
    let device: dbus::Path<'static> = manager
        .method_call(
            "org.freedesktop.NetworkManager",
            "GetDeviceByIpIface",
            (interface_name,),
        )
        .map(|(path,): (dbus::Path<'static>,)| path)
        .map_err(network_change_failed)?;
    let mut options = PropMap::new();
    insert(&mut options, "persist", "volatile".to_owned());
    let (active, profile, _result): (dbus::Path<'static>, dbus::Path<'static>, PropMap) = manager
        .method_call::<(dbus::Path<'static>, dbus::Path<'static>, PropMap), _, _, _>(
            "org.freedesktop.NetworkManager",
            "AddAndActivateConnection2",
            (
                bootstrap_settings(interface_name),
                device,
                dbus::Path::from("/"),
                options,
            ),
        )
        .map_err(network_change_failed)?;
    *BOOTSTRAP
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("bootstrap lock poisoned") = Some((active, profile));
    Ok(())
}

pub fn finish_bootstrap(remove: bool) -> Result<(), OperationFailure> {
    let bootstrap = BOOTSTRAP
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("bootstrap lock poisoned")
        .take();
    let Some((active, profile)) = bootstrap else {
        return Ok(());
    };
    if !remove {
        return Ok(());
    }
    let connection = Connection::new_system().map_err(|_| dbus_unavailable())?;
    let timeout = Duration::from_secs(30);
    let manager = connection.with_proxy(NM, ROOT, timeout);
    let _ = manager.method_call::<(), _, _, _>(
        "org.freedesktop.NetworkManager",
        "DeactivateConnection",
        (active,),
    );
    connection
        .with_proxy(NM, profile, timeout)
        .method_call::<(), _, _, _>(
            "org.freedesktop.NetworkManager.Settings.Connection",
            "Delete",
            (),
        )
        .map_err(network_change_failed)
}

pub fn disable_managed(interface_name: &str) -> Result<(), OperationFailure> {
    let connection = Connection::new_system().map_err(|_| dbus_unavailable())?;
    let timeout = Duration::from_secs(30);
    let profile = managed_profile(&connection, timeout)?;
    connection
        .with_proxy(NM, profile, timeout)
        .method_call::<(), _, _, _>(
            "org.freedesktop.NetworkManager.Settings.Connection",
            "Update",
            (managed_settings(interface_name, false),),
        )
        .map_err(network_change_failed)?;
    deactivate_managed(&connection, interface_name, timeout)
}

pub fn remove_managed(interface_name: &str) -> Result<(), OperationFailure> {
    let connection = Connection::new_system().map_err(|_| dbus_unavailable())?;
    let timeout = Duration::from_secs(30);
    deactivate_managed(&connection, interface_name, timeout)?;
    let profile = managed_profile(&connection, timeout)?;
    connection
        .with_proxy(NM, profile, timeout)
        .method_call::<(), _, _, _>(
            "org.freedesktop.NetworkManager.Settings.Connection",
            "Delete",
            (),
        )
        .map_err(network_change_failed)
}

fn managed_profile(
    connection: &Connection,
    timeout: Duration,
) -> Result<dbus::Path<'static>, OperationFailure> {
    connection
        .with_proxy(NM, "/org/freedesktop/NetworkManager/Settings", timeout)
        .method_call(
            "org.freedesktop.NetworkManager.Settings",
            "GetConnectionByUuid",
            (MANAGED_UUID,),
        )
        .map(|(path,): (dbus::Path<'static>,)| path)
        .map_err(|_| {
            OperationFailure::safe(
                "managed_profile_missing",
                "The managed sharing profile no longer exists.",
                "Inspect NetworkManager state before trying another change.",
            )
        })
}

fn deactivate_managed(
    connection: &Connection,
    interface_name: &str,
    timeout: Duration,
) -> Result<(), OperationFailure> {
    let manager = connection.with_proxy(NM, ROOT, timeout);
    let device: dbus::Path<'static> = manager
        .method_call(
            "org.freedesktop.NetworkManager",
            "GetDeviceByIpIface",
            (interface_name,),
        )
        .map(|(path,): (dbus::Path<'static>,)| path)
        .map_err(network_change_failed)?;
    let active: dbus::Path<'static> = connection
        .with_proxy(NM, device, timeout)
        .get("org.freedesktop.NetworkManager.Device", "ActiveConnection")
        .unwrap_or_else(|_| dbus::Path::from("/"));
    if active == "/" {
        return Ok(());
    }
    let active_proxy = connection.with_proxy(NM, active.clone(), timeout);
    let settings_path: dbus::Path<'static> = active_proxy
        .get(
            "org.freedesktop.NetworkManager.Connection.Active",
            "Connection",
        )
        .map_err(network_change_failed)?;
    let settings: HashMap<String, PropMap> = connection
        .with_proxy(NM, settings_path, timeout)
        .method_call(
            "org.freedesktop.NetworkManager.Settings.Connection",
            "GetSettings",
            (),
        )
        .map(|(settings,): (HashMap<String, PropMap>,)| settings)
        .map_err(network_change_failed)?;
    if profile_kind_for_uuid(setting_string(&settings, "connection", "uuid").as_deref())
        != ProfileKind::Managed
    {
        return Ok(());
    }
    manager
        .method_call::<(), _, _, _>(
            "org.freedesktop.NetworkManager",
            "DeactivateConnection",
            (active,),
        )
        .map_err(network_change_failed)
}

fn managed_settings(interface_name: &str, autoconnect: bool) -> HashMap<String, PropMap> {
    let mut connection = PropMap::new();
    insert(&mut connection, "id", MANAGED_ID.to_owned());
    insert(&mut connection, "uuid", MANAGED_UUID.to_owned());
    insert(&mut connection, "type", "802-3-ethernet".to_owned());
    insert(&mut connection, "interface-name", interface_name.to_owned());
    insert(&mut connection, "autoconnect", autoconnect);

    let mut address = PropMap::new();
    insert(&mut address, "address", "192.168.137.1".to_owned());
    insert(&mut address, "prefix", 24u32);
    let mut ipv4 = PropMap::new();
    insert(&mut ipv4, "method", "shared".to_owned());
    insert(&mut ipv4, "address-data", vec![address]);
    insert(&mut ipv4, "never-default", true);

    let mut ipv6 = PropMap::new();
    insert(&mut ipv6, "method", "disabled".to_owned());

    HashMap::from([
        ("connection".into(), connection),
        ("802-3-ethernet".into(), PropMap::new()),
        ("ipv4".into(), ipv4),
        ("ipv6".into(), ipv6),
    ])
}

fn bootstrap_settings(interface_name: &str) -> HashMap<String, PropMap> {
    let mut connection = PropMap::new();
    insert(
        &mut connection,
        "id",
        "Puppis S1 Manager bootstrap".to_owned(),
    );
    insert(&mut connection, "type", "802-3-ethernet".to_owned());
    insert(&mut connection, "interface-name", interface_name.to_owned());
    insert(&mut connection, "autoconnect", false);
    let mut address = PropMap::new();
    insert(&mut address, "address", "192.168.137.1".to_owned());
    insert(&mut address, "prefix", 24u32);
    let mut ipv4 = PropMap::new();
    insert(&mut ipv4, "method", "manual".to_owned());
    insert(&mut ipv4, "address-data", vec![address]);
    insert(&mut ipv4, "never-default", true);
    let mut ipv6 = PropMap::new();
    insert(&mut ipv6, "method", "disabled".to_owned());
    HashMap::from([
        ("connection".into(), connection),
        ("802-3-ethernet".into(), PropMap::new()),
        ("ipv4".into(), ipv4),
        ("ipv6".into(), ipv6),
    ])
}

fn insert<T: RefArg + 'static>(map: &mut PropMap, key: &str, value: T) {
    map.insert(key.into(), Variant(Box::new(value)));
}

fn network_change_failed(error: dbus::Error) -> OperationFailure {
    let denied = error
        .name()
        .is_some_and(|name| name.contains("NotAuthorized") || name.contains("Cancelled"));
    if denied {
        OperationFailure::safe(
            "network_authorization_denied",
            "NetworkManager authorization was denied.",
            "Keep the current network state or try again when authorization is available.",
        )
    } else {
        OperationFailure::safe(
            "network_change_failed",
            "NetworkManager could not complete the requested sharing change.",
            "Inspect the current network state and diagnostics before trying again.",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_stable_uuid_proves_profile_ownership() {
        assert_eq!(
            profile_kind_for_uuid(Some(MANAGED_UUID)),
            ProfileKind::Managed
        );
        assert_eq!(
            profile_kind_for_uuid(Some("external-profile-with-the-same-display-name")),
            ProfileKind::External
        );
        assert_eq!(profile_kind_for_uuid(None), ProfileKind::External);
    }
}
