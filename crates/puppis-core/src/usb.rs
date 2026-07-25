use crate::{OperationFailure, PuppisCandidate, UsbLinkSpeed};
use std::fs;
use std::path::{Path, PathBuf};

pub fn discover_linux_candidates() -> Result<Vec<PuppisCandidate>, OperationFailure> {
    discover_candidates_at(Path::new("/sys/class/net"))
}

pub fn discover_candidates_at(root: &Path) -> Result<Vec<PuppisCandidate>, OperationFailure> {
    let entries = fs::read_dir(root).map_err(|_| {
        OperationFailure::safe(
            "usb_discovery_unavailable",
            "USB network discovery is unavailable.",
            "Check that the system device filesystem is mounted, then refresh.",
        )
    })?;
    let mut candidates = Vec::new();
    for entry in entries.flatten() {
        let interface_name = entry.file_name().to_string_lossy().into_owned();
        let device_link = entry.path().join("device");
        let Ok(device_path) = device_link.canonicalize().or_else(|_| {
            if device_link.is_dir() {
                Ok(device_link.clone())
            } else {
                Err(std::io::Error::from(std::io::ErrorKind::NotFound))
            }
        }) else {
            continue;
        };
        let Some(usb_device) = find_usb_device(&device_path) else {
            continue;
        };
        let vendor = read_trimmed(usb_device.join("idVendor"));
        if vendor.as_deref() != Some("0b95") {
            continue;
        }
        let product =
            read_trimmed(usb_device.join("idProduct")).unwrap_or_else(|| "unknown".into());
        let link_speed = match read_trimmed(usb_device.join("speed")).as_deref() {
            Some("5000" | "10000" | "20000") => UsbLinkSpeed::SuperSpeed,
            Some("1.5" | "12" | "480") => UsbLinkSpeed::Usb2,
            _ => UsbLinkSpeed::Unknown,
        };
        candidates.push(PuppisCandidate {
            id: format!("usb:{interface_name}:{product}"),
            display_name: format!("USB network adapter ({interface_name})"),
            interface_name,
            link_speed,
        });
    }
    candidates.sort_by(|left, right| left.interface_name.cmp(&right.interface_name));
    Ok(candidates)
}

fn find_usb_device(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|ancestor| ancestor.join("idVendor").is_file())
        .map(Path::to_path_buf)
}

fn read_trimmed(path: PathBuf) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_owned())
}
