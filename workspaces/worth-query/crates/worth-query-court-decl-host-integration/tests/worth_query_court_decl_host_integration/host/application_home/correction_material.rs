//! Public directory original operations use the shared installed handler.
mod handler_ergonomics;
mod kill_original_owner;
pub(in crate::host) mod numeric_home;
mod original_operation;
mod program;
mod refused_custody;
use program::*;
use super::*;
use worth_query_host::facade::application_entry::{WorthQueryApplicationMutationOutcome as Outcome};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture as label;
use label::label_edit::*;
fn scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(Some(Instant::now() + Duration::from_secs(120)),
        WorthQueryCancellationSource::new().token(),
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
}
fn open(
    location: std::path::PathBuf,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Home {
    with_bootstrap_advancement_from_request(execution, |phase| {
        program::request(location).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("actual correction Home: {refused:?}"))
}
fn edit(
    home: &Home,
) -> worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt {
    let request = scope();
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(
        runtime.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &request,
    );
    let outcome = runtime
        .request(&principal, &request)
        .advanced()
        .mutate(LabelEdit {
            status: "open".into(),
            label: "changed".into(),
        })
        .without_source()
        .idempotency(&"original-label".to_owned())
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    match outcome {
        Outcome::Committed { receipt, result } => {
            assert_eq!(result.label, "changed");
            receipt
        }
        other => panic!("actual performed original: {other:?}"),
    }
}
fn count(home: &Home, execution: &worth_execution::ExecutionRequest<'_, '_>) -> usize {
    use worth_query_host::facade::{
        application_installation::WorthQueryProgramSupportRetirementFailure as Failure,
        domain::WorthQueryProgramSupportRetirementDenial as Denial,
    };
    let revision = *home.runtime().installed_program().revision();
    match home.runtime().retire_program_support(execution, &revision) {
        Err(Failure::Policy(Denial::CurrentBranches(inventory))) => {
            inventory.retained_interpretations()
        }
        other => panic!("current program retirement observes actual custody: {other:?}"),
    }
}
#[test]
fn acknowledged_home_original_transfers_its_single_interpretation_across_graceful_reopen() {
    let location = fixture::location();
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(location.clone(), execution);
    drop(edit(&home));
    assert_eq!(
        count(&home, &execution),
        1,
        "the actual producer replaces its local count with one durable membership"
    );
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("original close: {refused:?}"))
        .reclaim();
    let home = open(location, execution);
    assert_eq!(
        count(&home, &execution),
        1,
        "readmission never resets or doubles the membership"
    );
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("original reopened close: {refused:?}"))
        .reclaim();
}
