use crate::{OperationFailure, SavedClient};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedClientFile {
    schema_version: u32,
    clients: Vec<SavedClient>,
}

pub fn load(path: &Path) -> Result<Vec<SavedClient>, OperationFailure> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err(storage_failure()),
    };
    let state: SavedClientFile = serde_json::from_str(&contents).map_err(|_| {
        OperationFailure::safe(
            "saved_clients_corrupt",
            "Saved client recognition data could not be read.",
            "Reset saved clients to discard the unreadable file, or preserve it for inspection.",
        )
    })?;
    if state.schema_version != SCHEMA_VERSION {
        return Err(OperationFailure::safe(
            "saved_clients_version_unsupported",
            "Saved client recognition data uses an unsupported version.",
            "Upgrade the manager or reset saved clients to start with an empty file.",
        ));
    }
    Ok(state.clients)
}

pub fn save(path: &Path, clients: &[SavedClient]) -> Result<(), OperationFailure> {
    let parent = path.parent().ok_or_else(storage_failure)?;
    let parent_existed = parent.exists();
    fs::create_dir_all(parent).map_err(|_| storage_failure())?;
    if !parent_existed {
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
            .map_err(|_| storage_failure())?;
    }
    let temporary = temporary_path(path);
    let serialized = serde_json::to_vec_pretty(&SavedClientFile {
        schema_version: SCHEMA_VERSION,
        clients: clients.to_vec(),
    })
    .map_err(|_| storage_failure())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|_| storage_failure())?;
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|_| storage_failure())?;
    file.write_all(&serialized).map_err(|_| storage_failure())?;
    file.sync_all().map_err(|_| storage_failure())?;
    fs::rename(&temporary, path).map_err(|_| storage_failure())?;
    Ok(())
}

pub fn remove(path: &Path) -> Result<(), OperationFailure> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(storage_failure()),
    }
}

pub fn system_path() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("XDG_STATE_HOME") {
        return Some(PathBuf::from(root).join("puppis-s1-manager/saved-clients.json"));
    }
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".local/state/puppis-s1-manager/saved-clients.json"))
}

fn temporary_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("saved-clients.json");
    path.with_file_name(format!(".{name}.tmp-{}", std::process::id()))
}

fn storage_failure() -> OperationFailure {
    OperationFailure::safe(
        "saved_clients_storage_failed",
        "Saved client recognition data could not be updated.",
        "Check the user state directory permissions and try again.",
    )
}
