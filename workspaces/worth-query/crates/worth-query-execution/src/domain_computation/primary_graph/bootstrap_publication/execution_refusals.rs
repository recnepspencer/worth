//! Bootstrap's seams consume real execution refusals; production has no lease injector.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel,
};
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
            charged_memory_bytes: None,
        })
        .unwrap()
    })
}

pub(in crate::domain_computation::primary_graph) fn request() -> LeaseRequest {
    request_with_memory(None)
}

/// `None` has no request-memory ceiling; only an explicitly finite caller
/// policy has memory that a build can exceed.
fn request_with_memory(charged_memory_bytes: Option<u64>) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, charged_memory_bytes, Some(1_000_000)),
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
fn request_memory_refusal_is_typed_with_its_quote() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    {
        isolated(
            concat!(
                module_path!(),
                "::request_memory_refusal_is_typed_with_its_quote"
            ),
            || {
                const POLICY: u64 = 64 * 1024;
                let build = refused_index_build(execution, request_with_memory(Some(POLICY)));
                let refusal = build.execution_denial.expect("real execution refusal");
                // The caller's own 64 KiB policy refuses the build, quoting
                // more than it holds; no stage keeps a share of its own.
                let DerivedIndexExecutionDenialKind::Cause(Cause::PolicyMemoryExhausted {
                    requested,
                    admitted,
                    ancestor,
                }) = refusal.kind
                else {
                    panic!("the request policy must refuse: {refusal:?}");
                };
                assert!(admitted <= POLICY && requested > admitted);
                let denied = identity_indexes_built(build, 1).unwrap_err();
                assert_eq!(
                    denied.kind(),
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                        kind: QueryKind::ExecutionResource {
                            denial: Resource::MemoryLimit {
                                requested,
                                admitted,
                                level: WorthQueryMemoryLimitLevel::Policy,
                            },
                            partition_identity: refusal.partition_identity,
                            policy_ancestor: Some(ancestor),
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
            SerialMemoryBudget::new(None),
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
