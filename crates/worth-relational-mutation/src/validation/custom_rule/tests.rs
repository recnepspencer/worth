use std::sync::Arc;

use super::*;
use crate::contracts::identity::data::primitives::{KindId, PartitionId, VersionId};
use crate::contracts::symbols::data::client_key::ClientKey;
use crate::contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, DeleteEntityIntent, DeleteRelationIntent, MutationIntent, RelationMutationIntent,
    UpdateRelationEndpointsIntent,
};
use crate::contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::contracts::transactions::data::primitives::{
    CreatedEntityRef, EntityReference, EntitySpec, RelationSpec, TransactionId,
};
use crate::contracts::validation::data::custom_rule::errors::{
    CustomInvariantExecutionError, CustomInvariantPreparationError, CustomInvariantVerdict,
};
use crate::contracts::validation::data::descriptor::CustomInvariantOperationalMetadata;
use crate::contracts::validation::data::execution::{
    InvariantExecutionPoint, InvariantFailureEffect,
};
use crate::contracts::validation::data::groups::{
    InvariantCostClass, InvariantGroup, InvariantGroupSet,
};
use crate::contracts::validation::data::rule_id::{
    CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion,
};
use crate::validation::engine::{test_view::empty_invariant_view, InvariantObservation};
use worth_relational_storage::internal_port::storage::overlay::small_relation_partition::relation_partition_for_test;

pub mod touched_scope_tests;

struct TestRule;

fn prepared_scope(
    current_version: VersionId,
    observation: &InvariantObservation<'_>,
    merged_plan: Option<&MergedCommitPlan>,
) -> PreparedCustomInvariantScope {
    scope_with_access(
        current_version,
        observation,
        merged_plan,
        &crate::contracts::validation::data::descriptor::CustomInvariantAccessContract::default(),
    )
}

fn scope_with_access(
    current_version: VersionId,
    observation: &InvariantObservation<'_>,
    merged_plan: Option<&MergedCommitPlan>,
    access: &crate::contracts::validation::data::descriptor::CustomInvariantAccessContract,
) -> PreparedCustomInvariantScope {
    PreparedCustomInvariantScope::capture(
        observation,
        current_version,
        merged_plan,
        access,
        &super::CustomInvariantWorkMeter::new(Some(std::num::NonZeroU64::new(4096).unwrap())),
    )
}

#[test]
fn unrelated_shared_endpoint_does_not_expand_its_existing_adjacency() {
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let id = PartitionId::main();
    let edges = (1..=64)
        .map(|leaf| (0, leaf, KindId(2)))
        .collect::<Vec<_>>();
    let (partition, entities, _) = relation_partition_for_test(id, 66, &edges);
    let partitions = std::collections::BTreeMap::from([(id, partition)]);
    let shared = entities[0];
    let target = entities[65];
    let version = VersionId(1);
    let relation = MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: crate::contracts::identity::data::primitives::PartitionId::main(),
        kind_id: KindId(2),
        client_key: ClientKey::raw("planned"),
        source: EntityReference::Existing(shared),
        target: EntityReference::Existing(target),
        fields: crate::contracts::transactions::data::aspect_field_patch::AspectFieldPatch::default(
        ),
    }));
    let merged_plan = MergedCommitPlan {
        transaction_id: TransactionId(9_001),
        merged_intents: vec![relation],
    };
    let access = crate::contracts::validation::data::descriptor::CustomInvariantAccessContract {
        read_entity_kinds: vec![KindId(1)],
        read_relation_kinds: vec![KindId(2)],
        affected_entity_kinds: vec![KindId(3)],
        affected_relation_kinds: vec![KindId(2)],
        include_relation_endpoint_entity_touches: true,
    };
    let observation = InvariantObservation::committed_branch(&partitions);
    let prepared = scope_with_access(version, &observation, Some(&merged_plan), &access);
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let view = empty_invariant_view(&instrumentation);
    let planner = CustomInvariantScopePlanner::new_at_current_version(
        &view,
        *execution,
        &observation,
        version,
        version,
        &prepared,
        super::CustomInvariantWorkMeter::new(Some(std::num::NonZeroU64::new(4096).unwrap())),
        Arc::new(access),
    );

    assert!(planner.touched().visible_entity_ids().is_empty());
    assert!(planner.touched().visible_relation_ids().is_empty());
    assert_eq!(planner.touched().planned_relation_creates().len(), 1);
}

#[test]
fn custom_scope_planner_preserves_owner_selected_current_version() {
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let partitions = std::collections::BTreeMap::new();
    let observation = InvariantObservation::committed_branch(&partitions);
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let view = empty_invariant_view(&instrumentation);
    let prepared_scope = prepared_scope(view.current_version_id(), &observation, None);
    let selected_version = VersionId(77);
    let planner = CustomInvariantScopePlanner::new_at_current_version(
        &view,
        *execution,
        &observation,
        selected_version,
        selected_version,
        &prepared_scope,
        super::CustomInvariantWorkMeter::new(Some(std::num::NonZeroU64::new(u64::MAX).unwrap())),
        std::sync::Arc::new(
            crate::contracts::validation::data::descriptor::CustomInvariantAccessContract::default(
            ),
        ),
    );

    assert_eq!(planner.version_id(), selected_version);
    assert_eq!(planner.current_version_id(), selected_version);
    assert_ne!(planner.current_version_id(), view.current_version_id());
}

impl CustomInvariantRule for TestRule {
    type Scope = usize;

    fn descriptor(
        &self,
    ) -> crate::contracts::validation::data::descriptor::CustomInvariantDescriptor {
        crate::contracts::validation::data::descriptor::CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: crate::contracts::validation::data::rule_id::CustomInvariantRuleId::new("test.rule"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Test Rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(std::num::NonZeroU64::new(1).unwrap()),
                access: crate::contracts::validation::data::descriptor::CustomInvariantAccessContract::default(
                ),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        let traversal = planner
            .traversal()
            .walk_outgoing_from(planner.touched().visible_entity_ids(), 1)?;
        Ok(traversal.visited_entities().len())
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        assert_eq!(*scope, 0);
        Ok(CustomInvariantVerdict::Pass)
    }
}

#[test]
fn custom_registration_exposes_descriptor_and_rule_id() {
    let registration = CustomInvariantRegistration::new(TestRule).unwrap();
    assert_eq!(registration.rule_id().as_str(), "test.rule");
    assert_eq!(registration.descriptor().display_name.as_ref(), "Test Rule");
}

#[test]
fn custom_registration_rejects_empty_ids() {
    struct EmptyRule;

    impl CustomInvariantRule for EmptyRule {
        type Scope = ();

        fn descriptor(
            &self,
        ) -> crate::contracts::validation::data::descriptor::CustomInvariantDescriptor {
            crate::contracts::validation::data::descriptor::CustomInvariantDescriptor {
                identity: CustomInvariantSemanticIdentity {
                    rule_id: crate::contracts::validation::data::rule_id::CustomInvariantRuleId::new(""),
                    semantic_version: CustomInvariantSemanticVersion::new(1, 0),
                },
                display_name: Arc::from("Empty"),
                operational: CustomInvariantOperationalMetadata {
                    maximum_work_units: Some(std::num::NonZeroU64::new(1).unwrap()),
                    access:
                        crate::contracts::validation::data::descriptor::CustomInvariantAccessContract::default(
                        ),
                    execution_point: InvariantExecutionPoint::CommitBoundary,
                    groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                    cost_class: InvariantCostClass::Touched,
                    failure_effect: InvariantFailureEffect::BlockCommit,
                },
            }
        }

        fn prepare_scope(
            &self,
            _planner: &mut CustomInvariantScopePlanner<'_>,
        ) -> Result<Self::Scope, CustomInvariantPreparationError> {
            Ok(())
        }

        fn evaluate(
            &self,
            _context: &CustomInvariantExecutionContext<'_>,
            _scope: &Self::Scope,
        ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
            Ok(CustomInvariantVerdict::Pass)
        }
    }

    let error = CustomInvariantRegistration::new(EmptyRule).unwrap_err();
    assert_eq!(error, CustomInvariantRegistrationError::EmptyRuleId);
}

#[test]
fn traversal_budget_is_session_wide() {
    let host_request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let partitions = std::collections::BTreeMap::new();
    let observation = InvariantObservation::committed_branch(&partitions);
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let view = empty_invariant_view(&instrumentation);
    let prepared_scope = prepared_scope(view.current_version_id(), &observation, None);
    let context = CustomInvariantExecutionContext::new(
        &view,
        *execution,
        &observation,
        view.current_version_id(),
        view.current_version_id(),
        &prepared_scope,
        None,
        super::CustomInvariantWorkMeter::new(Some(std::num::NonZeroU64::new(u64::MAX).unwrap())),
        std::sync::Arc::new(
            crate::contracts::validation::data::descriptor::CustomInvariantAccessContract::default(
            ),
        ),
    );

    for _ in 0..8 {
        context.traversal().walk_outgoing_from(&[], 1).unwrap();
    }
    let large_seed_set = vec![
        crate::contracts::identity::data::record_ids::EntityId::new(
            crate::contracts::identity::data::primitives::PartitionId::main(),
            0,
            1
        );
        257
    ];
    let error = context
        .traversal()
        .walk_outgoing_from(&large_seed_set, 1)
        .unwrap_err();
    assert!(error.detail().contains("session frontier budget"));
}
