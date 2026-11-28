use crate::capabilities::PublicationDiagnosticsSource;
use crate::publication::data::{
    PublicationDiagnosticsSnapshot, PublicationObservationSnapshot,
};

use super::{PublicationArtifactsAccess, PublicationDiagnosticsAccess};

impl<'runtime> PublicationArtifactsAccess<'runtime> {
    pub fn observation(&self) -> PublicationObservationSnapshot {
        let runtime = self.runtime;
        let route = runtime.history.latest_canonical_patch_route();
        let envelope = route.and_then(|(position, _)| runtime.history.canonical_envelope_at_patch(position));
        let commit = envelope.as_ref().map(|envelope| &envelope.commit);
        let snapshot_id = commit
            .and_then(|commit| runtime.visibility.published_snapshot_binding_for_version(commit.version_id))
            .map(|(snapshot_id, _)| snapshot_id);
        PublicationObservationSnapshot {
            latest_commit_id: commit.map(|commit| commit.commit_id),
            publication_snapshot_id: snapshot_id,
            publication_status: commit.map(|_| crate::publication::bundle::PublicationStatus::Published),
            latest_patch_position: route.map(|(position, _)| position),
            latest_patch_record_count: envelope.as_ref().map(|envelope| envelope.patch.authoritative_record_patches.len()),
            latest_replay_commit_id: commit.map(|commit| commit.commit_id),
            latest_patch_present: route.is_some(),
            latest_replay_present: envelope.is_some(),
            diagnostics_artifact_count: runtime.publication_diagnostic_artifact_count(),
        }
    }
}

impl<'runtime> PublicationDiagnosticsAccess<'runtime> {
    pub fn snapshot(&self) -> PublicationDiagnosticsSnapshot {
        PublicationDiagnosticsSnapshot {
            observation: self.runtime.publication_access().artifacts().observation(),
            diagnostics: self.artifacts().to_vec(),
        }
    }
}
