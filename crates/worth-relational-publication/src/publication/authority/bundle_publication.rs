use branch::snapshots::data::SnapshotHandle;
use crate::publication::authority::PublicationAuthority;
use crate::publication::authority::PublicationPreparationAuthority;
use branch::snapshots::data::assemble_snapshot_handle;
use branch::snapshots::data::SnapshotHandleFieldsPort as _;
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::identity::data::primitives::VersionId;
use contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureOwnerPort as _;
use contracts::snapshots::data::coordinates::SnapshotReadPolicy;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

impl<'runtime> PublicationPreparationAuthority<'runtime> {
    pub(crate) fn reserve_publication_snapshot(
        &self,
        commit_reference: &RelationalCommitReceipt,
        version_id: VersionId,
    ) -> Result<
        SnapshotHandle,
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
        Ok(snapshot)
    }
}

impl<'runtime> PublicationAuthority<'runtime> {
    pub(crate) fn install_published_snapshot(
        &self,
        reserved_snapshot: SnapshotHandle,
        basis: branch::history::retention::selected_snapshot::VisibilitySnapshotBasis,
        published_snapshot_slot: branch::runtime::state::subsystems::visibility::snapshot_handles::published_capacity::PublishedSnapshotSlotReservation,
    ) {
        let snapshot_id = *reserved_snapshot.owner_snapshot_id();
        self.visibility.insert_published_handle(
            snapshot_id,
            branch::runtime::state::subsystems::visibility::snapshot_handle_binding::SnapshotHandleBinding::new(
                basis,
                *reserved_snapshot.owner_read_policy(),
            ),
        );
        published_snapshot_slot.install();
    }
}
