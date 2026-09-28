use super::RecordPublicationDirector;
use crate::physical_runtime::{
    PhysicalCurrentRootAdvanceOutcome, PhysicalRootNamespaceDurabilityOutcome,
    PhysicalRootReplacementOutcome, RootNamespaceDurablePhysicalMutationMembers,
    RootPublicationPreparedPhysicalMutationMembers, RootReplacedPhysicalMutationMembers,
};

impl RecordPublicationDirector {
    pub(super) fn replace_prepared_root(
        &self,
        prepared: RootPublicationPreparedPhysicalMutationMembers,
    ) -> PhysicalRootReplacementOutcome {
        let Some(runtime) = self.runtime.upgrade() else {
            return PhysicalRootReplacementOutcome::runtime_released(prepared);
        };
        crate::physical_runtime::durability::replace_root_candidate(
            prepared,
            &self.root_work,
            &runtime.health,
        )
    }

    pub(super) fn synchronize_replaced_root_namespace(
        &self,
        replaced: RootReplacedPhysicalMutationMembers,
    ) -> PhysicalRootNamespaceDurabilityOutcome {
        let Some(runtime) = self.runtime.upgrade() else {
            return PhysicalRootNamespaceDurabilityOutcome::runtime_released(replaced);
        };
        crate::physical_runtime::durability::synchronize_root_namespace(
            replaced,
            &self.root_work,
            &runtime.health,
        )
    }

    pub(super) fn advance_namespace_durable_root(
        &self,
        durable: RootNamespaceDurablePhysicalMutationMembers,
    ) -> PhysicalCurrentRootAdvanceOutcome {
        let outcome = self.root_owner.advance(durable);
        if let PhysicalCurrentRootAdvanceOutcome::Advanced(completed) = &outcome {
            for member in completed.settled_members() {
                if let Some(copy) = member.source_copy_evidence() {
                    if self
                        .observe_published_extent_copy(
                            copy.operation(),
                            completed.current_root().generation(),
                            copy.publication_lsn(),
                        )
                        .is_err()
                    {
                        // The root is already durably published. Preserve that
                        // fact while revoking further work; the copy obligation
                        // remains held for inspection rather than disappearing.
                        if let Some(runtime) = self.runtime.upgrade() {
                            runtime.health.revoke();
                        }
                    }
                }
            }
        }
        if matches!(
            outcome,
            PhysicalCurrentRootAdvanceOutcome::InspectionRequired(_)
        ) {
            if let Some(runtime) = self.runtime.upgrade() {
                runtime.health.revoke();
            }
        }
        outcome
    }
}
