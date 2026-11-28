use super::validation_engine_fixtures::{
    DeleteRelationIntent, InvariantCatalog, InvariantEngine, InvariantExecutionRequest,
    InvariantGroup, InvariantGroupSet, InvariantObservation, InvariantRegistration,
    InvariantRequestProfile, InvariantRule, MergedCommitPlan, MutationIntent, PartitionId,
    RelationId, RelationMutationIntent, TransactionId,
};
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_contracts::internal_port::config::data::runtime_config::RelationalRuntimeConfig;
use worth_relational_contracts::internal_port::identity::data::primitives::VersionId;

#[test]
fn engine_skips_rules_when_request_groups_do_not_intersect() {
    let mut config = RelationalRuntimeConfig::default();
    config.schema.invariant_catalog = InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::unique_entity_aspect_field(crate::contracts::identity::data::KindId(1), crate::contracts::indexes::data::definition::DerivedIndexId(1),
                AspectKey::new("name").unwrap(),
                FieldKey::new("name").unwrap(),
            ),
        )],
    };
    let instrumentation = RuntimeInstrumentation::default();
    let view = crate::validation::engine::test_view::invariant_view(
        &instrumentation,
        config,
        VersionId(0),
    );
    let partitions = BTreeMap::new();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(1),
        merged_intents: vec![MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: RelationId::new(PartitionId::main(), 0, 1),
            },
        ))],
    };

    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            VersionId(0),
            Some(&plan),
            Some(
                worth_relational_contracts::internal_port::validation::data::contracts::InvariantPlanContract::from_merged_plan(&plan),
            ),
        )
        .with_applicable_groups(InvariantGroupSet::of(InvariantGroup::LineageIntegrity)),
    );

    assert!(results.results().is_empty());
}
