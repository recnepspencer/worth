//! A coupled assembly that ends the edit without a refusing subject is refused
//! as a custody fault, never reported as an ordinary uncommitted outcome.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialKind as Kind;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

fn refuses_typed(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(host, &request, "a");
    second::settle(host, &request, "a");
    let before = failure::checkpoint(host);
    let fault = host
        .runtime()
        .break_next_coupled_assembly_without_a_subject_for_test();
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"lost-refusal-synchronous-edit".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(fault.taken_for_test(), "the real assembly ended the edit");
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("a lost refusal must refuse typed: {outcome:?}")
    };
    assert_eq!(denial.kind(), Kind::CoupledRefusalWithoutSubject);
    failure::assert_model_unchanged(host, &before);
}

#[test]
fn a_coupled_break_without_a_subject_refuses_as_a_custody_fault() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    refuses_typed(&world::install());
}
#[test]
fn worker_only_coupled_break_without_a_subject_refuses_as_a_custody_fault() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    refuses_typed(&world::install_worker_only());
}
