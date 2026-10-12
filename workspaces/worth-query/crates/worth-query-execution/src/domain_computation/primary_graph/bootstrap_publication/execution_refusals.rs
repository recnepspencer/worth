//! Bootstrap's seams consume real execution refusals; production has no lease injector.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as QueryKind,
};
use std::{num::NonZeroUsize, sync::OnceLock, time::Instant};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    LeaseRequest,
};
use worth_foundational::facade::{
    aspects, AspectFieldLocator, AspectIdentity, AspectKey, AspectValue, CanonicalFieldPath,
    FieldKey, LocatorAuthority, ScalarAspectType,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    history::BranchId,
    identity::{KindId, PartitionId},
    indexes::{DerivedIndexDefinition, DerivedIndexExecutionDenialKind, DerivedIndexKind},
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntimeApi,
    schema::{
        AspectBinding, DeclaredAspectContractBinding, EntityKindRegistration,
        KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
    },
    transactions::{BulkEntityCreateIntent, RelationalExecutionDenialCause as Cause},
};
pub(in crate::domain_computation::primary_graph) fn isolated(test_name: &str, run: impl FnOnce()) {
    let test_name = test_name.split_once("::").unwrap().1;
    const PROBE: &str = "WORTH_BOOTSTRAP_INDEX_EXECUTION_PROBE";
    if std::env::var(PROBE).as_deref() == Ok(test_name) {
        run();
        return;
    }
    let output = crate::process_deadline::output(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test_name, "--nocapture"])
            .env(PROBE, test_name),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}

pub(in crate::domain_computation::primary_graph) fn authority() -> &'static ExecutionAuthority {
    static OWNER: OnceLock<ExecutionAuthority> = OnceLock::new();
    OWNER.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::MIN,
            charged_memory_bytes: Some(64 * 1024 * 1024),
        })
        .unwrap()
    })
}

pub(in crate::domain_computation::primary_graph) fn request() -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024, 1_000_000),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    }
}

fn refused_index_build(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: LeaseRequest,
) -> DerivedIndexBuildOutcome {
    let (runtime, build_request) = index_fixture(execution);
    let baseline = runtime
        .index_authority()
        .build_for_commit(execution, build_request.clone());
    assert!(baseline.execution_denial.is_none());

    assert_eq!(baseline.generations.len(), 1);
    assert!(identity_indexes_built(baseline, 1).is_ok());
    let lease = authority().request_lease(request).unwrap();
    runtime.index_authority().build_for_commit(
        &worth_execution::ExecutionRequest::leased(&lease),
        build_request,
    )
}
#[test]
fn result_refusal_is_typed_with_partition_identity() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    {
        isolated(
            concat!(
                module_path!(),
                "::result_refusal_is_typed_with_partition_identity"
            ),
            || {
                let build = refused_index_build(execution, request());
                let refusal = build.execution_denial.expect("real execution refusal");
                assert!(refusal.partition_identity.is_some());
                assert_eq!(
                    refusal.kind,
                    DerivedIndexExecutionDenialKind::Cause(Cause::ResultCapacityExceeded)
                );
                let denied = identity_indexes_built(build, 1).unwrap_err();
                assert_eq!(
                    denied.kind(),
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                        kind: QueryKind::ExecutionResource {
                            denial: Resource::ResultCapacityExceeded,
                            partition_identity: refusal.partition_identity,
                            policy_ancestor: None,
                        },
                    }
                );
            },
        );
    }
}

#[test]
fn cancellation_and_deadline_are_control_stops() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    isolated(
        concat!(
            module_path!(),
            "::cancellation_and_deadline_are_control_stops"
        ),
        || {
            let cancellation = CancellationSource::new();
            cancellation.cancel();
            let mut cancelled = request();
            cancelled.cancellation = cancellation.token();
            let mut timed_out = request();
            timed_out.deadline = Some(Instant::now());
            for (request, expected) in [
                (cancelled, Control::Cancelled),
                (timed_out, Control::TimedOut),
            ] {
                let build = refused_index_build(execution, request);
                assert!(build.execution_denial.is_some());
                let denied = identity_indexes_built(build, 1).unwrap_err();
                assert_eq!(
                    denied.kind(),
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
                        kind: expected
                    }
                );
            }
        },
    );
}

mod unleased_fixture;
use unleased_fixture::index_fixture;
#[test]
fn unleased_bootstrap_index_stop_keeps_execution_denial() {
    use worth_execution::{SerialMemoryBudget, SerialRequest};
    let setup_host = crate::test_request_policy::serial_request();
    let setup = worth_execution::ExecutionRequest::serial(&setup_host);
    let (runtime, request) = index_fixture(&setup);
    for expired in [false, true] {
        let source = CancellationSource::new();
        if !expired {
            source.cancel();
        }
        let host = SerialRequest::from_memory(
            SerialMemoryBudget::new(64 * 1024 * 1024),
            source.token(),
            expired.then(Instant::now),
        );
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let build = runtime
            .index_authority()
            .build_for_commit(&execution, request.clone());
        assert_eq!(
            build.execution_denial,
            Some(
                worth_relational::facade::indexes::DerivedIndexExecutionDenial {
                    kind: DerivedIndexExecutionDenialKind::Cause(if expired {
                        Cause::DeadlineElapsed
                    } else {
                        Cause::Cancelled
                    }),
                    partition_identity: Some(0),
                }
            )
        );
        assert_eq!(
            identity_indexes_built(build, 1).unwrap_err().kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped {
                kind: if expired {
                    Control::TimedOut
                } else {
                    Control::Cancelled
                },
            }
        );
    }
}
