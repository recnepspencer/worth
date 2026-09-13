use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_offline_verifier::RecoveryObserverReport;

use super::artifacts::{artifact_identity_digest, artifact_set_digest, collect_files};
use super::{parent_oracle, ExpectedWriterHistory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParentHistoryMismatch {
    ArtifactCount,
    BytesRead,
    ArtifactSetDigest,
    ArtifactIdentityDigest,
    SemanticEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParentPhysicalHistory {
    artifact_count: u64,
    bytes_read: u64,
    artifact_set_digest: [u8; 32],
    artifact_identity_digest: [u8; 32],
    publication_artifact_set_digest: [u8; 32],
    artifacts: Box<[(String, u64, [u8; 32])]>,
    evidence: parent_oracle::ParentPhysicalEvidence,
}

impl ParentPhysicalHistory {
    pub(crate) fn capture(root: &Path, expected: &ExpectedWriterHistory) -> Result<Self, String> {
        Self::capture_with_membership(root, expected, None)
    }

    pub(crate) fn capture_with_unresolved_record(
        root: &Path,
        expected: &ExpectedWriterHistory,
    ) -> Result<Self, String> {
        let idempotency = expected
            .dirty_idempotency()
            .ok_or_else(|| "missing dirty idempotency binding".to_owned())?;
        Self::capture_with_membership(
            root,
            expected,
            Some((idempotency, expected.in_flight_payload())),
        )
    }

    fn capture_with_membership(
        root: &Path,
        expected: &ExpectedWriterHistory,
        unresolved_binding: Option<([u8; 32], &[u8])>,
    ) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|error| format!("canonicalize parent history root: {error}"))?;
        let mut files = Vec::new();
        collect_files(&root, &root, &mut files)?;
        files.sort_by(|left, right| left.0.cmp(&right.0));
        let unresolved_payload = match unresolved_binding {
            Some((idempotency, payload)) => {
                parent_oracle::require_current_root_membership_with_unresolved_payload(
                    &files,
                    expected.durable_bindings(),
                    &idempotency,
                    payload,
                )?
            }
            None => {
                parent_oracle::require_current_root_membership(
                    &files,
                    expected.durable_bindings(),
                )?;
                false
            }
        };
        let evidence = parent_oracle::derive(&files)?;
        let contents = files
            .into_iter()
            .map(|(path, bytes)| {
                let digest: [u8; 32] = Sha256::digest(&bytes).into();
                (path, bytes.len() as u64, digest)
            })
            .collect::<Vec<_>>();
        Ok(Self {
            artifact_count: contents.len() as u64,
            bytes_read: contents.iter().map(|(_, bytes, _)| *bytes).sum(),
            artifact_set_digest: artifact_set_digest(&contents),
            artifact_identity_digest: artifact_identity_digest(&contents),
            publication_artifact_set_digest: evidence.publication_digest(unresolved_payload),
            artifacts: contents.into_boxed_slice(),
            evidence,
        })
    }

    pub(crate) fn checkpoint_count(&self) -> u64 {
        self.evidence.checkpoint_count()
    }

    pub(crate) fn latest_checkpoint_sequence(&self) -> u64 {
        self.evidence.latest_checkpoint_sequence()
    }

    pub(crate) fn current_root_generation(&self) -> Option<u64> {
        self.evidence.current_root_generation()
    }

    pub(crate) fn wal_segment_count(&self) -> u64 {
        self.evidence.wal_segment_count()
    }

    pub(crate) fn publication_changed_from(&self, before: &Self) -> bool {
        self.publication_artifact_set_digest != before.publication_artifact_set_digest
    }

    pub(crate) fn changed_paths_from(&self, before: &Self) -> Vec<String> {
        let mut paths = std::collections::BTreeSet::new();
        for (path, bytes, digest) in self.artifacts.iter() {
            if before
                .artifacts
                .iter()
                .find(|(candidate, _, _)| candidate == path)
                != Some(&(path.clone(), *bytes, *digest))
            {
                paths.insert(path.clone());
            }
        }
        for (path, _, _) in before.artifacts.iter() {
            if !self
                .artifacts
                .iter()
                .any(|(candidate, _, _)| candidate == path)
            {
                paths.insert(path.clone());
            }
        }
        paths.into_iter().collect()
    }

    pub(crate) fn compare_report(
        &self,
        report: &RecoveryObserverReport,
    ) -> Result<(), ParentHistoryMismatch> {
        if report.artifact_count() != self.artifact_count {
            return Err(ParentHistoryMismatch::ArtifactCount);
        }
        if report.bytes_read() != self.bytes_read {
            return Err(ParentHistoryMismatch::BytesRead);
        }
        if report.artifact_set_digest() != self.artifact_set_digest {
            return Err(ParentHistoryMismatch::ArtifactSetDigest);
        }
        if report.artifact_identity_digest() != self.artifact_identity_digest {
            return Err(ParentHistoryMismatch::ArtifactIdentityDigest);
        }
        if !self.evidence.matches(report) {
            return Err(ParentHistoryMismatch::SemanticEvidence);
        }
        Ok(())
    }
}
