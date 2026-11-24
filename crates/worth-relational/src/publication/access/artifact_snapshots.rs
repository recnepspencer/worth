use crate::capabilities::PublicationDiagnosticsSource;
use crate::publication::data::{
    PublicationArtifactSnapshot, PublicationDiagnosticsSnapshot, PublicationObservationSnapshot,
};
use worth_relational_branch::internal_port::snapshots::data::SnapshotHandleFieldsPort as _;

use super::{PublicationArtifactsAccess, PublicationDiagnosticsAccess};

impl<'runtime> PublicationArtifactsAccess<'runtime> {
    pub fn observation(&self) -> PublicationObservationSnapshot {
        let latest_bundle = self.latest_bundle();
        let latest_bundle = latest_bundle.as_deref();
        let latest_patch = latest_bundle.map(|bundle| &bundle.patch);
        let latest_replay = latest_bundle.map(|bundle| &bundle.replay);

        PublicationObservationSnapshot {
            latest_commit_id: latest_bundle.map(|bundle| bundle.commit.commit_id),
            publication_snapshot_id: latest_bundle
                .map(|bundle| *bundle.snapshot.owner_snapshot_id()),
            publication_status: latest_bundle.map(|bundle| bundle.status.clone()),
            latest_patch_position: latest_patch.map(|patch| patch.position),
            latest_patch_record_count: latest_patch
                .map(|patch| patch.authoritative_record_patches.len()),
            latest_replay_commit_id: latest_replay.map(|replay| replay.commit_id),
            latest_patch_present: latest_patch.is_some(),
            latest_replay_present: latest_replay.is_some(),
            diagnostics_artifact_count: self.runtime.publication_diagnostic_artifact_count(),
        }
    }

    pub fn snapshot(&self) -> PublicationArtifactSnapshot {
        let latest_bundle = self.latest_bundle();

        PublicationArtifactSnapshot {
            observation: self.observation(),
            latest_patch: latest_bundle.as_ref().map(|bundle| bundle.patch.clone()),
            latest_replay: latest_bundle.as_ref().map(|bundle| bundle.replay.clone()),
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
