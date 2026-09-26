use super::validation_engine_fixtures::*;

#[test]
fn engine_skips_rules_when_request_groups_do_not_intersect() {
    let runtime = runtime_with_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
        )],
    });
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(1),
        merged_intents: vec![MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: RelationId::new(PartitionId::main(), 0, 1),
            },
        ))],
    };

    let results = InvariantEngine::new(&runtime).execute(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            Some(&plan),
            Some(crate::validation::data::InvariantPlanContract::from_merged_plan(&plan)),
        )
        .with_applicable_groups(InvariantGroupSet::of(InvariantGroup::LineageIntegrity)),
    );

    assert!(results.results().is_empty());
}

#[test]
fn engine_marks_unrelated_commit_boundary_rules_not_applicable() {
    let runtime = runtime_with_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
        )],
    });
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(2),
        merged_intents: vec![MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: RelationId::new(PartitionId::main(), 0, 1),
            },
        ))],
    };

    let results =
        crate::validation::invariant_access::test_support::evaluate_main_commit_boundary_plan(
            &runtime, &plan,
        );

    assert!(results.results().is_empty());
}
