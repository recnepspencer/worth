use std::sync::Arc;

use super::*;
use crate::facade::identity::KindId;
use crate::facade::runtime::RelationalRuntimeApi;
use crate::facade::schema::RelationalSchemaRegistry;
use crate::facade::transactions::{
    CreateIntent, CreatedEntityRef, DeleteEntityIntent, DeleteRelationIntent, EntityReference,
    EntitySpec, MutationIntent, RelationMutationIntent, UpdateRelationEndpointsIntent,
    WorkerIntentBatch,
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

struct TestRule;

fn prepared_scope(
    runtime: &crate::runtime::RelationalRuntime,
    observation: &InvariantObservation<'_>,
    merged_plan: Option<&MergedCommitPlan>,
) -> PreparedCustomInvariantScope {
    PreparedCustomInvariantScope::capture(
        observation,
        runtime.current_version_id(),
        merged_plan,
        &super::CustomInvariantWorkMeter::new(std::num::NonZeroU64::new(4096).unwrap()),
    )
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

#[test]
fn touched_scope_tracks_planned_relation_endpoint_updates() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let old_target = create_entity(&runtime, "old-target");
    let new_target = create_entity(&runtime, "new-target");
    let relation_id = create_relation(&runtime, source, old_target, "edge");
    let intent = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id,
            kind_id: KindId(2),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(new_target),
        },
    ));
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(WorkerIntentBatch::new("rewire").push(intent.clone()))
        .expect("test staging stays within configured resource budgets");
    let merged_plan = MergedCommitPlan {
        transaction_id: txn.transaction_id,
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = prepared_scope(&runtime, &observation, Some(&merged_plan));
    let planner = test_scope_planner(
        &runtime,
        &observation,
        runtime.current_version_id(),
        &prepared_scope,
    );

    let updates = planner.touched().planned_relation_endpoint_updates();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].relation_id(), relation_id);
    assert_eq!(updates[0].kind_id(), KindId(2));
    assert_eq!(updates[0].source(), &EntityReference::Existing(source));
    assert_eq!(updates[0].target(), &EntityReference::Existing(new_target));
    assert_eq!(planner.counts().planned_relation_endpoint_update_count(), 1);
    assert_eq!(
        planner
            .touched()
            .provenance_summary()
            .planned_relation_endpoint_update_count,
        1
    );
}

#[test]
fn touched_scope_tracks_planned_relation_endpoint_updates_to_created_entities() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let old_target = create_entity(&runtime, "old-target");
    let relation_id = create_relation(&runtime, source, old_target, "edge");
    let created_target = CreatedEntityRef {
        partition_id: crate::identity::data::PartitionId(1),
        kind_id: KindId(1),
        client_key: ClientKey::raw("planned-target"),
    };
    let create_target = MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: created_target.partition_id,
        kind_id: created_target.kind_id,
        client_key: created_target.client_key.clone(),
        fields: crate::transactions::data::AspectFieldPatch::default(),
    }));
    let update_relation = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id,
            kind_id: KindId(2),
            source: EntityReference::Existing(source),
            target: EntityReference::Created(created_target.clone()),
        },
    ));
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("rewire-to-created")
            .push(create_target.clone())
            .push(update_relation.clone()),
    )
    .expect("test staging stays within configured resource budgets");
    let merged_plan = MergedCommitPlan {
        transaction_id: txn.transaction_id,
        merged_intents: vec![create_target, update_relation],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = prepared_scope(&runtime, &observation, Some(&merged_plan));
    let planner = test_scope_planner(
        &runtime,
        &observation,
        runtime.current_version_id(),
        &prepared_scope,
    );

    let updates = planner.touched().planned_relation_endpoint_updates();
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].relation_id(), relation_id);
    assert_eq!(updates[0].kind_id(), KindId(2));
    assert_eq!(updates[0].source(), &EntityReference::Existing(source));
    assert_eq!(
        updates[0].target(),
        &EntityReference::Created(created_target)
    );
}

#[test]
fn touched_scope_tracks_planned_relation_deletes() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let relation_id = create_relation(&runtime, source, target, "edge");
    let intent = MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id,
    }));
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(WorkerIntentBatch::new("delete").push(intent.clone()))
        .expect("test staging stays within configured resource budgets");
    let merged_plan = MergedCommitPlan {
        transaction_id: txn.transaction_id,
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = prepared_scope(&runtime, &observation, Some(&merged_plan));
    let planner = test_scope_planner(
        &runtime,
        &observation,
        runtime.current_version_id(),
        &prepared_scope,
    );

    assert_eq!(planner.touched().planned_relation_deletes(), &[relation_id]);
    assert_eq!(planner.counts().planned_relation_delete_count(), 1);
    assert_eq!(
        planner
            .touched()
            .provenance_summary()
            .planned_relation_delete_count,
        1
    );
}

#[test]
fn touched_scope_tracks_planned_entity_deletes() {
    let runtime = runtime_with_test_schema();
    let entity_id = create_entity(&runtime, "entity");
    let intent = MutationIntent::Entity(crate::facade::transactions::EntityMutationIntent::Delete(
        DeleteEntityIntent { entity_id },
    ));
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(WorkerIntentBatch::new("delete-entity").push(intent.clone()))
        .expect("test staging stays within configured resource budgets");
    let merged_plan = MergedCommitPlan {
        transaction_id: txn.transaction_id,
        merged_intents: vec![intent],
    };
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let prepared_scope = prepared_scope(&runtime, &observation, Some(&merged_plan));
    let planner = test_scope_planner(
        &runtime,
        &observation,
        runtime.current_version_id(),
        &prepared_scope,
    );

    assert_eq!(planner.touched().planned_entity_deletes(), &[entity_id]);
    assert_eq!(planner.counts().planned_entity_delete_count(), 1);
    assert_eq!(
        planner
            .touched()
            .provenance_summary()
            .planned_entity_delete_count,
        1
    );
}

fn test_scope_planner<'a>(
    runtime: &'a crate::runtime::RelationalRuntime,
    observation: &'a InvariantObservation<'a>,
    version: crate::identity::data::VersionId,
    prepared: &PreparedCustomInvariantScope,
) -> CustomInvariantScopePlanner<'a> {
    let view = crate::validation::engine::InvariantRuntimeView::from_runtime(runtime);
    CustomInvariantScopePlanner::new_at_current_version(
        &view,
        observation,
        version,
        runtime.current_version_id(),
        prepared,
        super::CustomInvariantWorkMeter::new(std::num::NonZeroU64::new(4096).unwrap()),
        Arc::new(crate::validation::data::CustomInvariantAccessContract {
            read_entity_kinds: vec![KindId(1)],
            read_relation_kinds: vec![KindId(2)],
            affected_entity_kinds: vec![KindId(1)],
            affected_relation_kinds: vec![KindId(2)],
        }),
    )
}
