pub(crate) use {std::collections::BTreeMap};
pub(crate) use {std::sync::atomic::Ordering};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::graph_composition_probe::registration};
pub(crate) use {crate::relational_certification::ordinary_context::graph_selected_state_probe::registration as selected_state_registration, crate::relational_certification::ordinary_context::graph_selected_state_probe::RULE_ID};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::EntityId, worth_relational::facade::identity::PartitionId};
pub(crate) use {worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial};
pub(crate) use {worth_relational::facade::runtime::InvariantCatalog, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantExecutionResult, worth_relational::facade::runtime::InvariantReportedRule, worth_relational::facade::runtime::InvariantVerdict, worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::transactions::planned_single_field_locator, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RecordRef, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) const COMMIT_PROBE_ID: &str = "phase5.common.commit-boundary";
pub(crate) const GRAPH_PROBE_ID: &str = "phase5.common.graph-composition";

pub(crate) fn assert_graph_binding_stales_after_child_diverges(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    child: &BranchId,
) {
    let child_identity = runtime.branch_identity(child).unwrap();
    let stale_options = runtime.admit_branch_basis(&child_identity).unwrap();
    commit_graph_entity(execution, runtime, child, "child-divergence");
    let denied = runtime
        .begin_branch_transaction(
            &stale_options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap_err();
    assert_eq!(
        denied,
        worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial::StaleBasis
    );
}

pub(crate) fn custom_execution<'a>(
    commit: &'a worth_relational::facade::transactions::CommitResult,
    id: &str,
) -> &'a InvariantExecutionResult {
    commit
        .invariant_executions()
        .iter()
        .find(|execution| {
            execution.results().iter().any(|result| {
                matches!(
                    &result.rule,
                    worth_relational::facade::runtime::InvariantReportedRule::Custom(identity) if identity.rule_id.as_str() == id
                )
            })
        })
        .unwrap_or_else(|| panic!("missing custom execution {id}"))
}

pub(crate) fn contains_custom_rule(
    commit: &worth_relational::facade::transactions::CommitResult,
    id: &str,
) -> bool {
    commit.invariant_executions().iter().any(|execution| {
        execution.results().iter().any(|result| {
            matches!(
                &result.rule,
                worth_relational::facade::runtime::InvariantReportedRule::Custom(identity) if identity.rule_id.as_str() == id
            )
        })
    })
}

pub(crate) fn contains_custom_rule_in_execution(
    execution: &InvariantExecutionResult,
    id: &str,
) -> bool {
    execution.results().iter().any(|result| {
        matches!(
            &result.rule,
            worth_relational::facade::runtime::InvariantReportedRule::Custom(identity) if identity.rule_id.as_str() == id
        )
    })
}

pub(crate) fn custom_rule_verdict<'a>(
    execution: &'a InvariantExecutionResult,
    id: &str,
) -> &'a InvariantVerdict {
    &execution
        .results()
        .iter()
        .find(|result| {
            matches!(
                &result.rule,
                worth_relational::facade::runtime::InvariantReportedRule::Custom(identity) if identity.rule_id.as_str() == id
            )
        })
        .unwrap_or_else(|| panic!("missing custom execution {id}"))
        .verdict
}

pub(crate) fn graph_execution(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> InvariantExecutionResult {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    let options = runtime
        .admit_branch_basis(&identity)
        .expect("transaction authority is owner-issued");
    let mut transaction = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(
            graph_entity_batch("common-graph-plan"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .graph_composition_plan(execution, runtime)
        .expect("the Standard graph plan is branch-bound and owner-prepared")
}

pub(crate) fn commit_graph_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    client_key: &str,
) -> EntityId {
    let identity = runtime.branch_identity(branch).unwrap();
    let options = runtime.admit_branch_basis(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &options,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("owner-admitted transaction context");
    transaction
        .push_batch(
            graph_entity_batch(client_key),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let commit = transaction
        .commit(execution, runtime)
        .expect("branch divergence commits");
    commit
        .changed_records
        .iter()
        .find_map(|record| match *record {
            worth_relational::facade::transactions::RecordRef::Entity(entity) => Some(entity),
            worth_relational::facade::transactions::RecordRef::Relation(_) => None,
        })
        .expect("graph divergence creates one entity")
}

pub(crate) fn graph_entity_batch(client_key: &str) -> WorkerIntentBatch {
    worth_relational::facade::transactions::WorkerIntentBatch::new(client_key).push(worth_relational::facade::transactions::MutationIntent::Create(worth_relational::facade::transactions::CreateIntent::Entity(
        EntitySpec {
            partition_id: worth_relational::facade::identity::PartitionId::main(),
            kind_id: worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel),
            client_key: worth_relational::facade::symbols::ClientKey::raw(client_key),
            fields: vessel_fields(client_key),
        },
    )))
}

pub(crate) fn vessel_fields(call_sign: &str) -> AspectFieldPatch {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new("call_sign").expect("call-sign aspect"),
            worth_foundational::facade::FieldKey::new("call_sign").expect("call-sign field"),
        ),
        worth_foundational::facade::AspectValue::String(worth_foundational::facade::InternedString::Raw(call_sign.to_owned())),
    );
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new("class").expect("class aspect"),
            worth_foundational::facade::FieldKey::new("class").expect("class field"),
        ),
        worth_foundational::facade::AspectValue::String(worth_foundational::facade::InternedString::Raw("Feeder".to_owned())),
    );
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new("capacity").expect("capacity aspect"),
            worth_foundational::facade::FieldKey::new("capacity").expect("capacity field"),
        ),
        worth_foundational::facade::AspectValue::UInt64(9_999),
    );
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new("posture").expect("posture aspect"),
            worth_foundational::facade::FieldKey::new("posture").expect("posture field"),
        ),
        worth_foundational::facade::AspectValue::String(worth_foundational::facade::InternedString::Raw("Open".to_owned())),
    );
    worth_relational::facade::transactions::AspectFieldPatch::new(fields)
}