use crate::{ApplicationSnapshot, OperationFailure};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsBundle {
    pub preview: String,
    pub suggested_name: String,
}

pub fn preview(snapshot: &ApplicationSnapshot) -> Result<DiagnosticsBundle, OperationFailure> {
    let candidates: Vec<_> = snapshot
        .candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            json!({
                "alias": format!("candidate-{}", index + 1),
                "usbLinkSpeed": candidate.link_speed,
            })
        })
        .collect();
    let verified = snapshot.verified_puppis.as_ref().map(|identity| {
        json!({
            "model": identity.model,
            "firmware": identity.firmware,
            "role": identity.role_code,
            "mutationsQualified": snapshot.mutations_qualified,
        })
    });
    let host_sharing = snapshot.host_sharing.as_ref().map(|sharing| {
        json!({
            "upstreamAvailable": sharing.upstream_available,
            "profileKind": sharing.profile_kind,
            "active": sharing.active,
            "ipv4Shared": sharing.ipv4_shared,
            "downstreamAddress": sharing.downstream_address,
            "ipv6Disabled": sharing.ipv6_disabled,
            "autoconnect": sharing.autoconnect,
            "subnetConflict": sharing.subnet_conflict,
            "authorization": sharing.authorization,
        })
    });
    let radios: Vec<_> = snapshot
        .radios
        .iter()
        .map(|radio| {
            json!({
                "band": radio.band,
                "channel": radio.channel,
                "country": radio.country,
                "enabled": radio.enabled,
                "bandwidth": radio.bandwidth,
                "ssidMutation": radio.ssid_mutation,
                "passwordMutation": radio.password_mutation,
                "qualifiedChannels": radio.qualified_channels,
                "passwordCapabilityReason": radio.password_unavailable_reason,
            })
        })
        .collect();
    let value = json!({
        "schemaVersion": 1,
        "application": {
            "name": "Puppis S1 Manager for Linux",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "stateRevision": snapshot.revision,
        "status": {
            "usb": snapshot.usb,
            "sharing": snapshot.sharing,
            "protocol": snapshot.protocol,
            "configuration": snapshot.configuration,
            "clients": snapshot.clients,
        },
        "candidates": candidates,
        "verifiedPuppis": verified,
        "hostSharing": host_sharing,
        "radios": radios,
        "deviceRole": snapshot.device_role,
        "recoveryRequired": snapshot.recovery_required,
        "activeOperation": snapshot.active_operation,
        "lastFailure": snapshot.last_failure,
        "clientEvidence": snapshot.client_evidence,
    });
    let preview = serde_json::to_string_pretty(&value).map_err(|_| {
        OperationFailure::safe(
            "diagnostics_generation_failed",
            "The diagnostics preview could not be generated.",
            "Refresh application state and try again.",
        )
    })?;
    Ok(DiagnosticsBundle {
        preview,
        suggested_name: "puppis-s1-diagnostics.json".into(),
    })
}
