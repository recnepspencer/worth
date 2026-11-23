use std::sync::OnceLock;

use super::current_snapshot;
use worth_relational::facade::commit_strategies::{
    CanonicalStrategyCommitRequest, CanonicalStrategyOutputArtifact, CommitStrategyDescriptor,
    CommitStrategyFamilyName, CommitStrategyId, CommitStrategyRegistration,
    CommitStrategySemanticName, CommitStrategyVersion, NativeStrategyCommitRequest,
    PersistentArtifactName, StrategyExecutionResult, StrategyInputSchemaName,
    StrategyInputSchemaVersion, StrategyIntentName, StrategyMutationProgram,
    StrategyOutputSchemaName, StrategyPacketContract, StrategyReadContract, StrategyReadCostClass,
    StrategyReadLocalityClass, StrategyReadScopeClass, StrategyTraversalBasis,
};
use worth_relational::facade::commit_strategies::{
    CommitStrategyExecutionRegistration, CommitStrategyExecutor,
};
use worth_relational::facade::commit_strategies::{
    CommitStrategyExecutionRegistryError, StrategyExecutionError,
};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeConstructionDenial};
use worth_relational::facade::snapshots::SnapshotHandle;

fn empty_result() -> StrategyExecutionResult {
    StrategyExecutionResult::new(
        CanonicalStrategyOutputArtifact::new(
            StrategyOutputSchemaName::new("intent.reconcile.output.v1"),
            b"status=ok".to_vec(),
            PersistentArtifactName::new("strategy.intent.reconcile"),
        ),
        StrategyMutationProgram::new(Vec::<
            worth_relational::facade::transactions::WorkerIntentBatch,
        >::new()),
    )
}

#[derive(Clone, Copy)]
struct EchoExecutor;

impl CommitStrategyExecutor for EchoExecutor {
    fn execute(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _request: &CanonicalStrategyCommitRequest,
        observation: &worth_relational::facade::commit_strategies::StrategyObservationContext<'_>,
    ) -> Result<
        StrategyExecutionResult,
        worth_relational::facade::commit_strategies::StrategyExecutorFailure,
    > {
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
        observation: &worth_relational::facade::commit_strategies::StrategyObservationContext<'_>,
    ) -> Result<
        StrategyExecutionResult,
        worth_relational::facade::commit_strategies::StrategyExecutorFailure,
    > {
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
        _observation: &worth_relational::facade::commit_strategies::StrategyObservationContext<'_>,
    ) -> Result<
        StrategyExecutionResult,
        worth_relational::facade::commit_strategies::StrategyExecutorFailure,
    > {
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
        _observation: &worth_relational::facade::commit_strategies::StrategyObservationContext<'_>,
    ) -> Result<
        StrategyExecutionResult,
        worth_relational::facade::commit_strategies::StrategyExecutorFailure,
    > {
        panic!("panic containment probe")
    }
}

struct TestEntityProjection;

impl worth_relational::facade::runtime::EntityRecordProjection for TestEntityProjection {
    const KIND: KindId = KindId(100);

    fn from_record(
        _record: worth_relational::facade::runtime::EntityProjectionRecord<'_>,
    ) -> Option<Self> {
        Some(Self)
    }
}

struct MissingAspectProjection;

impl worth_relational::facade::runtime::EntityRecordProjection for MissingAspectProjection {
    const KIND: KindId = KindId(100);

    fn projection_scope() -> worth_relational::facade::runtime::ProjectionAspectScope {
        static REQUIRED: OnceLock<Box<[worth_foundational::facade::AspectKey]>> = OnceLock::new();
        worth_relational::facade::runtime::ProjectionAspectScope::whole_aspects(
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
        _record: worth_relational::facade::runtime::EntityProjectionRecord<'_>,
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

fn canonical_request_for(
    runtime: &RelationalRuntime,
    descriptor: &CommitStrategyDescriptor,
) -> CanonicalStrategyCommitRequest {
    runtime
        .commit_strategies()
        .canonicalize_request(&NativeStrategyCommitRequest::from_native_canonical_bytes(
            descriptor.semantic_name().clone(),
            b"a=1".to_vec(),
            worth_relational::facade::commit_strategies::StrategyCallerProvenance {
                request_origin:
                    worth_relational::facade::commit_strategies::StrategyRequestOrigin::Test,
                actor_identity: None,
                correlation_id: None,
            },
        ))
        .unwrap()
}

#[test]
fn executor_registry_rejects_binding_without_descriptor_registration() {
    let descriptor = descriptor();
    let error = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .commit_strategy_executor(CommitStrategyExecutionRegistration::new(
            &descriptor,
            EchoExecutor,
        ))
        .build()
        .err()
        .expect("executor without a descriptor must be refused");
    assert_eq!(
        error,
        RelationalRuntimeConstructionDenial::CommitStrategyExecutor(
            CommitStrategyExecutionRegistryError::MissingDescriptorRegistration {
                strategy_id: CommitStrategyId(41),
                descriptor_digest: descriptor.digest(),
            }
        ),
    );
}

#[test]
fn bound_execution_executes_against_snapshot_and_contains_panics() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = descriptor();
    let registration = CommitStrategyRegistration::new(descriptor.clone()).unwrap();
    let executor_binding = CommitStrategyExecutionRegistration::new(&descriptor, EchoExecutor);
    let panic_binding = CommitStrategyExecutionRegistration::new(&descriptor, PanicExecutor);

    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .commit_strategy(registration.clone())
        .commit_strategy_executor(executor_binding)
        .build()
        .expect("valid native runtime configuration");
    let snapshot = current_snapshot(&runtime);
    let request = canonical_request_for(&runtime, &descriptor);

    let draft = runtime
        .commit_strategies()
        .execute(&request, &snapshot, execution)
        .expect("execute strategy");

    assert_eq!(
        draft.output().schema_name().as_str(),
        "intent.reconcile.output.v1"
    );
    assert_eq!(draft.summary().projected_partition_reads, 0);
    assert_eq!(draft.summary().projected_entity_record_reads, 0);

    let panic_runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .commit_strategy(registration)
        .commit_strategy_executor(panic_binding)
        .build()
        .expect("valid native runtime configuration");
    let panic_snapshot: SnapshotHandle = current_snapshot(&panic_runtime);

    let error = panic_runtime
        .commit_strategies()
        .execute(&request, &panic_snapshot, execution)
        .unwrap_err();
    assert_eq!(
        error,
        StrategyExecutionError::ExecutorPanicked {
            strategy_id: CommitStrategyId(41)
        }
    );
}

#[test]
fn single_partition_locality_rejects_cross_partition_strategy_reads() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = descriptor();
    let registration = CommitStrategyRegistration::new(descriptor.clone()).unwrap();
    let executor_binding =
        CommitStrategyExecutionRegistration::new(&descriptor, CrossPartitionExecutor);

    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .commit_strategy(registration)
        .commit_strategy_executor(executor_binding)
        .build()
        .expect("valid native runtime configuration");
    let snapshot = current_snapshot(&runtime);
    let request = canonical_request_for(&runtime, &descriptor);

    let error = runtime
        .commit_strategies()
        .execute(&request, &snapshot, execution)
        .unwrap_err();

    assert!(matches!(
        error,
        StrategyExecutionError::ExecutorFailed {
            strategy_id: CommitStrategyId(41),
            failure,
        } if matches!(&failure, worth_relational::facade::commit_strategies::StrategyExecutorFailure::Domain { class, detail, .. } if *class
            == worth_relational::facade::commit_strategies::StrategyExecutorFailureClass::ReadContractViolation
            && detail.contains("SinglePartition locality"))
    ));
}

#[test]
fn foreign_snapshot_handle_is_rejected_for_strategy_execution() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = descriptor();
    let registration = CommitStrategyRegistration::new(descriptor.clone()).unwrap();
    let executor_binding = CommitStrategyExecutionRegistration::new(&descriptor, EchoExecutor);

    let left_runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .commit_strategy(registration.clone())
        .commit_strategy_executor(executor_binding.clone())
        .build()
        .expect("valid native runtime configuration");
    let foreign_snapshot = current_snapshot(&left_runtime);

    let right_runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .commit_strategy(registration)
        .commit_strategy_executor(executor_binding)
        .build()
        .expect("valid native runtime configuration");
    let request = canonical_request_for(&right_runtime, &descriptor);

    let error =
        match right_runtime
            .commit_strategies()
            .execute(&request, &foreign_snapshot, execution)
        {
            Ok(_) => panic!("foreign snapshot should be rejected"),
            Err(error) => error,
        };
    assert_eq!(
        error,
        StrategyExecutionError::UnknownSnapshot {
            snapshot_id: foreign_snapshot.snapshot_id()
        }
    );
}

#[test]
fn unsupported_read_contract_is_rejected_before_execution() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let descriptor = unsupported_descriptor();
    let registration = CommitStrategyRegistration::new(descriptor.clone()).unwrap();
    let executor_binding =
        CommitStrategyExecutionRegistration::new(&descriptor, UnsupportedContractExecutor);

    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .commit_strategy(registration)
        .commit_strategy_executor(executor_binding)
        .build()
        .expect("valid native runtime configuration");
    let snapshot = current_snapshot(&runtime);
    let request = canonical_request_for(&runtime, &descriptor);

    let error = match runtime
        .commit_strategies()
        .execute(&request, &snapshot, execution)
    {
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

mod projection_denial;
