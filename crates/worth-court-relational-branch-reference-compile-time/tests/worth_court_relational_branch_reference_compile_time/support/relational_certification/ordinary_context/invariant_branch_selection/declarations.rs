pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::compiler::compile_supply_chain_baseline_with_custom_invariant, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::head_for_supply_chain_branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::definition::SupplyChainWorldDefinition};
pub(crate) use {std::sync::Arc};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::identity::EntityId, worth_relational::facade::identity::PartitionId};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::runtime::CustomInvariantDescriptor, worth_relational::facade::runtime::CustomInvariantExecutionContext, worth_relational::facade::runtime::CustomInvariantExecutionError, worth_relational::facade::runtime::CustomInvariantOperationalMetadata, worth_relational::facade::runtime::CustomInvariantPreparationError, worth_relational::facade::runtime::CustomInvariantRegistration, worth_relational::facade::runtime::CustomInvariantRule, worth_relational::facade::runtime::CustomInvariantScopePlanner, worth_relational::facade::runtime::CustomInvariantSemanticIdentity, worth_relational::facade::runtime::CustomInvariantSemanticVersion, worth_relational::facade::runtime::CustomInvariantVerdict, worth_relational::facade::runtime::InvariantCostClass, worth_relational::facade::runtime::InvariantExecutionPoint, worth_relational::facade::runtime::InvariantFailureEffect, worth_relational::facade::runtime::InvariantGroup, worth_relational::facade::runtime::InvariantGroupSet, worth_relational::facade::runtime::InvariantReportedRule, worth_relational::facade::runtime::InvariantRule};
pub(crate) use {worth_relational::facade::schema::CardinalityContractDeclaration, worth_relational::facade::schema::ContractId, worth_relational::facade::schema::MinimumCardinalityEnforcement, worth_relational::facade::schema::RelationIntegrityDeclarations, worth_relational::facade::schema::RelationalSchemaRegistry};
pub(crate) use {worth_relational::facade::symbols::ClientKey};
pub(crate) use {worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::EntityReference, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RelationSpec, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) fn assert_native_cardinality_execution(
    commit: &worth_relational::facade::transactions::CommitResult,
) {
    assert!(
        commit.invariant_executions().iter().any(|execution| {
            execution.results().iter().any(|result| {
                matches!(
                    &result.rule,
                    worth_relational::facade::runtime::InvariantReportedRule::Native(worth_relational::facade::runtime::InvariantRule::CardinalityMaximumContract(_))
                )
            })
        }),
        "commit must record execution of the native source-cardinality invariant"
    );
}

pub(crate) struct BranchVersionProbeRule;

impl BranchVersionProbeRule {
    pub(crate) const RULE_ID: &'static str = "supply_chain.branch-selected-version";
}

impl CustomInvariantRule for BranchVersionProbeRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: worth_relational::facade::runtime::CustomInvariantRuleId::new(
                    Self::RULE_ID,
                ),
                semantic_version:
                    worth_relational::facade::runtime::CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: std::sync::Arc::from("Supply Chain branch-selected version probe"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(std::num::NonZeroU64::new(1_000_000).unwrap()),
                access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                    read_entity_kinds: vec![],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: true,
                }
                .canonicalize(),
                execution_point:
                    worth_relational::facade::runtime::InvariantExecutionPoint::CommitBoundary,
                groups: worth_relational::facade::runtime::InvariantGroupSet::of(
                    worth_relational::facade::runtime::InvariantGroup::RelationIntegrity,
                ),
                cost_class: worth_relational::facade::runtime::InvariantCostClass::Touched,
                failure_effect:
                    worth_relational::facade::runtime::InvariantFailureEffect::BlockCommit,
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
        context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        assert_eq!(
            context.current_version_id(),
            context.provenance().current_version_id
        );
        Ok(worth_relational::facade::runtime::CustomInvariantVerdict::Pass)
    }
}

pub(crate) fn commit_relation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: BranchId,
    source: EntityId,
    target: EntityId,
    client_key: &str,
) -> Result<worth_relational::facade::transactions::CommitResult, TransactionCommitError> {
    let identity = runtime
        .branch_identity(&branch_id)
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
            worth_relational::facade::transactions::WorkerIntentBatch::new(client_key).push(worth_relational::facade::transactions::MutationIntent::Create(
                worth_relational::facade::transactions::CreateIntent::Relation(RelationSpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: relation_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth),
                    client_key: worth_relational::facade::symbols::ClientKey::raw(client_key),
                    source: worth_relational::facade::transactions::EntityReference::Existing(source),
                    target: worth_relational::facade::transactions::EntityReference::Existing(target),
                    fields: worth_relational::facade::transactions::AspectFieldPatch::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime)
}

pub(crate) fn registry_with_vessel_source_cardinality(
    source: &RelationalSchemaRegistry,
) -> RelationalSchemaRegistry {
    let mut registry = worth_relational::facade::schema::RelationalSchemaRegistry::new();
    for kind in [
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection,
    ] {
        registry = registry
            .register_entity_kind(
                source
                    .entity_registration(entity_kind_id(kind))
                    .expect("source schema entity")
                    .clone(),
            )
            .expect("entity registration remains valid");
    }
    for kind in [
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::TerminalAtPort,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::BerthAtTerminal,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageUsesVessel,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VoyageHasCall,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallAtPort,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CallPrecedes,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::CargoBookedOnVoyage,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::InspectionCoversVessel,
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::SharesPilotageZone,
    ] {
        let mut registration = source
            .relation_registration(relation_kind_id(kind))
            .expect("source schema relation")
            .clone();
        if kind == worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKind::VesselAssignedToBerth {
            let mut declarations = registration.relation_integrity.clone();
            declarations
                .cardinality_contracts
                .push(CardinalityContractDeclaration {
                contract_id: worth_relational::facade::schema::ContractId::new("supply_chain.vessel_source_max_one"),
                source_max: Some(1),
                source_min: None,
                target_max: None,
                target_min: None,
                pair_max: None,
                pair_min: None,
                pair_min_semantics:
                    worth_relational::facade::schema::PairMinimumSemantics::ObservedDirectedPairs,
                minimum_enforcement: worth_relational::facade::schema::MinimumCardinalityEnforcement::CommitBoundary,
            });
            let acyclicity = declarations.acyclicity_contracts.clone();
            let partition_isolation = declarations.partition_isolation_contracts.clone();
            let connectivity = declarations.connectivity_minimum_contracts.clone();
            registration.relation_integrity = worth_relational::facade::schema::RelationIntegrityDeclarations::new(
                declarations.endpoint_kind_contracts,
                declarations.cardinality_contracts,
                declarations.uniqueness_contracts,
                declarations.symmetry_contracts,
                declarations.endpoint_deletion_integrity_contracts,
            )
            .with_acyclicity_contracts(acyclicity)
            .with_partition_isolation_contracts(partition_isolation)
            .with_connectivity_minimum_contracts(connectivity);
        }
        registry = registry
            .register_relation_kind(registration)
            .expect("relation registration remains valid");
    }
    registry
}
