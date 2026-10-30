pub(crate) use std::collections::BTreeMap;
pub(crate) use std::sync::atomic::Ordering;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::graph_composition_probe::registration};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::baseline_audit::audit as audit_supply_chain_baseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_budget_and_invariant_catalog_and_custom_invariants, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::head_for_supply_chain_branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_uniqueness_assertion::assert_unique_conflict};
pub(crate) use {crate::relational_certification::growth_context::scale_invariant_admission::scale_snapshot_observation::current_snapshot_version, crate::relational_certification::growth_context::scale_invariant_admission::scale_snapshot_observation::live_record_count, crate::relational_certification::growth_context::scale_invariant_admission::scale_snapshot_observation::vessel_call_signs};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::PartitionId};
pub(crate) use {worth_relational::facade::runtime::InvariantCatalog, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantExecutionResult, worth_relational::facade::runtime::InvariantRegistration, worth_relational::facade::runtime::InvariantReportedRule, worth_relational::facade::runtime::InvariantRule, worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::transactions::planned_single_field_locator, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::EntitySpec, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) const COMMIT_PROBE_ID: &str = "phase5.large.commit-boundary";
pub(crate) const PUBLICATION_PROBE_ID: &str = "phase5.large.publication";
pub(crate) const GRAPH_PROBE_ID: &str = "phase5.large.graph-composition";
pub(crate) const DUPLICATE_CALL_SIGN: &str = "AURORA";

pub(crate) fn assert_scale_fork_shares_complete_authority(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) {
    let main = BranchId("main".to_owned());
    let (_, source) = runtime
        .observe_fork_source(&main)
        .expect("Scale main remains an owner-observed fork source");
    let child = BranchId("scale-zero-copy".to_owned());
    runtime
        .fork_branch(execution, child.clone(), source)
        .expect("Scale fork remains metadata-only");
    let identities = [
        runtime.main_branch_identity(),
        runtime.branch_identity(&child).unwrap(),
    ];
    let sharing = runtime.observe_branch_sharing(&identities).unwrap();

    assert_eq!(sharing.unique_root_count(), 1);
    assert_eq!(sharing.unique_canonical_commit_artifacts(), 1);
    assert_eq!(sharing.fork_materialized_entity_count(), 0);
    assert_eq!(sharing.fork_materialized_relation_count(), 0);
    assert_eq!(sharing.fork_materialized_authoritative_bytes(), 0);
    assert_eq!(sharing.copied_commit_envelopes(), 0);
    assert_eq!(
        sharing
            .coordination_cells()
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        2
    );
    assert!(
        sharing.logical_branch_authoritative_bytes()
            > sharing.unique_physical_authoritative_bytes(),
        "two Scale branches must expose two logical views of one physical authority graph"
    );
}

pub(crate) fn custom_execution<'a>(
    commit: &'a worth_relational::facade::transactions::CommitResult,
    id: &str,
) -> &'a worth_relational::facade::runtime::InvariantExecutionResult {
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

pub(crate) fn contains_native_uniqueness(
    commit: &worth_relational::facade::transactions::CommitResult,
) -> bool {
    commit.invariant_executions().iter().any(|execution| {
        execution.results().iter().any(|result| {
            matches!(
                &result.rule,
                worth_relational::facade::runtime::InvariantReportedRule::Native(
                    worth_relational::facade::runtime::InvariantRule::UniqueEntityAspectField { .. }
                )
            )
        })
    })
}

pub(crate) fn commit_vessel(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: BranchId,
    client_key: &str,
    call_sign: &str,
) -> Result<worth_relational::facade::transactions::CommitResult, TransactionCommitError> {
    let identity = runtime
        .branch_identity(&branch)
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
            worth_relational::facade::transactions::WorkerIntentBatch::new(client_key).push(worth_relational::facade::transactions::MutationIntent::Create(worth_relational::facade::transactions::CreateIntent::Entity(
                EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(client_key),
                    fields: vessel_fields(call_sign),
                },
            ))),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime)
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
            worth_relational::facade::transactions::WorkerIntentBatch::new("large-graph-plan").push(worth_relational::facade::transactions::MutationIntent::Create(
                worth_relational::facade::transactions::CreateIntent::Entity(EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(
                        "large-graph-plan",
                    ),
                    fields: vessel_fields("GRAPH-PLAN"),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .graph_composition_plan(execution, runtime)
        .expect("the real Scale graph plan is branch-bound and owner-prepared")
}

pub(crate) fn vessel_fields(call_sign: &str) -> AspectFieldPatch {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new("call_sign").expect("call-sign aspect"),
            worth_foundational::facade::FieldKey::new("call_sign").expect("call-sign field"),
        ),
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw(call_sign.to_owned()),
        ),
    );
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new("class").expect("class aspect"),
            worth_foundational::facade::FieldKey::new("class").expect("class field"),
        ),
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw("Feeder".to_owned()),
        ),
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
        worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::Raw("Open".to_owned()),
        ),
    );
    worth_relational::facade::transactions::AspectFieldPatch::new(fields)
}
