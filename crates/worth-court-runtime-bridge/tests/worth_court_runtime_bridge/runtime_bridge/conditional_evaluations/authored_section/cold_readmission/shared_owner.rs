//! Concurrent first demands use the same sealed owner and independent real branch cells.
use super::*;
#[test]
fn cold_owner_first_demands_use_actual_branch_services_from_shared_threads() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let (definitions, advances) = source(&execution);
    let medium = recorded_cut::RecordedCut::new(
        &definitions,
        &advances,
        &execution,
        "22222222-2222-4222-8222-222222222222",
    );
    let mut bindings = Bindings { providers: 0 };
    let owner: Arc<BridgeReadmittedAuthoredAssembly> = Arc::from(
        Box::new(source_free_builder())
            .readmit_authored_owner(
                medium,
                SignalRecordedIdentityContribution::empty(branch().owner()),
                &mut bindings,
                &execution,
            )
            .unwrap(),
    );
    let branches = owner.signal_owner().source_port().branches().unwrap();
    std::thread::scope(|scope| {
        let mut workers = Vec::new();
        for (identity, _) in branches {
            let owner = Arc::clone(&owner);
            workers.push(scope.spawn(move || {
                let funding = request();
                let execution = ExecutionRequest::serial(&funding);
                let source = owner.signal_owner().source_port();
                let descriptor = source.observe(identity).unwrap();
                let selected = source.readmit_descriptor(&descriptor).unwrap();
                let lowering = owner
                    .with_authored_branch(identity, |view| {
                        view.with_conditional_definitions(|items| items.next().unwrap())
                    })
                    .unwrap();
                let runtime = owner.conditional_execution(identity).unwrap();
                let binding = runtime
                    .admit_conditional_signal_basis(&lowering, selected.basis())
                    .unwrap();
                let mut value = identity.ordinal() + 40;
                let evidence = runtime
                    .execute(
                        execution,
                        &binding,
                        BridgeConditionalExecutionRequest {
                            lowering: &lowering,
                            query_binding_identity: "shared-cold-query",
                            query_capability_identity: identity.ordinal(),
                            snapshot_identity: "first-demand",
                            truth_branch_identity: None,
                            bridge_snapshot_identity: None,
                            execution_identity: "shared-cold-execution",
                            attempt: 1,
                        },
                        &mut value,
                    )
                    .unwrap();
                assert!(evidence.signal().computed_evaluation().is_some());
                assert_eq!(evidence.signal().output_version(), identity.ordinal() + 40);
            }));
        }
        for worker in workers {
            worker.join().unwrap();
        }
    });
}
