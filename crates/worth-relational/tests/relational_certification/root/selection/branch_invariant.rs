use super::world::supply_chain::{
    compile_supply_chain_baseline, compile_supply_chain_baseline_with_custom_invariant,
    entity_kind_id, head_for_supply_chain_branch, relation_kind_id, CompiledSupplyChainProgram,
    EntityKind, RelationKind, SupplyChainScale, SupplyChainWorldDefinition,
};
use std::sync::Arc;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::runtime::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
    CustomInvariantRegistration, CustomInvariantRule, CustomInvariantScopePlanner,
    CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion, CustomInvariantVerdict,
    InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup,
    InvariantGroupSet, InvariantReportedRule, InvariantRule,
};
use worth_relational::facade::schema::{
    CardinalityContractDeclaration, ContractId, MinimumCardinalityEnforcement,
    RelationIntegrityDeclarations, RelationalSchemaRegistry,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    CreateIntent, EntityReference, MutationIntent, RelationSpec, TransactionCommitError,
    WorkerIntentBatch,
};

#[test]
fn supply_chain_child_commit_uses_child_root_after_main_advances() {
    let original = CompiledSupplyChainProgram::compile(
        SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let registry = registry_with_vessel_source_cardinality(original.schema_registry());
    let program = original.with_schema_registry_for_test(registry);
    let world = compile_supply_chain_baseline(program).expect("Court world compiles");

    let source = world.handles.entities
        [&super::world::supply_chain::EntityKey::new(EntityKind::Vessel, 1)]
        .id;
    let main_target = world.handles.entities
        [&super::world::supply_chain::EntityKey::new(EntityKind::Berth, 1)]
        .id;
    let child_target = world.handles.entities
        [&super::world::supply_chain::EntityKey::new(EntityKind::Berth, 2)]
        .id;

    let (_, source_observation) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    world
        .runtime
        .fork_branch(BranchId("storm".to_owned()), source_observation)
        .expect("storm fork shares the Supply Chain root");

    let main_commit = commit_relation(
        &world.runtime,
        BranchId("main".to_owned()),
        source,
        main_target,
        "main-vessel-one-assignment",
    )
    .expect("main assignment commits against the advanced main root");
    assert_native_cardinality_execution(&main_commit);

    let main_head_before_rejection =
        head_for_supply_chain_branch(&world.runtime, &BranchId("main".to_owned())).version_id;
    let rejected_main_assignment = commit_relation(
        &world.runtime,
        BranchId("main".to_owned()),
        source,
        child_target,
        "main-vessel-one-conflicting-assignment",
    );
    assert!(
        rejected_main_assignment.is_err(),
        "native source-cardinality invariant must reject the second main assignment"
    );
    let main_head_after_rejection =
        head_for_supply_chain_branch(&world.runtime, &BranchId("main".to_owned())).version_id;
    assert_eq!(main_head_after_rejection, main_head_before_rejection);

    let child_commit = commit_relation(
        &world.runtime,
        BranchId("storm".to_owned()),
        source,
        child_target,
        "storm-vessel-one-assignment",
    );
    assert!(
        child_commit.is_ok(),
        "the child root has no assignment; a global-main invariant route would reject it: {child_commit:?}"
    );
    assert_native_cardinality_execution(&child_commit.expect("child assignment remains legal"));
}

fn assert_native_cardinality_execution(
    commit: &worth_relational::facade::transactions::CommitResult,
) {
    assert!(
        commit.invariant_executions().iter().any(|execution| {
            execution.results().iter().any(|result| {
                matches!(
                    &result.rule,
                    InvariantReportedRule::Native(InvariantRule::CardinalityMaximumContract(_))
                )
            })
        }),
        "commit must record execution of the native source-cardinality invariant"
    );
}

#[test]
fn supply_chain_child_custom_invariant_uses_child_selected_version() {
    let original = CompiledSupplyChainProgram::compile(
        SupplyChainWorldDefinition::operating(SupplyChainScale::court())
            .expect("Court Supply Chain definition is valid"),
    )
    .expect("Supply Chain program compiles");
    let registry = registry_with_vessel_source_cardinality(original.schema_registry());
    let program = original.with_schema_registry_for_test(registry);
    let world = compile_supply_chain_baseline_with_custom_invariant(
        program,
        CustomInvariantRegistration::new(BranchVersionProbeRule)
            .expect("branch-version custom invariant registers"),
    )
    .expect("Court world compiles with the custom invariant");

    let source = world.handles.entities
        [&super::world::supply_chain::EntityKey::new(EntityKind::Vessel, 1)]
        .id;
    let main_target = world.handles.entities
        [&super::world::supply_chain::EntityKey::new(EntityKind::Berth, 1)]
        .id;
    let child_target = world.handles.entities
        [&super::world::supply_chain::EntityKey::new(EntityKind::Berth, 2)]
        .id;

    let (_, source_observation) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    world
        .runtime
        .fork_branch(BranchId("storm".to_owned()), source_observation)
        .expect("storm fork shares the Supply Chain root");

    let main_commit = commit_relation(
        &world.runtime,
        BranchId("main".to_owned()),
        source,
        main_target,
        "main-vessel-one-custom-assignment",
    )
    .expect("main assignment commits against the advanced main root");
    let child_basis_version =
        head_for_supply_chain_branch(&world.runtime, &BranchId("storm".to_owned())).version_id;
    assert_ne!(child_basis_version, main_commit.version_id);

    let child_commit = commit_relation(
        &world.runtime,
        BranchId("storm".to_owned()),
        source,
        child_target,
        "storm-vessel-one-custom-assignment",
    )
    .expect("storm commit uses its own root and custom invariant");
    let custom_execution = child_commit
        .invariant_executions()
        .iter()
        .find_map(|execution| {
            execution.results().iter().find_map(|result| {
                matches!(
                    &result.rule,
                    InvariantReportedRule::Custom(identity)
                        if identity.rule_id.as_str() == BranchVersionProbeRule::RULE_ID
                )
                .then_some((execution, result))
            })
        })
        .expect("child commit records the registered custom invariant");
    assert_eq!(
        custom_execution.0.metadata().execution_point(),
        InvariantExecutionPoint::CommitBoundary
    );
    assert_eq!(
        custom_execution.0.metadata().current_version_id(),
        child_basis_version,
        "custom metadata must preserve the branch-selected basis version"
    );
    assert_eq!(
        custom_execution
            .1
            .custom_provenance()
            .expect("custom execution carries provenance")
            .current_version_id,
        child_basis_version,
        "custom provenance must preserve the branch-selected basis version"
    );
}

struct BranchVersionProbeRule;

impl BranchVersionProbeRule {
    const RULE_ID: &'static str = "supply_chain.branch-selected-version";
}

impl CustomInvariantRule for BranchVersionProbeRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: worth_relational::facade::runtime::CustomInvariantRuleId::new(
                    Self::RULE_ID,
                ),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Supply Chain branch-selected version probe"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(1_000_000).unwrap(),
                access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                    read_entity_kinds: vec![],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![],
                    affected_relation_kinds: vec![],
                }
                .canonicalize(),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::RelationIntegrity),
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
        context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        assert_eq!(
            context.current_version_id(),
            context.provenance().current_version_id
        );
        Ok(CustomInvariantVerdict::Pass)
    }
}

fn commit_relation(
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
            WorkerIntentBatch::new(client_key).push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: relation_kind_id(RelationKind::VesselAssignedToBerth),
                    client_key: ClientKey::raw(client_key),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    fields: worth_relational::facade::transactions::AspectFieldPatch::default(),
                }),
            )),
        )
        .unwrap();
    transaction.commit(runtime)
}

fn registry_with_vessel_source_cardinality(
    source: &RelationalSchemaRegistry,
) -> RelationalSchemaRegistry {
    let mut registry = RelationalSchemaRegistry::new();
    for kind in [
        EntityKind::Port,
        EntityKind::Terminal,
        EntityKind::Berth,
        EntityKind::Vessel,
        EntityKind::Voyage,
        EntityKind::PortCall,
        EntityKind::CargoLot,
        EntityKind::Inspection,
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
        RelationKind::TerminalAtPort,
        RelationKind::BerthAtTerminal,
        RelationKind::VesselAssignedToBerth,
        RelationKind::VoyageUsesVessel,
        RelationKind::VoyageHasCall,
        RelationKind::CallAtPort,
        RelationKind::CallPrecedes,
        RelationKind::CargoBookedOnVoyage,
        RelationKind::InspectionCoversVessel,
        RelationKind::SharesPilotageZone,
    ] {
        let mut registration = source
            .relation_registration(relation_kind_id(kind))
            .expect("source schema relation")
            .clone();
        if kind == RelationKind::VesselAssignedToBerth {
            let mut declarations = registration.relation_integrity.clone();
            declarations
                .cardinality_contracts
                .push(CardinalityContractDeclaration {
                contract_id: ContractId::new("supply_chain.vessel_source_max_one"),
                source_max: Some(1),
                source_min: None,
                target_max: None,
                target_min: None,
                pair_max: None,
                pair_min: None,
                pair_min_semantics:
                    worth_relational::facade::schema::PairMinimumSemantics::ObservedDirectedPairs,
                minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
            });
            let acyclicity = declarations.acyclicity_contracts.clone();
            let partition_isolation = declarations.partition_isolation_contracts.clone();
            let connectivity = declarations.connectivity_minimum_contracts.clone();
            registration.relation_integrity = RelationIntegrityDeclarations::new(
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
