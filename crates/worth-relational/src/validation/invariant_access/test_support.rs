use crate::capabilities::SchemaSource;
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy, RelationIntegrityScopeBudget};
use crate::facade::identity::PartitionId;
use crate::facade::runtime::{InvariantCatalog, RelationalExecutionModel};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::identity::data::KindId;
use crate::schema::data::{
    CardinalityContractDeclaration, EndpointKindContractDeclaration, RelationIntegrityDeclarations,
    SymmetryContractDeclaration, SymmetryMode,
};
use crate::symbols::data::ClientKey;
use crate::transactions::data::MergedCommitPlan;
use crate::transactions::data::{EntitySpec, MutationIntent};
use crate::validation::engine::InvariantExecutionResult;

pub(crate) fn evaluate_main_commit_boundary_plan(
    runtime: &RelationalRuntime,
    plan: &MergedCommitPlan,
) -> InvariantExecutionResult {
    let selected = selected_main_branch_state(runtime);
    runtime
        .validation()
        .commit_boundary_for_selected_branch_plan(&selected, plan)
}

pub(crate) fn evaluate_main_graph_composition_plan(
    runtime: &RelationalRuntime,
    plan: &MergedCommitPlan,
) -> InvariantExecutionResult {
    let selected = selected_main_branch_state(runtime);
    runtime
        .validation()
        .graph_composition_for_selected_branch_plan(&selected, plan)
}

fn selected_main_branch_state(
    runtime: &RelationalRuntime,
) -> crate::branch::SelectedRelationalBranchState {
    let context = crate::tests::support::test_owner_transaction_validation_input_for_main(runtime);
    runtime
        .selected_branch_state(context.basis())
        .expect("owner-admitted main basis selects its exact branch state")
}

pub(super) fn runtime_with_invariants(
    invariant_catalog: InvariantCatalog,
    execution_model: RelationalExecutionModel,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .invariant_catalog(invariant_catalog)
        .execution_model(execution_model)
        .build()
}

pub(super) fn relation_integrity_runtime() -> RelationalRuntime {
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
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    vec![EndpointKindContractDeclaration {
                        contract_id: "no_self".into(),
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

pub(super) fn relation_symmetry_runtime(mode: SymmetryMode) -> RelationalRuntime {
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
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    vec![SymmetryContractDeclaration {
                        contract_id: "paired_twin".into(),
                        mode,
                    }],
                    Vec::new(),
                ),
            })
        })
        .unwrap();
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
}

pub(super) fn relation_cardinality_runtime() -> RelationalRuntime {
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
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::new(
                    Vec::new(),
                    vec![CardinalityContractDeclaration {
                        contract_id: "source_max_one".into(),
                        source_max: Some(1),
                        source_min: None,
                        target_max: None,
                        target_min: None,
                        pair_max: None,
                        pair_min: None,
                        pair_min_semantics:
                            crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
                        minimum_enforcement:
                            crate::schema::data::MinimumCardinalityEnforcement::CertificationBoundary,
                    }],
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

pub(super) fn relation_integrity_runtime_with_scope_budget(
    relation_integrity_scope_budget: RelationIntegrityScopeBudget,
) -> RelationalRuntime {
    let registry = relation_integrity_runtime().schema_registry().clone();
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .relation_integrity_scope_budget(relation_integrity_scope_budget)
        .build()
}

pub(super) fn create_entity(
    runtime: &RelationalRuntime,
    name: &str,
) -> crate::identity::data::EntityId {
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(runtime);
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        crate::facade::transactions::WorkerIntentBatch::new(name).push(MutationIntent::Create(
            crate::transactions::data::CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(name),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(runtime).unwrap();
    match outcome.changed_records[0] {
        crate::facade::transactions::RecordRef::Entity(entity_id) => entity_id,
        _ => panic!("expected entity"),
    }
}
