//! The decision-read entry refuses a foreign core with no request, product or
//! graph work. The fixture operation declares no managed-index reads, so this
//! courts the entry, not its decision-read branch.

use super::*;

/// One run of the decision-read entry, counted as `execute` counts.
fn execute_with_query_reads(
    world: &AuthorizationWorld,
    principal: u64,
    request: &WorthQueryRequestScope,
    admission: Admission,
) -> Execution {
    let external = world.authenticate("alice", Duration::from_secs(60), request);
    let key = "forgery".to_owned();
    let input = ProgramRequiredInput::new("open");
    let identities = ApplicationMutationIdentities::<
        IdentityExecutionSchema,
        ProgramRequiredMutationBinding,
    >::encode(&key, &input)
    .expect("the request encodes");
    world
        .application
        .with_host_advancement(|phase| {
            let _observation =
                crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
            fact_work();
            let before = reads();
            let consults = observe_consults_for_test(|_| {});
            let (outcome, work) = world
                .application
                .execute_mutation_handler_report_with_query_reads::<ProgramRequiredMutationBinding>(
                    &phase,
                    &identities,
                    &principal,
                    &external,
                    admission,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .into_parts();
            Execution {
                denial: outcome.err(),
                work,
                source_reads: reads() - before,
                source_fact_admissions: fact_work().source_admission_visits,
                request_consults: consult_count_for_test(&consults),
                request_charges: charged_prefixes_for_test(&consults).len(),
            }
        })
        .expect("fixture owner admits its advancement")
}

#[test]
fn the_decision_read_entry_refuses_a_foreign_core_before_any_work() {
    let source = installed_authorization_world(true);
    let target = installed_authorization_world(true);
    let request = live_scope();
    let (source_principal, source_account) = party(&source, &request);
    let (target_principal, target_account) = party(&target, &request);
    let identity = *target_principal.principal_identity();
    let source_admission = || {
        admit!(
            source,
            &source_principal,
            &source_account,
            ProgramRequiredOperation,
            &request
        )
    };
    let target_admission = || {
        admit!(
            target,
            &target_principal,
            &target_account,
            ProgramRequiredOperation,
            &request
        )
    };

    let control = execute_with_query_reads(&target, identity, &request, target_admission());
    assert!(
        matches!(
            control.work,
            WorthQueryMutationHandlerWork::Captured(work) if work.handler_contacted()
        ),
        "the owned admission reaches its handler: {:?}",
        control.denial
    );
    assert!(
        control.request_consults > 0,
        "the same entry consults its request"
    );

    let foreign = execute_with_query_reads(&target, identity, &request, source_admission());
    assert!(foreign.did_nothing());
    assert_projection_refused(foreign.denial, AuthorizationRefusal::ForeignRuntime);

    let foreign_core = target_admission().forge_with_core_of_for_test(source_admission());
    let execution = execute_with_query_reads(&target, identity, &request, foreign_core);
    assert!(execution.did_nothing());
    assert_projection_refused(execution.denial, AuthorizationRefusal::ForeignRuntime);

    let foreign_brand = source_admission().forge_with_core_of_for_test(target_admission());
    let execution = execute_with_query_reads(&target, identity, &request, foreign_brand);
    assert!(execution.did_nothing());
    assert_projection_refused(execution.denial, AuthorizationRefusal::ForeignRuntime);
}
