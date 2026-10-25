use super::*;

#[test]
fn refusing_ancestor_through_both_public_doors() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    in_isolated_process(
        concat!(
            module_path!(),
            "::refusing_ancestor_through_both_public_doors"
        ),
        || {
            let lease = authority()
                .request_lease(request(32 * 1024 * 1024, 1_000_000))
                .unwrap();
            let _held = lease.reserve_memory(32 * 1024 * 1024).unwrap();
            check_lease(execution, &lease, |stop, ancestor| {
                let Outcome::Denied(failure) = stop else {
                    panic!("wrong ancestor stop")
                };
                assert!(matches!(
                    failure.kind(),
                    Kind::ExecutionResource {
                        denial: Resource::MemoryLimit {
                            requested: 1..,
                            admitted: 0,
                            level: Level::Policy
                        },
                        policy_ancestor: Some(actual),
                        ..
                    } if actual == ancestor
                ));
            });
        },
    );
}

#[test]
fn unrelated_lease_through_public_commit() {
    in_isolated_process(
        concat!(module_path!(), "::unrelated_lease_through_public_commit"),
        || {
            use worth_execution::{ExecutionScan, MapKernelFailure, ScanOutcome};
            use worth_relational::facade::transactions::CommitExecutionDenialKind;
            let runtime = RelationalRuntimeApi::builder()
                .schema_registry(schema())
                .build();
            let before = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .0;
            let parent = authority()
                .request_lease(request(32 * 1024 * 1024, 1_000_000))
                .unwrap();
            let unrelated = authority()
                .request_lease(request(32 * 1024 * 1024, 1_000_000))
                .unwrap();
            let execution = &worth_execution::ExecutionRequest::leased(&unrelated);
            // Only a flat Copy denial crosses this probe's kernel boundary; the
            // rejected transaction and its log are dropped before returning.
            let captured = std::sync::Mutex::new(None);
            let identity = worth_foundational::PartitionIdentity::new(7);
            let scan =
                ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).unwrap();
            let outcome = scan.run(Some(&parent), (), 0, 0, 0, 0, |_, _, _| {
                let error = runtime
                    .prepare_branch_transaction(execution, transaction(&runtime, "nested-target"))
                    .expect_err("a public preparation cannot nest under an unrelated lease");
                let TransactionCommitError::Execution { denial, .. } = error else {
                    panic!("wrong nested refusal")
                };
                *captured.lock().unwrap() = Some(denial);
                Ok::<_, MapKernelFailure<()>>(((), ()))
            });
            // The kernel observes and handles the refusal as data; it does not
            // propagate a stop into the surrounding scan.
            assert!(matches!(outcome, ScanOutcome::Complete { .. }));
            let denial = captured
                .into_inner()
                .unwrap()
                .expect("public commit produced a denial");
            let CommitExecutionDenialKind::Cause(cause) = denial.kind;
            let stop = super::super::relational_execution_stop(cause, denial.partition_identity);
            let Stop::PreEffectDenied(failure) = stop else {
                panic!("nested cause must reach pre-effect refusal");
            };
            let outcome = provider_compare_denied(Compare::ProviderSession(failure));
            let Outcome::Denied(failure) = &outcome else {
                panic!("nested preparation refusal folded");
            };
            crate::domain_computation::primary_graph::conditional_operation::assert_preparation_retry(failure);
            assert_eq!(resource(outcome), Resource::NestedLeaseMisuse);
            assert_eq!(
                runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap()
                    .0,
                before
            );
        },
    );
}
