//! Fork and detach the selected branch history together with its performed routes.
use super::{Arc, AtomicU64, HistorySubsystem, RelationalPublicationHistory};
use crate::history::data::PositionedCanonicalCommit;
use crate::runtime::fork_denial::RelationalRuntimeForkDenial;

impl RelationalPublicationHistory {
    pub(crate) fn from_branch(branch: HistorySubsystem) -> Self {
        Self {
            branch,
            canonical_publication_routes: Arc::default(),
        }
    }

    pub(crate) fn detached_publication_snapshot(
        &self,
    ) -> Result<Self, RelationalRuntimeForkDenial> {
        self.snapshot_with_routes(&self.canonical_publication_routes.performed_snapshot())
    }

    pub(crate) fn replacing_recovered_branch(
        &self,
        branch: HistorySubsystem,
    ) -> Result<Self, RelationalRuntimeForkDenial> {
        let canonical_publication_routes =
            self.detached_routes(&self.canonical_publication_routes.performed_snapshot())?;
        Ok(Self {
            branch,
            canonical_publication_routes,
        })
    }

    fn snapshot_with_routes(
        &self,
        positioned: &[Arc<PositionedCanonicalCommit>],
    ) -> Result<Self, RelationalRuntimeForkDenial> {
        let branch = self.branch.detached_owner_snapshot()?;
        let canonical_publication_routes = self.detached_routes(positioned)?;
        Ok(Self {
            branch,
            canonical_publication_routes,
        })
    }

    fn detached_routes(
        &self,
        positioned: &[Arc<PositionedCanonicalCommit>],
    ) -> Result<Arc<super::RelationalCanonicalPublicationRoutes>, RelationalRuntimeForkDenial> {
        let canonical_publication_routes = Arc::new(
            self.canonical_publication_routes
                .detached_branch_transitions(),
        );
        canonical_publication_routes.install_checkpoint_catalog(self.commit_artifacts());
        for positioned in positioned {
            let artifact = self.commit_artifact(positioned.commit.commit_id).ok_or(
                crate::runtime::fork_denial::RelationalRuntimeForkDenial::CanonicalInventoryInvalid,
            )?;
            canonical_publication_routes
                .install_recovered(
                    Arc::clone(positioned),
                    artifact.envelope_residency(),
                    artifact.roots().clone(),
                )
                .map_err(|_| {
                    crate::runtime::fork_denial::RelationalRuntimeForkDenial::CanonicalInventoryInvalid
                })?;
        }
        Ok(canonical_publication_routes)
    }

    pub(crate) fn fork_snapshot(
        &self,
    ) -> Result<Self, crate::runtime::fork_denial::RelationalRuntimeForkDenial> {
        let _fork_guard = self.canonical_publication_routes.enter_fork()?;
        if let Some(commit_id) = self
            .canonical_publication_routes
            .has_unsettled_performed_publication()
        {
            return Err(
                crate::runtime::fork_denial::RelationalRuntimeForkDenial::PerformedPublicationRequiresSettlement {
                    commit_id,
                },
            );
        }
        let positioned = self.canonical_publication_routes.performed_snapshot();
        let mut fork = self.snapshot_with_routes(&positioned)?;
        let (next_commit_id, next_version_id) = {
            let mut ledger = fork.branch.ledger.write();
            if let Some(maximum) = positioned
                .iter()
                .map(|commit| commit.envelope().commit.commit_id.0)
                .max()
            {
                ledger.next_commit_id = ledger.next_commit_id.max(maximum.checked_add(1).ok_or(
                    crate::runtime::fork_denial::RelationalRuntimeForkDenial::IdentityCapacityExhausted,
                )?);
            }
            if let Some(maximum) = positioned
                .iter()
                .map(|commit| commit.envelope().commit.version_id.0)
                .max()
            {
                ledger.next_version_id = ledger.next_version_id.max(maximum.checked_add(1).ok_or(
                    crate::runtime::fork_denial::RelationalRuntimeForkDenial::IdentityCapacityExhausted,
                )?);
            }
            (ledger.next_commit_id, ledger.next_version_id)
        };
        fork.branch.commit_identity_allocator = Arc::new(AtomicU64::new(next_commit_id));
        fork.branch.version_identity_allocator = Arc::new(AtomicU64::new(next_version_id));
        Ok(fork)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::branch::target::RelationalBranchRootDescriptor;
    use crate::history::commit::{
        original_payload::RelationalOriginalRootCommitEvidence, RelationalCommitEnvelopeResidency,
    };
    use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update_for_test as envelope_with_patch_and_update;
    use crate::identity::data::{EntityId, PartitionId};
    use crate::publication::patch::data::PatchStreamPosition;
    use crate::runtime::state::subsystems::history::canonical_publication_routes::position_admission::readmit_positioned_canonical_commit;
    use crate::transactions::data::primitives::RecordRef;

    #[test]
    fn fork_refuses_a_missing_performed_artifact_without_reconstructing_inventory() {
        let host = worth_harness::facade::execution_fixture::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let history = RelationalPublicationHistory::from_branch(
            crate::branch::basis_fixture::installed_history_for_test(1),
        );
        let entity = EntityId::new(PartitionId::main(), 1, 1);
        let envelope = Arc::new(envelope_with_patch_and_update(
            RecordRef::Entity(entity),
            entity,
        ));
        let positioned =
            readmit_positioned_canonical_commit(PatchStreamPosition(1), Arc::clone(&envelope))
                .unwrap();
        let commit_id = positioned.commit.commit_id;
        let original = RelationalOriginalRootCommitEvidence::prepare(&execution, envelope).unwrap();
        history
            .canonical_publication_routes
            .install_recovered(
                Arc::new(positioned),
                Arc::new(RelationalCommitEnvelopeResidency::resident(original)),
                RelationalBranchRootDescriptor::new([4; 32], [8; 32]),
            )
            .unwrap();
        assert!(history.positioned_canonical_commit(commit_id).is_some());
        assert!(history.commit_artifact(commit_id).is_none());
        assert!(matches!(
            history.fork_snapshot(),
            Err(RelationalRuntimeForkDenial::CanonicalInventoryInvalid)
        ));
    }
}
