//! Bootstrap's seams consume real execution refusals; production has no lease injector.

pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::isolated;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::refused_index_build;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::request;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::request_with_memory;

pub use std::num::NonZeroUsize;
pub use std::sync::OnceLock;
pub use worth_execution::CancellationSource;
pub use worth_execution::CancellationToken;
pub use worth_execution::ExecutionAuthority;
pub use worth_execution::ExecutionAuthorityConfig;
pub use worth_execution::LeaseRequest;
pub use worth_foundational::facade::aspects;
pub use worth_foundational::facade::AspectFieldLocator;
pub use worth_foundational::facade::AspectIdentity;
pub use worth_foundational::facade::AspectKey;
pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::facade::CanonicalFieldPath;
pub use worth_foundational::facade::FieldKey;
pub use worth_foundational::facade::LocatorAuthority;
pub use worth_foundational::facade::ScalarAspectType;
pub use worth_foundational::DeterminismContract;
pub use worth_foundational::ExecutionBudget;
pub use worth_foundational::ExecutionPosture;
pub use worth_foundational::ExecutionRequestPolicy;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::bootstrap_publication::identity_indexes_built;
pub use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
pub use worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphInstallationDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
pub use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as QueryKind;
pub use worth_relational::facade::history::BranchId;
pub use worth_relational::facade::identity::KindId;
pub use worth_relational::facade::identity::PartitionId;
pub use worth_relational::facade::indexes::DerivedIndexBuildRequest;
pub use worth_relational::facade::indexes::DerivedIndexDefinition;
pub use worth_relational::facade::indexes::DerivedIndexExecutionDenialKind;
pub use worth_relational::facade::indexes::DerivedIndexId;
pub use worth_relational::facade::indexes::DerivedIndexKind;
pub use worth_relational::facade::mvcc::RelationalTransactionIntent;
pub use worth_relational::facade::runtime::RelationalRuntimeApi;
pub use worth_relational::facade::schema::AspectBinding;
pub use worth_relational::facade::schema::DeclaredAspectContractBinding;
pub use worth_relational::facade::schema::EntityKindRegistration;
pub use worth_relational::facade::schema::KindAspectContractDeclarations;
pub use worth_relational::facade::schema::RelationalSchemaRegistry;
pub use worth_relational::facade::schema::SchemaId;
pub use worth_relational::facade::schema::SchemaVersionId;
pub use worth_relational::facade::symbols::ClientKey;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::BulkEntityCreateIntent;
pub use worth_relational::facade::transactions::CreateIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
pub use worth_relational::facade::transactions::WorkerIntentBatch;
#[test]
fn request_memory_refusal_is_typed_with_its_quote() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
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
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
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
            timed_out.deadline = Some({
                let clock = worth_execution::MonotonicClock::system();
                clock.deadline_at(clock.now())
            });
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
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::unleased_fixture::index_fixture;
#[test]
fn unleased_bootstrap_index_stop_keeps_execution_denial() {
    use worth_execution::SerialMemoryBudget;
    use worth_execution::SerialRequest;
    let setup_host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
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
            expired.then(|| {
                let clock = worth_execution::MonotonicClock::system();
                clock.deadline_at(clock.now())
            }),
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
