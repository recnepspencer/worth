use crate::identity::data::{EntityId, KindId, VersionId};
use crate::storage::overlay::PartitionAccess;
use crate::storage::partition::AdjacencyDirection;

use super::VisibilityProjectionView;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalAdjacencyDirection {
    Outgoing,
    Incoming,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdjacencyStructuralRevision {
    revision: Option<VersionId>,
    work_units: usize,
}

/// Why a bounded structural-revision read gave no answer. Each cause is
/// distinct so a caller can tell an unpaid read from a missing anchor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdjacencyStructuralRevisionDenial {
    /// The caller's maximum cannot pay the one owner-index lookup. Nothing
    /// was read.
    WorkBudgetExceeded,
    /// The anchor is not visible at this snapshot.
    AnchorUnavailable,
    /// The snapshot's basis retains no adjacency index to read.
    BasisUnavailable,
}

impl AdjacencyStructuralRevision {
    pub const fn revision(self) -> Option<VersionId> {
        self.revision
    }

    pub const fn work_units(self) -> usize {
        self.work_units
    }
}

impl VisibilityProjectionView<'_> {
    pub fn bounded_adjacency_structural_revision(
        &self,
        anchor: EntityId,
        relation_kind: KindId,
        direction: RelationalAdjacencyDirection,
        maximum_work: usize,
    ) -> Result<AdjacencyStructuralRevision, AdjacencyStructuralRevisionDenial> {
        if maximum_work < 1 {
            return Err(AdjacencyStructuralRevisionDenial::WorkBudgetExceeded);
        }
        if self
            .entity_record_with_projection_scope(
                anchor,
                super::ProjectionAspectScope::empty(),
                |_| Some(()),
            )
            .is_none()
        {
            return Err(AdjacencyStructuralRevisionDenial::AnchorUnavailable);
        }
        let direction = match direction {
            RelationalAdjacencyDirection::Outgoing => AdjacencyDirection::Outgoing,
            RelationalAdjacencyDirection::Incoming => AdjacencyDirection::Incoming,
        };
        let Some(root) = self.basis.root() else {
            return Err(AdjacencyStructuralRevisionDenial::BasisUnavailable);
        };
        let revision = root
            .get_partition(anchor.partition_id)
            .and_then(|partition| direction.table(partition).get(anchor.slot_index()))
            .and_then(|adjacency| adjacency.structural_revision(relation_kind));
        Ok(AdjacencyStructuralRevision {
            revision,
            work_units: 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::durability::RecoveryVerificationMode;
    use crate::facade::history::BranchId;
    use crate::tests::support::*;

    #[test]
    fn selected_basis_adjacency_revision_detects_add_remove_aba() {
        let runtime = runtime_with_test_schema();
        let source_commit = create_entity_outcome(&runtime, "source");
        let source = changed_entities(&source_commit)[0];
        let target_commit = create_entity_outcome(&runtime, "target");
        let target = changed_entities(&target_commit)[0];
        let relation_commit = create_relation_outcome(&runtime, source, target, "edge");
        let relation = changed_relations(&relation_commit)[0];
        let deletion_commit =
            delete_relation_on_branch(&runtime, relation, BranchId("main".to_owned()));

        let revision_at = |snapshot| {
            let view = runtime
                .read_truth()
                .project_snapshot(snapshot)
                .expect("commit snapshot remains readable");
            let revision = view
                .bounded_adjacency_structural_revision(
                    source,
                    KindId(2),
                    RelationalAdjacencyDirection::Outgoing,
                    1,
                )
                .expect("the selected-basis revision is one owner-index lookup")
                .revision();
            assert_eq!(
                view.exact_adjacency_structural_revision(
                    source,
                    KindId(2),
                    RelationalAdjacencyDirection::Outgoing,
                )
                .unwrap(),
                revision
            );
            revision
        };
        assert_eq!(revision_at(&source_commit.snapshot), None);
        assert_eq!(
            revision_at(&relation_commit.snapshot),
            Some(relation_commit.version_id)
        );
        assert_eq!(
            revision_at(&deletion_commit.snapshot),
            Some(deletion_commit.version_id)
        );
        let deleted_view = runtime
            .read_truth()
            .project_snapshot(&deletion_commit.snapshot)
            .expect("deleted-relation snapshot remains readable");
        assert_eq!(
            deleted_view.bounded_adjacency_structural_revision(
                source,
                KindId(2),
                RelationalAdjacencyDirection::Outgoing,
                0,
            ),
            Err(AdjacencyStructuralRevisionDenial::WorkBudgetExceeded)
        );
        let missing_anchor = EntityId::new(
            source.partition_id,
            source.local_slot_value() + 1_000,
            source.generation_value(),
        );
        assert_eq!(
            deleted_view.bounded_adjacency_structural_revision(
                missing_anchor,
                KindId(2),
                RelationalAdjacencyDirection::Outgoing,
                1,
            ),
            Err(AdjacencyStructuralRevisionDenial::AnchorUnavailable)
        );
        // An unpaid read refuses before it reads the anchor.
        assert_eq!(
            deleted_view.bounded_adjacency_structural_revision(
                missing_anchor,
                KindId(2),
                RelationalAdjacencyDirection::Outgoing,
                0,
            ),
            Err(AdjacencyStructuralRevisionDenial::WorkBudgetExceeded)
        );

        runtime
            .durability_authority()
            .checkpoint()
            .expect("checkpoint captures the structural revision");
        let recovery = runtime
            .durability()
            .recovery_plan(RecoveryVerificationMode::NormalRecoveryVerification);
        let mut recovered = runtime_with_test_schema();
        recovered
            .durability_recovery()
            .recover(recovery)
            .expect("checkpoint recovery rebuilds adjacency indexes");
        let recovered_snapshot =
            snapshot_for_owner_branch(&recovered, &BranchId("main".to_owned()));
        assert_eq!(
            recovered
                .read_truth()
                .project_snapshot(&recovered_snapshot)
                .expect("recovered main basis is readable")
                .bounded_adjacency_structural_revision(
                    source,
                    KindId(2),
                    RelationalAdjacencyDirection::Outgoing,
                    1,
                )
                .expect("recovered revision remains one owner-index lookup")
                .revision(),
            Some(deletion_commit.version_id)
        );
        recovered
            .snapshots()
            .release_snapshot(&recovered_snapshot)
            .expect("recovered snapshot releases");

        for outcome in [
            &source_commit,
            &target_commit,
            &relation_commit,
            &deletion_commit,
        ] {
            release_test_commit_snapshot(&runtime, outcome);
        }
    }
}
