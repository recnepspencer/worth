//! The decision-read entry refuses a foreign core or a forged tag type with no
//! request, product or graph work. Its world declares one managed-index
//! decision read, so every case reaches the entry's own decision-read branch.

use crate::process::domain_computation::primary_graph::tests::application_attempt::erased_core_forgery::*;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::fixture::decision_read_world::installed_decision_read_world;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::fixture::decision_read_world::DecisionReadAdmission;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::decision_read_world::operation::DecisionReadBinding;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::fixture::decision_read_world::DecisionReadSchema;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::fixture::decision_read_world::DecisionReadWorld;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::fixture::decision_read_world::PRINCIPAL;

/// One run of the decision-read entry, counted as `execute` counts.
pub fn execute_decision(
    world: &DecisionReadWorld,
    request: &WorthQueryRequestScope,
    admission: DecisionReadAdmission,
) -> Execution {
    let external = world.authenticate(request);
    let identities =
        ApplicationMutationIdentities::<DecisionReadSchema, DecisionReadBinding>::encode(&1, &())
            .expect("the request encodes");
    world
        .application
        .with_host_advancement(|phase| {
            let _observation =
                worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
            fact_work();
            let before = reads();
            let consults = observe_consults_for_test(|_| {});
            let (outcome, work) = world
                .application
                .execute_mutation_handler_report_with_query_reads::<DecisionReadBinding>(
                    &phase,
                    &identities,
                    &PRINCIPAL,
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
fn an_owned_decision_read_reaches_its_handler_on_the_decision_branch() {
    let world = installed_decision_read_world();
    let request = live_scope();
    let owned = world.admit(&request);
    assert!(
        !owned
            .allowed_graph_contract()
            .managed_index_reads()
            .is_empty(),
        "the fixture operation takes the decision-read branch"
    );
    let control = execute_decision(&world, &request, owned);
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
}

/// Runs one foreign admission against `target` and expects the ownership refusal.
pub fn assert_foreign_refused(
    forge: impl FnOnce(
        &DecisionReadWorld,
        &DecisionReadWorld,
        &WorthQueryRequestScope,
    ) -> DecisionReadAdmission,
) {
    let source = installed_decision_read_world();
    let target = installed_decision_read_world();
    let request = live_scope();
    let admission = forge(&source, &target, &request);
    let execution = execute_decision(&target, &request, admission);
    assert!(execution.did_nothing());
    assert_projection_refused(execution.denial, AuthorizationRefusal::ForeignRuntime);
}

#[test]
fn a_plain_foreign_admission_is_refused_before_any_work() {
    assert_foreign_refused(|source, _, request| source.admit(request));
}

#[test]
fn a_foreign_core_under_the_owned_brand_is_refused_before_any_work() {
    assert_foreign_refused(|source, target, request| {
        target
            .admit(request)
            .forge_with_core_of_for_test(source.admit(request))
    });
}

#[test]
fn an_owned_core_under_a_foreign_brand_is_refused_before_any_work() {
    assert_foreign_refused(|source, target, request| {
        source
            .admit(request)
            .forge_with_core_of_for_test(target.admit(request))
    });
}

#[test]
fn the_decision_read_entry_refuses_a_forged_tag_type_before_basis_preparation() {
    let world = installed_decision_read_world();
    let request = live_scope();
    let mut forged = world.admit(&request);
    forged.forge_tag_input_type_for_test::<u8>();
    let execution = execute_decision(&world, &request, forged);
    assert!(execution.did_nothing());
    assert!(matches!(
        execution.denial,
        Some(MutationHandlerExecutionDenial::InputNotAdmitted)
    ));
}
