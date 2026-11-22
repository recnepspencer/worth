//! A partitioned computation's owner probes the declared managed Parent through
//! its lent reader: the operation's own issued decision read, never a sweep.
use super::super::{handler::DecisionRead, probe};
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryComputationManagedIndexDenial, WorthQueryComputationManagedIndexDenialKind,
    WorthQueryComputationReadDenial, WorthQueryPartitionedComputationDenial,
};

fn world(probe: probe::Probe) -> (host::Host, Arc<prefix_observation::Calls>) {
    let provider = Arc::new(prefix_observation::Calls::default());
    let rules = Arc::new(rule::Calls::new(Arc::clone(&provider)));
    let world = host::install_decision_read(
        Arc::clone(&provider),
        rules,
        DecisionRead::Partitioned(probe),
    );
    (world, provider)
}
fn principal(
    world: &host::Host,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<super::super::schema::Catalog> {
    authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            scope,
        ),
    )
    .unwrap()
}
fn completed() -> Vec<Vec<(String, String)>> {
    probe::COMPLETED.with(|runs| std::mem::take(&mut *runs.borrow_mut()))
}
/// The computation's typed managed index read refusal inside the handler stop.
fn read_denial(cause: HandlerExecutionDenial) -> WorthQueryComputationManagedIndexDenial {
    let cause = cause
        .downcast::<probe::Denial>()
        .expect("the handler forwards the computation's own denial");
    let WorthQueryPartitionedComputationDenial::Read(
        WorthQueryComputationReadDenial::ManagedIndex(denial),
    ) = cause
    else {
        panic!("the membership call's managed index read refused: {cause:?}")
    };
    denial
}

#[test]
fn computation_index_query_reads_the_committed_parent_afresh_on_every_full_run() {
    lifecycle::run(|| {
        let (world, provider) = world(probe::Probe::Declared);
        let scope = authentication::request_scope();
        let principal = principal(&world, &scope);
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let a = request
            .query(find("A", "sh"))
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
        completed();
        // Each name is a fact the Parent row reads; the authored universe is
        // the independent oracle of what the committed basis selects.
        for (serial, (before, after)) in [("shared", "shore"), ("shore", "shelf")]
            .into_iter()
            .enumerate()
        {
            let source = request.query(find("A", "sh")).execute().unwrap();
            let probes = provider.probe_validations.load(Ordering::SeqCst);
            let outcome = request
                .advanced()
                .mutate(rename("sh", after))
                .expect_result_set(source.result_set_observation().unwrap().clone())
                .idempotency(&(0x2760_0000u64 + serial as u64))
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            assert!(
                matches!(
                    outcome,
                    WorthQueryApplicationMutationOutcome::Committed { .. }
                ),
                "{outcome:?}"
            );
            assert_eq!(
                completed(),
                vec![expected("A", before, "sh")],
                "one run read exactly the committed Parent, including the edited name"
            );
            assert!(
                provider.probe_validations.load(Ordering::SeqCst) > probes,
                "a non-producer computation is a full run: the Parent is probed again"
            );
            assert_eq!(
                values(request.query(find("A", "")).execute().unwrap().rows()),
                expected("A", after, "")
            );
        }
        a.close();
    });
}

#[test]
fn computation_index_query_requires_the_declared_parent_and_the_operation_scope() {
    lifecycle::run(|| {
        for probe in [probe::Probe::OtherScope, probe::Probe::Undeclared] {
            let (world, provider) = world(probe);
            let scope = authentication::request_scope();
            let principal = principal(&world, &scope);
            let request = world
                .runtime()
                .request(&principal, &scope)
                .on_branch(world.current_world());
            let a = request
                .query(find("A", "sh"))
                .open_index::<prefix::PrefixIndex>()
                .unwrap();
            let b = request
                .query(find("B", "sh"))
                .open_index::<prefix::PrefixIndex>()
                .unwrap();
            let head = request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone();
            let source = request.query(find("A", "sh")).execute().unwrap();
            let probes = provider.probe_validations.load(Ordering::SeqCst);
            completed();
            let result = request
                .advanced()
                .mutate(rename("sh", "wrong"))
                .expect_result_set(source.result_set_observation().unwrap().clone())
                .idempotency(&0x2760_0100u64)
                .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
            let Err(WorthQueryApplicationRequestMutationDenial::Handler(
                MutationHandlerExecutionDenial::Handler(cause),
            )) = result
            else {
                panic!("the computation's read must reach its own typed refusal: {result:?}")
            };
            let denial = read_denial(cause);
            match probe {
                probe::Probe::OtherScope => {
                    assert_eq!(
                        denial.kind(),
                        WorthQueryComputationManagedIndexDenialKind::ManagedIndex
                    );
                    assert!(
                        matches!(
                            denial.managed_index_denial().unwrap().cause(),
                            WorthQueryManagedIndexDenialCause::Refused(
                                WorthQueryManagedIndexRefusal::ForeignAssociation
                            )
                        ),
                        "{denial:?}"
                    );
                }
                probe::Probe::Undeclared => {
                    assert_eq!(
                        denial.kind(),
                        WorthQueryComputationManagedIndexDenialKind::UndeclaredDecisionRead
                    );
                    assert_eq!(
                        denial.decision_plan_denial().unwrap().kind(),
                        WorthQueryInvariantDecisionPlanDenialKind::UndeclaredDecisionTarget
                    );
                }
                probe::Probe::Declared | probe::Probe::Prefix(_) => unreachable!(),
            }
            assert_eq!(
                denial.clone(),
                denial,
                "the refusal is retained and comparable"
            );
            assert_eq!(
                provider.probe_validations.load(Ordering::SeqCst),
                probes,
                "binding and scope refusal precede every provider query"
            );
            assert!(
                completed().is_empty(),
                "no partition ran on a refused membership"
            );
            assert_eq!(
                request.retain_current_read().unwrap().selected_commit(),
                &head
            );
            a.close();
            b.close();
        }
    });
}

#[test]
fn computation_index_query_carries_original_request_cancellation() {
    lifecycle::run(|| {
        // Only the owner probes "sh"; the mutation's own expectation is "sha".
        let (world, provider) = world(probe::Probe::Prefix("sh"));
        let scope = authentication::request_scope();
        let principal = principal(&world, &scope);
        let opened = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let a = opened
            .query(find("A", "sh"))
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
        let source = opened.query(find("A", "sha")).execute().unwrap();
        let head = opened
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let cancellation = WorthQueryCancellationSource::new();
        let stopped = WorthQueryRequestScope::new(
            None,
            cancellation.token(),
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        );
        let request = world
            .runtime()
            .request(&principal, &stopped)
            .on_branch(world.current_world());
        // The provider's genuine "sh" probe validation cancels the original
        // request inside the computation's own decision read.
        *provider.at_probe.lock().unwrap() =
            Some(("sh".into(), Box::new(move || cancellation.cancel())));
        completed();
        let result = request
            .advanced()
            .mutate(rename("sha", "stopped"))
            .expect_result_set(source.result_set_observation().unwrap().clone())
            .idempotency(&0x2760_0200u64)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
        assert!(
            provider.at_probe.lock().unwrap().is_none(),
            "the owner's own read reached the provider"
        );
        // The computation's read refused with the request's cancellation, as
        // its typed, comparable managed index denial.
        let refused = probe::REFUSED.with(|refused| std::mem::take(&mut *refused.borrow_mut()));
        let [denial] = refused.as_slice() else {
            panic!("exactly the owner's read refused: {refused:?}")
        };
        assert_eq!(
            denial.kind(),
            WorthQueryComputationManagedIndexDenialKind::ManagedIndex
        );
        assert!(
            matches!(
                denial.managed_index_denial().unwrap().cause(),
                WorthQueryManagedIndexDenialCause::Work(
                    WorthQueryManagedIndexWorkDenial::Controlled(
                        WorthQueryManagedIndexStop::Request(
                            worth_execution::WorkCeilingDenial::Stopped(
                                worth_execution::MapKernelStop::Cancelled
                            )
                        )
                    )
                )
            ),
            "{denial:?}"
        );
        // The original request reports its own cancellation; nothing commits.
        let Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Projection(projection),
        )) = result
        else {
            panic!("the cancelled request stops before any candidate: {result:?}")
        };
        assert_eq!(
            projection.kind(),
            WorthQueryOperationProjectionDenialKind::InvariantAdmission(
                WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
                    RelationalExecutionDenialCause::Cancelled
                )
            )
        );
        assert!(
            completed().is_empty(),
            "no partition ran after the stopped read"
        );
        assert_eq!(
            opened.retain_current_read().unwrap().selected_commit(),
            &head
        );
        a.close();
    });
}
