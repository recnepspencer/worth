#[cfg(test)]
mod tests {
    use super::super::adjacency_revision_read::*;
    use crate::facade::durability::RecoveryVerificationMode;
    use crate::facade::history::BranchId;
    use crate::identity::data::{EntityId, KindId};
    use crate::tests::support::*;

    #[test]
    fn selected_basis_adjacency_revision_detects_add_remove_aba() {
        let host_request = crate::execution::test_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let runtime = runtime_with_test_schema();
        let source_commit = create_entity_outcome(execution, &runtime, "source");
        let source = changed_entities(&source_commit)[0];
        let target_commit = create_entity_outcome(execution, &runtime, "target");
        let target = changed_entities(&target_commit)[0];
        let relation_commit = create_relation_outcome(execution, &runtime, source, target, "edge");
        let relation = changed_relations(&relation_commit)[0];
        let deletion_commit =
            delete_relation_on_branch(execution, &runtime, relation, BranchId("main".to_owned()));

        let revision_at = |snapshot| {
            let view = runtime
                .read_truth()
                .project_snapshot(snapshot)
                .expect("commit snapshot remains readable");
            let revision = view
                .bounded_adjacency_structural_revision(
                    execution,
                    source,
                    KindId(2),
                    RelationalAdjacencyDirection::Outgoing,
                    1,
                )
                .expect("the selected-basis revision is one owner-index lookup")
                .revision();
            assert_eq!(
                view.read_exact_adjacency_structural_revision(
                    execution,
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
                execution,
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
                execution,
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
                execution,
                missing_anchor,
                KindId(2),
                RelationalAdjacencyDirection::Outgoing,
                0,
            ),
            Err(AdjacencyStructuralRevisionDenial::WorkBudgetExceeded)
        );

        runtime
            .durability_authority()
            .checkpoint(execution)
            .expect("checkpoint captures the structural revision");
        let recovery = runtime
            .durability()
            .recovery_plan(
                execution,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .expect("native recovery input admitted");
        let mut recovered = runtime_with_test_schema();
        recovered
            .durability_recovery()
            .recover(execution, recovery)
            .expect("checkpoint recovery rebuilds adjacency indexes");
        let recovered_snapshot =
            snapshot_for_owner_branch(&recovered, &BranchId("main".to_owned()));
        assert_eq!(
            recovered
                .read_truth()
                .project_snapshot(&recovered_snapshot)
                .expect("recovered main basis is readable")
                .bounded_adjacency_structural_revision(
                    execution,
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
