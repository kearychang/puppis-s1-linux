use crate::OperationFailure;
use serde::Serialize;
use std::path::PathBuf;

const PRODUCTION_LIMIT: usize = 5 * 1024 * 1024;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SafeEvent {
    ApplicationStarted,
    PuppisVerified,
    ManagedSharingReconciled,
    SettingsTransactionStarted,
    SettingsTransactionVerified,
    SettingsTransactionRolledBack,
    RecoveryRequired,
}

#[derive(Debug, Clone)]
pub struct BoundedLog {
    path: PathBuf,
    limit: usize,
}

impl BoundedLog {
    pub fn with_limit(path: PathBuf, limit: usize) -> Self {
        Self { path, limit }
    }

    pub fn system_default() -> Option<Self> {
        super::marker_path().and_then(|marker| {
            marker
                .parent()
                .map(|directory| Self::with_limit(directory.join("events.jsonl"), PRODUCTION_LIMIT))
        })
    }

    pub fn record(&self, event: SafeEvent) -> Result<(), OperationFailure> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| log_failure())?;
        }
        let line =
            serde_json::to_string(&serde_json::json!({ "schemaVersion": 1, "event": event }))
                .map_err(|_| log_failure())?
                + "\n";
        let mut contents = std::fs::read_to_string(&self.path).unwrap_or_default();
        contents.push_str(&line);
        while contents.len() > self.limit {
            let Some(newline) = contents.find('\n') else {
                contents.clear();
                break;
            };
            contents.drain(..=newline);
        }
        std::fs::write(&self.path, contents).map_err(|_| log_failure())
    }
}

fn log_failure() -> OperationFailure {
    OperationFailure::safe(
        "structured_log_unavailable",
        "The private structured log could not be updated.",
        "Diagnostics remain available from current in-memory state.",
    )
}
