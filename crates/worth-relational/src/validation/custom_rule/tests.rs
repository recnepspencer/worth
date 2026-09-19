use std::sync::Arc;

use super::*;
use crate::facade::identity::KindId;
use crate::facade::runtime::RelationalRuntimeApi;
use crate::facade::schema::RelationalSchemaRegistry;
use crate::facade::transactions::{
    CreateIntent, CreatedEntityRef, DeleteEntityIntent, DeleteRelationIntent, EntityReference,
    EntitySpec, MutationIntent, RelationMutationIntent, RelationSpec, TransactionId,
    UpdateRelationEndpointsIntent, WorkerIntentBatch,
};
use crate::symbols::data::ClientKey;
use crate::tests::support::{create_entity, create_relation, runtime_with_test_schema};
use crate::transactions::data::MergedCommitPlan;
use crate::validation::data::{
    CustomInvariantExecutionError, CustomInvariantOperationalMetadata,
    CustomInvariantPreparationError, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
    InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
};
use crate::validation::engine::InvariantObservation;

mod touched_scope_tests;

struct TestRule;

fn prepared_scope(
    runtime: &crate::runtime::RelationalRuntime,
    observation: &InvariantObservation<'_>,
    merged_plan: Option<&MergedCommitPlan>,
) -> PreparedCustomInvariantScope {
    prepared_scope_with_access(
        runtime,
        observation,
        merged_plan,
        &crate::validation::data::CustomInvariantAccessContract::default(),
    )
}

fn prepared_scope_with_access(
    runtime: &crate::runtime::RelationalRuntime,
    observation: &InvariantObservation<'_>,
    merged_plan: Option<&MergedCommitPlan>,
    access: &crate::validation::data::CustomInvariantAccessContract,
) -> PreparedCustomInvariantScope {
    PreparedCustomInvariantScope::capture(
        observation,
        runtime.current_version_id(),
        merged_plan,
        access,
        &super::CustomInvariantWorkMeter::new(std::num::NonZeroU64::new(4096).unwrap()),
    )
}

#[test]
fn unrelated_shared_endpoint_does_not_expand_its_existing_adjacency() {
    let runtime = runtime_with_test_schema();
    let shared = create_entity(&runtime, "shared");
    for index in 0..64 {
        let leaf = create_entity(&runtime, &format!("leaf-{index}"));
        create_relation(&runtime, shared, leaf, &format!("existing-{index}"));
    }
    let target = create_entity(&runtime, "new-target");
    let relation = MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: crate::identity::data::PartitionId::main(),
        kind_id: KindId(2),
        client_key: ClientKey::raw("planned"),
        source: EntityReference::Existing(shared),
        target: EntityReference::Existing(target),
        fields: crate::transactions::data::AspectFieldPatch::default(),
    }));
    let merged_plan = MergedCommitPlan {
        transaction_id: TransactionId(9_001),
        merged_intents: vec![relation],
    };
    let access = crate::validation::data::CustomInvariantAccessContract {
        read_entity_kinds: vec![KindId(1)],
        read_relation_kinds: vec![KindId(2)],
        affected_entity_kinds: vec![KindId(3)],
        affected_relation_kinds: vec![KindId(2)],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared = prepared_scope_with_access(&runtime, &observation, Some(&merged_plan), &access);
    let view = crate::validation::engine::InvariantRuntimeView::from_runtime(&runtime);
    let planner = CustomInvariantScopePlanner::new_at_current_version(
        &view,
        &observation,
        runtime.current_version_id(),
        runtime.current_version_id(),
        &prepared,
        super::CustomInvariantWorkMeter::new(std::num::NonZeroU64::new(4096).unwrap()),
        Arc::new(access),
    );

    assert!(planner.touched().visible_entity_ids().is_empty());
    assert!(planner.touched().visible_relation_ids().is_empty());
    assert_eq!(planner.touched().planned_relation_creates().len(), 1);
}

#[test]
fn custom_scope_planner_preserves_owner_selected_current_version() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .build();
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = prepared_scope(&runtime, &observation, None);
    let selected_version = crate::identity::data::VersionId(77);
    let view = crate::validation::engine::InvariantRuntimeView::from_runtime(&runtime);
    let planner = CustomInvariantScopePlanner::new_at_current_version(
        &view,
        &observation,
        selected_version,
        selected_version,
        &prepared_scope,
        super::CustomInvariantWorkMeter::new(std::num::NonZeroU64::new(u64::MAX).unwrap()),
        std::sync::Arc::new(crate::validation::data::CustomInvariantAccessContract::default()),
    );

    assert_eq!(planner.version_id(), selected_version);
    assert_eq!(planner.current_version_id(), selected_version);
    assert_ne!(planner.current_version_id(), runtime.current_version_id());
}

impl CustomInvariantRule for TestRule {
    type Scope = usize;

    fn descriptor(&self) -> crate::validation::data::CustomInvariantDescriptor {
        crate::validation::data::CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: crate::validation::data::CustomInvariantRuleId::new("test.rule"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Test Rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(1).unwrap(),
                access: crate::validation::data::CustomInvariantAccessContract::default(),
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

        fn descriptor(&self) -> crate::validation::data::CustomInvariantDescriptor {
            crate::validation::data::CustomInvariantDescriptor {
                identity: CustomInvariantSemanticIdentity {
                    rule_id: crate::validation::data::CustomInvariantRuleId::new(""),
                    semantic_version: CustomInvariantSemanticVersion::new(1, 0),
                },
                display_name: Arc::from("Empty"),
                operational: CustomInvariantOperationalMetadata {
                    maximum_work_units: std::num::NonZeroU64::new(1).unwrap(),
                    access: crate::validation::data::CustomInvariantAccessContract::default(),
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
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .build();
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = prepared_scope(&runtime, &observation, None);
    let view = crate::validation::engine::InvariantRuntimeView::from_runtime(&runtime);
    let context = CustomInvariantExecutionContext::new(
        &view,
        &observation,
        runtime.current_version_id(),
        runtime.current_version_id(),
        &prepared_scope,
        super::CustomInvariantWorkMeter::new(std::num::NonZeroU64::new(u64::MAX).unwrap()),
        std::sync::Arc::new(crate::validation::data::CustomInvariantAccessContract::default()),
    );

    for _ in 0..8 {
        context.traversal().walk_outgoing_from(&[], 1).unwrap();
    }
    let large_seed_set =
        vec![
            crate::identity::data::EntityId::new(crate::identity::data::PartitionId::main(), 0, 1);
            257
        ];
    let error = context
        .traversal()
        .walk_outgoing_from(&large_seed_set, 1)
        .unwrap_err();
    assert!(error.detail().contains("session frontier budget"));
}
