use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy as NativeAllocationPolicy;

use worth_foundational::facade::AspectKey;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::ScalarAspectType;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::EntityKindRegistration;
use worth_relational::facade::schema::KindAspectContractDeclarations;
use worth_relational::facade::schema::RelationalSchemaRegistry;
use worth_relational::facade::schema::SchemaId;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::EntitySpec;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;
use worth_runtime_bridge::facade::BridgeAsyncRequestTruthViewBasis;
use worth_runtime_bridge::facade::BridgeAuthoritativeSourceProfile;
use worth_runtime_bridge::facade::BridgeBoundExecutionBasis;
use worth_runtime_bridge::facade::BridgeCommittedPatchEnvelope;
use worth_runtime_bridge::facade::BridgeDeliveryIntent;
use worth_runtime_bridge::facade::BridgeDeliveryReceipt;
use worth_runtime_bridge::facade::BridgeDiagnosticsTier;
use worth_runtime_bridge::facade::BridgeManagedExecutionIntent;
use worth_runtime_bridge::facade::BridgeManagedExecutionPartialEffectPosture;
use worth_runtime_bridge::facade::BridgeManagedExecutionStepContract;
use worth_runtime_bridge::facade::BridgeManagedExecutionStepLimits;
use worth_runtime_bridge::facade::BridgeMappingId;
use worth_runtime_bridge::facade::BridgeMappingRegistration;
use worth_runtime_bridge::facade::BridgeReplayMode;
use worth_runtime_bridge::facade::BridgeRuntimePolicy;
use worth_runtime_bridge::facade::BridgeSourceAdapter;
use worth_runtime_bridge::facade::BridgeSourceCapability;
use worth_runtime_bridge::facade::BridgeSourceCapabilitySet;
use worth_runtime_bridge::facade::BridgeTruthViewSelector;
use worth_runtime_bridge::facade::CoarseRoutingMode;
use worth_runtime_bridge::facade::CommittedPatchSource;
use worth_runtime_bridge::facade::HistoricalEvaluationDeclaration;
use worth_runtime_bridge::facade::InvalidationSink;
use worth_runtime_bridge::facade::MappingSelector;
use worth_runtime_bridge::facade::RelationalBridgeSourceError;
use worth_runtime_bridge::facade::RelationalCommittedPatchRequest;
use worth_runtime_bridge::facade::RuntimeBridge;
use worth_runtime_bridge::facade::RuntimeBridgeBuilder;
use worth_runtime_bridge::facade::RuntimeBridgeRelationalSource;
use worth_runtime_bridge::facade::SignalBridgeSinkError;
use worth_runtime_bridge::facade::SignalInvalidationScope;
use worth_runtime_bridge::facade::SnapshotReadContract;
use worth_runtime_bridge::facade::SnapshotReadPacket;
use worth_runtime_bridge::facade::SnapshotReadSource;
use worth_runtime_bridge::facade::SourceDeclaration;
use worth_runtime_bridge::facade::SourceDeclarationIdentity;
use worth_runtime_bridge::facade::TruthBranchIdentity;
use worth_runtime_bridge::facade::TruthPatchScope;
use worth_runtime_bridge::facade::TruthSnapshotIdentity;
use worth_runtime_bridge::facade::TruthSnapshotReader;

use worth_query_execution::court_doors_for_test::domain_computation::managed_run::relational_observation::WorthQueryManagedRelationalObservation;
use worth_query_execution::facade::domain_computation::WorthQueryManagedTruthReadRequest;

pub struct CausalLowerExecutionBasis {
    pub bridge: BridgeBoundExecutionBasis,
    pub relational: WorthQueryManagedRelationalObservation,
}

pub struct CausalManagedAdmissionContext {
    pub bridge: RuntimeBridge,
    pub relational: RuntimeBridgeRelationalSource,
    pub descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    pub _registration: worth_runtime_bridge::facade::RelationalBridgeObservationLease,
}

pub struct SourceProfileSubstitutionContext {
    pub foreign_bridge: RuntimeBridge,
    pub exact_bridge: RuntimeBridge,
    pub relational: RuntimeBridgeRelationalSource,
    pub descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    pub _registration: worth_runtime_bridge::facade::RelationalBridgeObservationLease,
}

impl CausalManagedAdmissionContext {
    pub fn read_request(&self) -> WorthQueryManagedTruthReadRequest {
        WorthQueryManagedTruthReadRequest::from_relational_basis(
            self.descriptor.clone(),
            SnapshotReadPacket::new(vec![]),
        )
    }
}

pub fn managed_admission_context(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> CausalManagedAdmissionContext {
    let mut runtime = relational_runtime();
    let committed = create_fixture_entity(execution, &mut runtime);
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
    let branch = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_branch::primary_truth_branch_identity();
    let bridge = bridge_for_source(relational.clone(), branch.clone(), registration_snapshot);
    CausalManagedAdmissionContext {
        bridge,
        relational,
        descriptor,
        _registration: registration,
    }
}

pub fn source_profile_substitution_context(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> SourceProfileSubstitutionContext {
    let mut runtime = relational_runtime();
    let committed = create_fixture_entity(execution, &mut runtime);
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
    let branch = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_branch::primary_truth_branch_identity();
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

pub fn causal_lower_execution_basis(
    operation_binding_identity: &str,
    resource_attempt_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> CausalLowerExecutionBasis {
    causal_lower_execution_basis_with_snapshot_match(
        operation_binding_identity,
        resource_attempt_identity,
        true,
        resource_request,
    )
}

pub fn mismatched_snapshot_lower_execution_basis(
    operation_binding_identity: &str,
    resource_attempt_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> CausalLowerExecutionBasis {
    causal_lower_execution_basis_with_snapshot_match(
        operation_binding_identity,
        resource_attempt_identity,
        false,
        resource_request,
    )
}

pub fn causal_lower_execution_basis_with_snapshot_match(
    operation_binding_identity: &str,
    resource_attempt_identity: &str,
    matching_snapshot: bool,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> CausalLowerExecutionBasis {
    let (source, bridge_relational, substitute, branch, snapshot) =
        relational_source_and_lease(&resource_request, matching_snapshot);
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
            resource_request,
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
            resource_request,
        )
        .expect("Bridge should mint Signal authority for the exact managed intent");
    let relational = substitute.unwrap_or(bridge_relational);
    CausalLowerExecutionBasis { bridge, relational }
}

pub fn relational_source_and_lease(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    matching_snapshot: bool,
) -> (
    RuntimeBridgeRelationalSource,
    WorthQueryManagedRelationalObservation,
    Option<WorthQueryManagedRelationalObservation>,
    TruthBranchIdentity,
    TruthSnapshotIdentity,
) {
    let mut runtime = relational_runtime();
    let committed = create_fixture_entity(execution, &mut runtime);
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
    let branch = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_branch::primary_truth_branch_identity();
    (source, bridge_basis, substitute, branch, snapshot)
}

pub fn relational_runtime() -> RelationalRuntime {
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
        .expect("valid native runtime configuration")
}

pub fn create_fixture_entity(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
            NativeAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(execution, runtime)
        .expect("managed-run fixture entity should commit")
}

#[derive(Clone)]
pub struct SameRuntimeSourceAdapter {
    pub source: RuntimeBridgeRelationalSource,
}

#[derive(Clone)]
pub struct ForeignProfileRelationalSource {
    pub source: RuntimeBridgeRelationalSource,
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
        resource_request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        CommittedPatchSource::load_committed_patch(&self.source, request, resource_request)
    }
}

impl SnapshotReadSource for ForeignProfileRelationalSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        resource_request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        SnapshotReadSource::open_snapshot(&self.source, identity, resource_request)
    }
}

pub struct ManagedRunSink;

impl InvalidationSink for ManagedRunSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
        _lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

pub fn bridge_for_source(
    source: RuntimeBridgeRelationalSource,
    branch: TruthBranchIdentity,
    snapshot: TruthSnapshotIdentity,
) -> RuntimeBridge {
    bridge_for_truth_source(source.clone(), source, branch, snapshot)
}

pub fn bridge_for_foreign_profile(
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

pub fn bridge_for_truth_source<S>(
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

pub fn managed_run_mapping() -> BridgeMappingRegistration {
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

#[path = "snapshot_adapter/mod.rs"]
pub mod snapshot_adapter;
