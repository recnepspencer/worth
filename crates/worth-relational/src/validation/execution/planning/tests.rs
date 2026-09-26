use super::{plan_invariant_execution, planned_proof_boundary_summary};
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::facade::{
    runtime::RelationalRuntimeApi,
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
        RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
};
use crate::identity::data::{KindId, PartitionId};
use crate::schema::data::{EndpointKindContractDeclaration, RelationIntegrityDeclarations};
use crate::transactions::data::{
    CreateIntent, EntitySpec, MergedCommitPlan, MutationIntent, RelationSpec, WorkerIntentBatch,
};
use crate::validation::data::InvariantPlanContract;
use crate::validation::engine::{
    InvariantExecutionRequest, InvariantObservation, InvariantPlanScopeClass,
    InvariantRequestProfile, InvariantScopeWideningCause,
};

fn relation_runtime() -> crate::runtime::RelationalRuntime {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.edge.a".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: "kind2".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(3),
                kind_name: "test.edge.b".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: "kind3".into(),
                        allowed_source_kinds: vec![KindId(1)],
                        allowed_target_kinds: vec![KindId(1)],
                        self_edges_allowed: false,
                        cross_context_policy: CrossContextPolicy::AllowExplicit,
                    }],
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
}

fn request_for_plan<'runtime>(
    runtime: &'runtime crate::runtime::RelationalRuntime,
    plan: &'runtime MergedCommitPlan,
) -> InvariantExecutionRequest<'runtime> {
    InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        Some(plan),
        Some(InvariantPlanContract::from_merged_plan(plan)),
    )
}

fn create_entity(
    runtime: &crate::runtime::RelationalRuntime,
    name: &str,
) -> crate::identity::data::EntityId {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(format!("entity-{name}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(name),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(runtime).unwrap();
    outcome
        .changed_records
        .iter()
        .find_map(|record| match record {
            crate::facade::transactions::RecordRef::Entity(entity_id) => Some(*entity_id),
            crate::facade::transactions::RecordRef::Relation(_) => None,
        })
        .expect("created entity")
}

#[test]
fn planner_packets_only_include_relation_integrity_registrations_authorized_by_plan_scope() {
    let runtime = relation_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("planned").push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("planned"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))),
    )
    .expect("test staging stays within configured resource budgets");
    let plan = txn.merged_plan(&runtime).unwrap().clone();

    let request = request_for_plan(&runtime, &plan);
    let view = crate::validation::engine::InvariantRuntimeView::from_runtime(&runtime);
    let prepared = plan_invariant_execution(&view, &request);
    let packet_relation_kinds = prepared
        .packets
        .iter()
        .filter_map(|packet| match &packet.registration {
            crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::Native(registration) => {
                relation_kind_for_rule(&registration.rule)
            }
            crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::Custom { .. }
            | crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::CustomNotApplicable { .. } => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(packet_relation_kinds, vec![KindId(2)]);
    assert_eq!(prepared.packets.len(), 1);
}

#[test]
fn planner_proof_boundary_reports_partition_scoped_relation_integrity_packets() {
    let runtime = relation_runtime();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("planned").push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("planned"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            },
        ))),
    )
    .expect("test staging stays within configured resource budgets");
    let plan = txn.merged_plan(&runtime).unwrap().clone();

    let request = request_for_plan(&runtime, &plan);
    let view = crate::validation::engine::InvariantRuntimeView::from_runtime(&runtime);
    let prepared = plan_invariant_execution(&view, &request);
    let summary = planned_proof_boundary_summary(&prepared);

    assert_eq!(
        summary.scope_class(),
        InvariantPlanScopeClass::PartitionScope
    );
    assert!(summary.widened_causes().is_empty());
    assert_eq!(summary.packet_count(), 1);
    assert_eq!(summary.touched_partition_count(), 1);
}

#[test]
fn planner_proof_boundary_reports_broader_scope_when_no_merged_plan_is_available() {
    let runtime = relation_runtime();
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );

    let view = crate::validation::engine::InvariantRuntimeView::from_runtime(&runtime);
    let prepared = plan_invariant_execution(&view, &request);
    let summary = planned_proof_boundary_summary(&prepared);

    assert_eq!(summary.scope_class(), InvariantPlanScopeClass::BroaderScope);
    assert_eq!(
        summary.widened_causes(),
        &[InvariantScopeWideningCause::AllObservedPartitionScope]
    );
}

fn relation_kind_for_rule(
    rule: &crate::validation::data::InvariantRule,
) -> Option<crate::identity::data::KindId> {
    match rule {
        crate::validation::data::InvariantRule::EndpointKindContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::validation::data::InvariantRule::CardinalityMaximumContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::validation::data::InvariantRule::CardinalityMinimumContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::validation::data::InvariantRule::UniquenessContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::validation::data::InvariantRule::SymmetryContract(contract) => {
            Some(contract.relation_kind_id)
        }
        crate::validation::data::InvariantRule::EndpointDeletionIntegrityContract(contract) => {
            Some(contract.relation_kind_id)
        }
        _ => None,
    }
}
