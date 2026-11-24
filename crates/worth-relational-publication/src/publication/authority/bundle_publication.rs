use crate::publication::authority::artifacts::PublicationArtifacts;
use crate::publication::authority::PublicationAuthority;
use crate::publication::authority::PublicationPreparationAuthority;
use crate::publication::bundle::PublicationBundle;
use branch::history::data::canonical_commit_envelope::replay_record::RelationalReplayRecord;
use branch::snapshots::data::assemble_snapshot_handle;
use branch::snapshots::data::SnapshotHandleFieldsPort as _;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::replay_version::ReplaySchemaVersion;
use contracts::identity::data::primitives::VersionId;
use contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureOwnerPort as _;
use contracts::publication::bundle::publication_status::PublicationStatus;
use contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativePatchEnvelopeOwnerPort as _;
use contracts::snapshots::data::coordinates::SnapshotId;
use contracts::snapshots::data::coordinates::SnapshotReadPolicy;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

impl<'runtime> PublicationPreparationAuthority<'runtime> {
    pub(crate) fn assemble_publication_bundle(
        &self,
        commit_reference: RelationalCommitReceipt,
        version_id: VersionId,
        patch: contracts::publication::patch::data::authoritative_patch_envelope::CanonicalAuthoritativePatch,
        diagnostics_summary: contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact,
        schema_authority: contracts::schema::data::authority_snapshot::carried_projection::SchemaAuthoritySnapshot,
    ) -> Result<
        PublicationArtifacts,
        contracts::mvcc::publication::outcome::failure::RelationalPublicationFailure,
    > {
        let snapshot_id = self.snapshot_ids.reserve_prepared().ok_or_else(|| {
            contracts::mvcc::publication::outcome::failure::RelationalPublicationFailure::new(
                contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureKind::SnapshotIdentityExhausted,
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
        patch_position: contracts::publication::patch::data::patch_position::PatchStreamPosition,
        basis: branch::history::retention::selected_snapshot::VisibilitySnapshotBasis,
        published_snapshot_slot: branch::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotSlotReservation,
    ) -> SnapshotId {
        let PublicationArtifacts {
            commit,
            snapshot,
            diagnostics_summary,
            patch,
            schema_authority,
        } = artifacts;
        let patch =
            contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativePatchEnvelope::from_canonical(
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
            branch::runtime::state::subsystems::visibility::snapshot_handle_binding::SnapshotHandleBinding::new(
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
