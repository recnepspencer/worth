use std::sync::OnceLock;

use super::{
    bind_execution, execute_bound_strategy, CommitStrategyExecutionRegistryError,
    FrozenCommitStrategyExecutorRegistry, StrategyExecutionError,
};
use crate::commit_strategies::data::{
    CanonicalStrategyCommitRequest, CanonicalStrategyInputArtifact, CanonicalStrategyInputDigest,
    CanonicalStrategyOutputArtifact, CommitStrategyDescriptor, CommitStrategyExecutionRegistration,
    CommitStrategyExecutor, CommitStrategyFamilyName, CommitStrategyId, CommitStrategyRegistration,
    CommitStrategySemanticName, CommitStrategyVersion, PersistentArtifactName,
    StrategyExecutionResult, StrategyInputSchemaName, StrategyInputSchemaVersion,
    StrategyIntentName, StrategyMutationProgram, StrategyOutputSchemaName, StrategyPacketContract,
    StrategyReadContract, StrategyReadCostClass, StrategyReadLocalityClass, StrategyReadScopeClass,
    StrategyTraversalBasis,
};
use crate::commit_strategies::FrozenCommitStrategyRegistry;
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::runtime::builder::RelationalRuntimeBuilder;
use crate::snapshots::data::SnapshotHandle;

fn empty_result() -> StrategyExecutionResult {
    StrategyExecutionResult::new(
        CanonicalStrategyOutputArtifact::new(
            StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
            b"status=ok".to_vec(),
            PersistentArtifactName::new("strategy.intent.reconcile"),
        ),
        StrategyMutationProgram::new(Vec::<crate::transactions::data::WorkerIntentBatch>::new()),
    )
}

#[derive(Clone, Copy)]
struct EchoExecutor;

impl CommitStrategyExecutor for EchoExecutor {
    fn execute(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _request: &CanonicalStrategyCommitRequest,
        observation: &crate::commit_strategies::data::StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, crate::commit_strategies::data::StrategyExecutorFailure>
    {
        let _ = observation
            .visibility()
            .entities_in::<TestEntityProjection>(_execution, PartitionId(1))?;
        Ok(empty_result())
    }
}

#[derive(Clone, Copy)]
struct CrossPartitionExecutor;

impl CommitStrategyExecutor for CrossPartitionExecutor {
    fn execute(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _request: &CanonicalStrategyCommitRequest,
        observation: &crate::commit_strategies::data::StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, crate::commit_strategies::data::StrategyExecutorFailure>
    {
        let _ = observation
            .visibility()
            .entity::<TestEntityProjection>(_execution, EntityId::new(PartitionId(1), 0, 0))?;
        let _ = observation
            .visibility()
            .entity::<TestEntityProjection>(_execution, EntityId::new(PartitionId(2), 0, 0))?;
        Ok(empty_result())
    }
}

#[derive(Clone, Copy)]
struct UnsupportedContractExecutor;

impl CommitStrategyExecutor for UnsupportedContractExecutor {
    fn execute(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _request: &CanonicalStrategyCommitRequest,
        _observation: &crate::commit_strategies::data::StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, crate::commit_strategies::data::StrategyExecutorFailure>
    {
        Ok(empty_result())
    }
}

#[derive(Clone, Copy)]
struct PanicExecutor;

impl CommitStrategyExecutor for PanicExecutor {
    fn execute(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _request: &CanonicalStrategyCommitRequest,
        _observation: &crate::commit_strategies::data::StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, crate::commit_strategies::data::StrategyExecutorFailure>
    {
        panic!("panic containment probe")
    }
}

struct TestEntityProjection;

impl crate::visibility::materialization::read_records::EntityRecordProjection
    for TestEntityProjection
{
    const KIND: KindId = KindId(100);

    fn from_record(
        _record: crate::visibility::materialization::read_records::EntityProjectionRecord<'_>,
    ) -> Option<Self> {
        Some(Self)
    }
}

struct MissingAspectProjection;

impl crate::visibility::materialization::read_records::EntityRecordProjection
    for MissingAspectProjection
{
    const KIND: KindId = KindId(100);

    fn projection_scope() -> crate::visibility::materialization::read_records::ProjectionAspectScope
    {
        static REQUIRED: OnceLock<Box<[worth_foundational::facade::AspectKey]>> = OnceLock::new();
        crate::visibility::materialization::read_records::ProjectionAspectScope::whole_aspects(
            REQUIRED
                .get_or_init(|| {
                    vec![worth_foundational::facade::AspectKey::new("missing.aspect").unwrap()]
                        .into_boxed_slice()
                })
                .iter()
                .cloned(),
        )
    }

    fn from_record(
        _record: crate::visibility::materialization::read_records::EntityProjectionRecord<'_>,
    ) -> Option<Self> {
        Some(Self)
    }
}

fn descriptor() -> CommitStrategyDescriptor {
    CommitStrategyDescriptor::new(
        CommitStrategyId(41),
        CommitStrategySemanticName::new("strategy.intent.reconcile"),
        CommitStrategyFamilyName::new("strategy.intent"),
        CommitStrategyVersion::new(1, 0),
        StrategyIntentName::new("reconcile.desired.state"),
        StrategyInputSchemaName::new("intent.reconcile.input.v1"),
        StrategyInputSchemaVersion(1),
        StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
        StrategyReadContract {
            scope_class: StrategyReadScopeClass::PartitionBoundedScan,
            locality_class: StrategyReadLocalityClass::SinglePartition,
            traversal_basis: StrategyTraversalBasis::NoTraversal,
            packet_contract: StrategyPacketContract::ProjectionOnly,
            cost_class: StrategyReadCostClass::ORequestedSurface,
        },
        PersistentArtifactName::new("strategy.intent.reconcile"),
    )
}

fn canonical_request() -> CanonicalStrategyCommitRequest {
    let descriptor = descriptor();
    canonical_request_for(&descriptor)
}

fn unsupported_descriptor() -> CommitStrategyDescriptor {
    CommitStrategyDescriptor::new(
        CommitStrategyId(42),
        CommitStrategySemanticName::new("strategy.intent.unsupported"),
        CommitStrategyFamilyName::new("strategy.intent"),
        CommitStrategyVersion::new(1, 0),
        StrategyIntentName::new("reconcile.desired.state"),
        StrategyInputSchemaName::new("intent.reconcile.input.v1"),
        StrategyInputSchemaVersion(1),
        StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
        StrategyReadContract {
            scope_class: StrategyReadScopeClass::KindBoundedScan,
            locality_class: StrategyReadLocalityClass::CrossPartitionBounded,
            traversal_basis: StrategyTraversalBasis::NoTraversal,
            packet_contract: StrategyPacketContract::ProjectionOnly,
            cost_class: StrategyReadCostClass::OPartitionBoundedSurface,
        },
        PersistentArtifactName::new("strategy.intent.unsupported"),
    )
}

fn canonical_request_for(descriptor: &CommitStrategyDescriptor) -> CanonicalStrategyCommitRequest {
    CanonicalStrategyCommitRequest::new(
        descriptor.id(),
        descriptor.digest(),
        CanonicalStrategyInputArtifact::new(
            descriptor.input_schema_name().clone(),
            descriptor.input_schema_version(),
            b"a=1".to_vec().into(),
            CanonicalStrategyInputDigest([7; 32]),
            descriptor.artifact_name().clone(),
        ),
        crate::commit_strategies::data::StrategyCallerProvenance {
            request_origin: crate::commit_strategies::data::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        },
    )
}

#[test]
fn executor_registry_rejects_binding_without_descriptor_registration() {
    let descriptor = descriptor();
    let registry = FrozenCommitStrategyRegistry::from_registrations(Vec::new()).unwrap();

    let error = FrozenCommitStrategyExecutorRegistry::from_registrations(
        vec![CommitStrategyExecutionRegistration::new(
            &descriptor,
            EchoExecutor,
        )],
        &registry,
    )
    .unwrap_err();

    assert_eq!(
        error,
        CommitStrategyExecutionRegistryError::MissingDescriptorRegistration {
            strategy_id: CommitStrategyId(41),
            descriptor_digest: descriptor.digest(),
        }
    );
}

#[test]
fn bound_execution_executes_against_snapshot_and_contains_panics() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = descriptor();
    let registration = CommitStrategyRegistration::new(descriptor.clone()).unwrap();
    let executor_binding = CommitStrategyExecutionRegistration::new(&descriptor, EchoExecutor);
    let panic_binding = CommitStrategyExecutionRegistration::new(&descriptor, PanicExecutor);

    let runtime = RelationalRuntimeBuilder::new()
        .commit_strategy(registration.clone())
        .commit_strategy_executor(executor_binding)
        .build();
    let snapshot = runtime.visibility_authority().snapshot();
    let request = canonical_request();

    let bound = bind_execution(&runtime, &request, &snapshot, execution).expect("bind strategy");
    let draft = execute_bound_strategy(execution, bound).expect("execute strategy");

    assert_eq!(
        draft.output().schema_name().as_str(),
        "intent.reconcile.output.v1"
    );
    assert_eq!(draft.summary().projected_partition_reads, 0);
    assert_eq!(draft.summary().projected_entity_record_reads, 0);

    let panic_runtime = RelationalRuntimeBuilder::new()
        .commit_strategy(registration)
        .commit_strategy_executor(panic_binding)
        .build();
    let panic_snapshot: SnapshotHandle = panic_runtime.visibility_authority().snapshot();

    let panic_bound = bind_execution(&panic_runtime, &request, &panic_snapshot, execution)
        .expect("bind strategy");
    let error = execute_bound_strategy(execution, panic_bound).unwrap_err();
    assert_eq!(
        error,
        StrategyExecutionError::ExecutorPanicked {
            strategy_id: CommitStrategyId(41)
        }
    );
}

#[test]
fn single_partition_locality_rejects_cross_partition_strategy_reads() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = descriptor();
    let registration = CommitStrategyRegistration::new(descriptor.clone()).unwrap();
    let executor_binding =
        CommitStrategyExecutionRegistration::new(&descriptor, CrossPartitionExecutor);

    let runtime = RelationalRuntimeBuilder::new()
        .commit_strategy(registration)
        .commit_strategy_executor(executor_binding)
        .build();
    let snapshot = runtime.visibility_authority().snapshot();
    let request = canonical_request();

    let bound = bind_execution(&runtime, &request, &snapshot, execution).expect("bind strategy");
    let error = execute_bound_strategy(execution, bound).unwrap_err();

    assert!(matches!(
        error,
        StrategyExecutionError::ExecutorFailed {
            strategy_id: CommitStrategyId(41),
            failure,
        } if matches!(&failure, crate::commit_strategies::data::StrategyExecutorFailure::Domain { class, ref detail, .. } if *class
            == crate::commit_strategies::data::StrategyExecutorFailureClass::ReadContractViolation
            && detail.contains("SinglePartition locality"))
    ));
}

#[test]
fn foreign_snapshot_handle_is_rejected_for_strategy_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = descriptor();
    let registration = CommitStrategyRegistration::new(descriptor.clone()).unwrap();
    let executor_binding = CommitStrategyExecutionRegistration::new(&descriptor, EchoExecutor);

    let left_runtime = RelationalRuntimeBuilder::new()
        .commit_strategy(registration.clone())
        .commit_strategy_executor(executor_binding.clone())
        .build();
    let foreign_snapshot = left_runtime.visibility_authority().snapshot();

    let right_runtime = RelationalRuntimeBuilder::new()
        .commit_strategy(registration)
        .commit_strategy_executor(executor_binding)
        .build();
    let request = canonical_request();

    let error = match bind_execution(&right_runtime, &request, &foreign_snapshot, execution) {
        Ok(_) => panic!("foreign snapshot should be rejected"),
        Err(error) => error,
    };
    assert_eq!(
        error,
        StrategyExecutionError::UnknownSnapshot {
            snapshot_id: foreign_snapshot.snapshot_id
        }
    );
}

#[test]
fn unsupported_read_contract_is_rejected_before_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = unsupported_descriptor();
    let registration = CommitStrategyRegistration::new(descriptor.clone()).unwrap();
    let executor_binding =
        CommitStrategyExecutionRegistration::new(&descriptor, UnsupportedContractExecutor);

    let runtime = RelationalRuntimeBuilder::new()
        .commit_strategy(registration)
        .commit_strategy_executor(executor_binding)
        .build();
    let snapshot = runtime.visibility_authority().snapshot();
    let request = canonical_request_for(&descriptor);

    let error = match bind_execution(&runtime, &request, &snapshot, execution) {
        Ok(_) => panic!("unsupported read contract should be rejected"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        StrategyExecutionError::UnsupportedReadContract {
            strategy_id: CommitStrategyId(42),
            ..
        }
    ));
}

#[path = "tests/projection_denial.rs"]
mod projection_denial;
