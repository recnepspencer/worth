use worth_relational_branch::internal_port::snapshots::data::assemble_snapshot_handle;
use worth_relational_branch::internal_port::snapshots::data::SnapshotHandleFieldsPort as _;
use worth_relational_contracts::internal_port::mvcc::publication::outcome::failure::RelationalPublicationFailureOwnerPort as _;
use worth_relational_contracts::internal_port::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativePatchEnvelopeOwnerPort as _;
use crate::history::data::canonical_commit_envelope::RelationalReplayRecord;
use crate::history::data::replay_version::ReplaySchemaVersion;
use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::VersionId;
use crate::publication::authority::artifacts::PublicationArtifacts;
use crate::publication::authority::PublicationAuthority;
use crate::publication::authority::PublicationPreparationAuthority;
use crate::publication::bundle::{PublicationBundle, PublicationStatus};
use crate::snapshots::data::{SnapshotId, SnapshotReadPolicy};

impl<'runtime> PublicationPreparationAuthority<'runtime> {
    pub(crate) fn assemble_publication_bundle(
        &self,
        commit_reference: RelationalCommitReceipt,
        version_id: VersionId,
        patch: crate::publication::patch::data::CanonicalAuthoritativePatch,
        diagnostics_summary: crate::diagnostics::data::RelationalDiagnosticArtifact,
        schema_authority: crate::schema::data::SchemaAuthoritySnapshot,
    ) -> Result<
        PublicationArtifacts,
        crate::mvcc::publication::outcome::failure::RelationalPublicationFailure,
    > {
        let snapshot_id = self.snapshot_ids.reserve_prepared().ok_or_else(|| {
            crate::mvcc::publication::outcome::failure::RelationalPublicationFailure::new(
                crate::mvcc::publication::outcome::failure::RelationalPublicationFailureKind::SnapshotIdentityExhausted,
                "snapshot identity space is exhausted",
            )
        })?;
        let snapshot = assemble_snapshot_handle(
            self.runtime_instance_id,
            commit_reference.branch_id.clone(),
            snapshot_id,
            version_id,
            SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        );
        Ok(PublicationArtifacts {
            commit: commit_reference,
            snapshot,
            diagnostics_summary,
            patch,
            schema_authority,
        })
    }
}

impl<'runtime> PublicationAuthority<'runtime> {
    pub(crate) fn publish_artifacts(
        &self,
        version_id: VersionId,
        artifacts: PublicationArtifacts,
        patch_position: crate::publication::patch::data::PatchStreamPosition,
        basis: crate::history::retention::VisibilitySnapshotBasis,
        published_snapshot_slot: crate::runtime::state::subsystems::visibility::PublishedSnapshotSlotReservation,
    ) -> SnapshotId {
        let PublicationArtifacts {
            commit,
            snapshot,
            diagnostics_summary,
            patch,
            schema_authority,
        } = artifacts;
        let patch =
            crate::publication::patch::data::PublishedAuthoritativePatchEnvelope::from_canonical(
                patch_position,
                &patch,
            );
        let replay = RelationalReplayRecord {
            schema_version: ReplaySchemaVersion(1),
            commit_id: commit.commit_id,
            version_id,
            snapshot_id: (*snapshot.owner_snapshot_id()),
            patch: patch.clone(),
            schema_authority,
        };
        let bundle = PublicationBundle {
            commit,
            snapshot,
            diagnostics_summary,
            patch,
            replay,
            status: PublicationStatus::Published,
        };
        let snapshot_id = *bundle.snapshot.owner_snapshot_id();
        self.visibility.insert_published_handle(
            snapshot_id,
            crate::runtime::state::subsystems::visibility::SnapshotHandleBinding::new(
                basis,
                *bundle.snapshot.owner_read_policy(),
            ),
        );
        published_snapshot_slot.install();
        self.push_diagnostic_artifact(bundle.diagnostics_summary.clone());
        self.publication.replace_latest_bundle(bundle);
        snapshot_id
    }
}
