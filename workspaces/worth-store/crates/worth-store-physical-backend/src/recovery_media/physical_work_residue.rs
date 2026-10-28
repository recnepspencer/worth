//! Physical-work obligation records a killed runtime left behind.
//!
//! Records live under `families/physical-work` as
//! `effect-<runtime>-<generation>-<operation>.pending`. Recovery reads them
//! bounded and discharges the ones it has adjudicated; any other name in the
//! directory is damaged evidence and is never touched.

use crate::filesystem_media::{ArtifactTreeDirectory, ArtifactTreeFailure, ArtifactTreeFile};

use super::AdmittedRecoveryFilesystemMedia;

/// One obligation record: its file name and its exact bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPhysicalWorkRecord {
    name: String,
    bytes: Box<[u8]>,
}

impl RecoveryPhysicalWorkRecord {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl AdmittedRecoveryFilesystemMedia {
    /// Reads at most `limit` obligation records of at most `record_bytes`
    /// each. A non-record name or an oversized record refuses.
    pub fn list_recovery_physical_work_records(
        &self,
        limit: usize,
        record_bytes: u64,
    ) -> Result<Box<[RecoveryPhysicalWorkRecord]>, ArtifactTreeFailure> {
        let directory = journal()?;
        let tree = self.parts.artifact_tree();
        if !tree.directory_exists(&directory)? {
            return Ok(Box::new([]));
        }
        tree.list_file_names_bounded(&directory, limit)?
            .iter()
            .map(|name| {
                let file = record(&directory, name)?;
                let bytes = tree.read_bounded(&file, record_bytes)?;
                Ok(RecoveryPhysicalWorkRecord {
                    name: name.clone(),
                    bytes: bytes.into(),
                })
            })
            .collect()
    }

    /// Durably removes one adjudicated obligation record. An already absent
    /// record completes, so an interrupted discharge can be repeated.
    pub fn discharge_recovery_physical_work_record(
        &self,
        name: &str,
    ) -> Result<(), ArtifactTreeFailure> {
        let directory = journal()?;
        self.parts
            .artifact_tree()
            .remove_file_durably(&record(&directory, name)?)
    }
}

fn journal() -> Result<ArtifactTreeDirectory, ArtifactTreeFailure> {
    ArtifactTreeDirectory::families()
        .child("physical-work")
        .map_err(|_| ArtifactTreeFailure::recovery_denial())
}

fn record(
    directory: &ArtifactTreeDirectory,
    name: &str,
) -> Result<ArtifactTreeFile, ArtifactTreeFailure> {
    if !is_record_name(name) {
        return Err(ArtifactTreeFailure::recovery_damaged());
    }
    directory
        .file(name)
        .map_err(|_| ArtifactTreeFailure::recovery_damaged())
}

fn is_record_name(name: &str) -> bool {
    let Some(fields) = name
        .strip_prefix("effect-")
        .and_then(|rest| rest.strip_suffix(".pending"))
    else {
        return false;
    };
    let parts: Vec<&str> = fields.split('-').collect();
    parts.len() == 3
        && parts.iter().all(|part| {
            part.len() == 16
                && part
                    .bytes()
                    .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        })
}

#[cfg(test)]
mod tests {
    use super::is_record_name;

    #[test]
    fn only_exact_obligation_names_are_records() {
        assert!(is_record_name(
            "effect-0000000000000001-0000000000000002-00000000000000ff.pending"
        ));
        for name in [
            "effect-1-2-3.pending",
            "effect-0000000000000001-0000000000000002.pending",
            "effect-0000000000000001-0000000000000002-00000000000000FF.pending",
            "effect-0000000000000001-0000000000000002-00000000000000ff.done",
            "other.pending",
        ] {
            assert!(!is_record_name(name), "{name}");
        }
    }
}
