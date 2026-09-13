use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
use worth_relational::facade::bridge::RuntimeBridgeRelationalSource;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};
use worth_runtime_bridge::facade::{
    BridgeAsyncRequestTruthViewBasis, BridgeAuthoritativeSourceProfile, BridgeBoundExecutionBasis,
    BridgeCommittedPatchEnvelope, BridgeDeliveryIntent, BridgeDeliveryReceipt,
    BridgeDiagnosticsTier, BridgeManagedExecutionIntent,
    BridgeManagedExecutionPartialEffectPosture, BridgeManagedExecutionStepContract,
    BridgeManagedExecutionStepLimits, BridgeMappingId, BridgeMappingRegistration, BridgeReplayMode,
    BridgeRuntimePolicy, BridgeSourceAdapter, BridgeSourceCapability, BridgeSourceCapabilitySet,
    BridgeTruthViewSelector, CoarseRoutingMode, CommittedPatchSource,
    HistoricalEvaluationDeclaration, InvalidationSink, MappingSelector,
    RelationalBridgeSourceError, RelationalCommittedPatchRequest, RuntimeBridge,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadContract,
    SnapshotReadPacket, SnapshotReadSource, SourceDeclaration, SourceDeclarationIdentity,
    TruthBranchIdentity, TruthPatchScope, TruthSnapshotIdentity, TruthSnapshotReader,
};

use super::super::{WorthQueryManagedRelationalObservation, WorthQueryManagedTruthReadRequest};

pub(super) struct CausalLowerExecutionBasis {
    pub bridge: BridgeBoundExecutionBasis,
    pub relational: WorthQueryManagedRelationalObservation,
}

pub(crate) struct CausalManagedAdmissionContext {
    pub bridge: RuntimeBridge,
    pub relational: RuntimeBridgeRelationalSource,
    pub descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    _registration: worth_relational::facade::bridge::RelationalBridgeObservationLease,
}

pub(super) struct SourceProfileSubstitutionContext {
    pub foreign_bridge: RuntimeBridge,
    pub exact_bridge: RuntimeBridge,
    pub relational: RuntimeBridgeRelationalSource,
    pub descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    _registration: worth_relational::facade::bridge::RelationalBridgeObservationLease,
}

impl CausalManagedAdmissionContext {
    pub fn read_request(&self) -> WorthQueryManagedTruthReadRequest {
        WorthQueryManagedTruthReadRequest::from_relational_basis(
            self.descriptor.clone(),
            SnapshotReadPacket::new(vec![]),
        )
    }
}

pub(crate) fn managed_admission_context() -> CausalManagedAdmissionContext {
    let mut runtime = relational_runtime();
    let committed = create_fixture_entity(&mut runtime);
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    assert!(runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .is_ok());
    let relational = RuntimeBridgeRelationalSource::for_graph_role(Arc::new(runtime), "model")
        .expect("model should be a valid graph role");
    let registration = relational.retain_branch_basis_for_bridge(&basis).unwrap();
    let registration_snapshot = registration.snapshot_identity().clone();
    let branch = crate::domain_computation::primary_graph::primary_truth_branch_identity();
    let bridge = bridge_for_source(relational.clone(), branch.clone(), registration_snapshot);
    CausalManagedAdmissionContext {
        bridge,
        relational,
        descriptor,
        _registration: registration,
    }
}

pub(super) fn source_profile_substitution_context() -> SourceProfileSubstitutionContext {
    let mut runtime = relational_runtime();
    let committed = create_fixture_entity(&mut runtime);
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    assert!(runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .is_ok());
    let relational = RuntimeBridgeRelationalSource::for_graph_role(Arc::new(runtime), "model")
        .expect("model should be a valid graph role");
    let registration = relational.retain_branch_basis_for_bridge(&basis).unwrap();
    let registration_snapshot = registration.snapshot_identity().clone();
    let branch = crate::domain_computation::primary_graph::primary_truth_branch_identity();
    let exact_bridge = bridge_for_source(
        relational.clone(),
        branch.clone(),
        registration_snapshot.clone(),
    );
    let foreign_bridge =
        bridge_for_foreign_profile(relational.clone(), branch.clone(), registration_snapshot);
    SourceProfileSubstitutionContext {
        foreign_bridge,
        exact_bridge,
        relational,
        descriptor,
        _registration: registration,
    }
}

pub(super) fn causal_lower_execution_basis(
    operation_binding_identity: &str,
    resource_attempt_identity: &str,
) -> CausalLowerExecutionBasis {
    causal_lower_execution_basis_with_snapshot_match(
        operation_binding_identity,
        resource_attempt_identity,
        true,
    )
}

pub(super) fn mismatched_snapshot_lower_execution_basis(
    operation_binding_identity: &str,
    resource_attempt_identity: &str,
) -> CausalLowerExecutionBasis {
    causal_lower_execution_basis_with_snapshot_match(
        operation_binding_identity,
        resource_attempt_identity,
        false,
    )
}

fn causal_lower_execution_basis_with_snapshot_match(
    operation_binding_identity: &str,
    resource_attempt_identity: &str,
    matching_snapshot: bool,
) -> CausalLowerExecutionBasis {
    let (source, bridge_relational, substitute, branch, snapshot) =
        relational_source_and_lease(matching_snapshot);
    let bridge = bridge_for_source(source, branch.clone(), snapshot.clone());
    let planned = bridge
        .plan_truth_view_packet(
            HistoricalEvaluationDeclaration::new(
                BridgeTruthViewSelector::branch_snapshot(branch.clone(), snapshot.clone()),
                BridgeReplayMode::Enabled,
                BridgeDiagnosticsTier::Standard,
                BridgeDeliveryIntent::PrepareSignalEvaluation,
            ),
            SnapshotReadPacket::new(vec![]),
        )
        .expect("active Relational snapshot should plan");
    let bridge = bridge
        .admit_managed_execution_basis(
            BridgeManagedExecutionIntent::new(
                operation_binding_identity,
                resource_attempt_identity,
            ),
            BridgeManagedExecutionStepContract::new(
                "managed-run-safe-point",
                BridgeManagedExecutionStepLimits::new(8, 8, 8).with_memory_ceilings(8, 8),
                BridgeManagedExecutionPartialEffectPosture::None,
            )
            .expect("managed test step contract should be bounded"),
            BridgeAsyncRequestTruthViewBasis::branch_head(branch, snapshot),
            planned,
        )
        .expect("Bridge should mint Signal authority for the exact managed intent");
    let relational = substitute.unwrap_or(bridge_relational);
    CausalLowerExecutionBasis { bridge, relational }
}

fn relational_source_and_lease(
    matching_snapshot: bool,
) -> (
    RuntimeBridgeRelationalSource,
    WorthQueryManagedRelationalObservation,
    Option<WorthQueryManagedRelationalObservation>,
    TruthBranchIdentity,
    TruthSnapshotIdentity,
) {
    let mut runtime = relational_runtime();
    let committed = create_fixture_entity(&mut runtime);
    let branch_identity = runtime.main_branch_identity();
    let (_, bridge_basis) = runtime.observe_branch(&branch_identity).unwrap();
    let substitute_basis = runtime
        .readmit_branch_basis(bridge_basis.descriptor())
        .unwrap();
    assert!(runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .is_ok());
    let source = RuntimeBridgeRelationalSource::for_graph_role(Arc::new(runtime), "model")
        .expect("model should be a valid graph role");
    let bridge_basis = WorthQueryManagedRelationalObservation::retain(&source, bridge_basis, true)
        .expect("Relational source should retain the bridge execution basis");
    let substitute = if matching_snapshot {
        None
    } else {
        Some(
            WorthQueryManagedRelationalObservation::retain(&source, substitute_basis, true)
                .expect("same version should admit an independent execution basis"),
        )
    };
    let snapshot = bridge_basis.identity().snapshot_identity().clone();
    let branch = crate::domain_computation::primary_graph::primary_truth_branch_identity();
    (source, bridge_basis, substitute, branch, snapshot)
}

fn relational_runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(
            RelationalSchemaRegistry::new()
                .register_entity_kind(EntityKindRegistration {
                    kind_id: KindId(1),
                    kind_name: "managed.run.fixture".into(),
                    schema_id: SchemaId("managed-run-fixture".into()),
                    schema_version_id: SchemaVersionId(1),
                    aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
                })
                .expect("managed-run fixture schema should register"),
        )
        .build()
}

fn create_fixture_entity(
    runtime: &mut RelationalRuntime,
) -> worth_relational::facade::transactions::CommitResult {
    let mut transaction = {
        let transaction_validation_input = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .expect("main branch binding");
        runtime
            .begin_branch_transaction(
                &transaction_validation_input,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            WorkerIntentBatch::new("managed-run-fixture").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("managed-run-entity"),
                    fields: AspectFieldPatch::new(BTreeMap::new()),
                }),
            )),
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(runtime)
        .expect("managed-run fixture entity should commit")
}

#[derive(Clone)]
struct SameRuntimeSourceAdapter {
    source: RuntimeBridgeRelationalSource,
}

#[derive(Clone)]
struct ForeignProfileRelationalSource {
    source: RuntimeBridgeRelationalSource,
}

impl CommittedPatchSource for ForeignProfileRelationalSource {
    fn authoritative_source_profile(&self) -> Option<BridgeAuthoritativeSourceProfile> {
        Some(
            BridgeAuthoritativeSourceProfile::new(
                self.source
                    .authoritative_source_profile()
                    .runtime_instance_id(),
                "foreign-managed-run-adapter",
            )
            .expect("hostile adapter profile should remain structurally valid"),
        )
    }

    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        CommittedPatchSource::load_committed_patch(&self.source, request)
    }
}

impl SnapshotReadSource for ForeignProfileRelationalSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        SnapshotReadSource::open_snapshot(&self.source, identity)
    }
}

impl BridgeSourceAdapter for SameRuntimeSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ])
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        SnapshotReadSource::open_snapshot(&self.source, identity)
    }
}

struct ManagedRunSink;

impl InvalidationSink for ManagedRunSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

fn bridge_for_source(
    source: RuntimeBridgeRelationalSource,
    branch: TruthBranchIdentity,
    snapshot: TruthSnapshotIdentity,
) -> RuntimeBridge {
    bridge_for_truth_source(source.clone(), source, branch, snapshot)
}

fn bridge_for_foreign_profile(
    source: RuntimeBridgeRelationalSource,
    branch: TruthBranchIdentity,
    snapshot: TruthSnapshotIdentity,
) -> RuntimeBridge {
    bridge_for_truth_source(
        ForeignProfileRelationalSource {
            source: source.clone(),
        },
        source,
        branch,
        snapshot,
    )
}

fn bridge_for_truth_source<S>(
    truth_source: S,
    adapter_source: RuntimeBridgeRelationalSource,
    branch: TruthBranchIdentity,
    snapshot: TruthSnapshotIdentity,
) -> RuntimeBridge
where
    S: CommittedPatchSource + SnapshotReadSource,
{
    let selector = BridgeTruthViewSelector::branch_snapshot(branch, snapshot);
    RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::development())
        .with_relational_source(truth_source)
        .with_source_adapter(SameRuntimeSourceAdapter {
            source: adapter_source,
        })
        .with_signal_sink(ManagedRunSink)
        .register_source(SourceDeclaration::new(
            SourceDeclarationIdentity::from_stable_name("managed-run-source"),
            selector,
            BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::BranchRead,
            ]),
        ))
        .register_mapping(managed_run_mapping())
        .build()
        .expect("managed-run Bridge should build")
}

fn managed_run_mapping() -> BridgeMappingRegistration {
    let aspect = AspectKey::new("managed-run").expect("valid aspect key");
    BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("managed-run-mapping"),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact("managed-run-entity"),
            aspect.clone(),
            FieldKey::new("value".to_owned()).expect("valid field key"),
        ),
        SnapshotReadContract::scalar(aspect, ScalarAspectType::String),
        SignalInvalidationScope::from_stable_name("managed-run-signal"),
        CoarseRoutingMode::Direct,
    )
}
