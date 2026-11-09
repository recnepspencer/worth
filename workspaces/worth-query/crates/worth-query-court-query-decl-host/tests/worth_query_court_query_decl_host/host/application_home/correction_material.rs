//! Public directory originals and corrections use the shared installed handler.
mod expiry;
mod prepared_reopen;
mod program;
use program::*;
use super::*;
use worth_query_host::facade::application_entry::{WorthQueryApplicationMutationOutcome as Outcome,
    WorthQueryCorrectionHistoryLimits, WorthQueryApplicationCorrectionPreparation as Prepared};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture as label;
use label::correction_source::label_edit::*;
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
#[test]
fn home_correction_uses_original_material_and_the_durable_key_after_graceful_reopen() {
    let location = fixture::location();
    let lease = checkpoint_capture::lease(256 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(location.clone(), execution);
    let receipt = edit(&home);
    let coordinate = worth_query_host::facade::primary_graph::WorthQueryRecordedCorrectionSource::from_untrusted_parts(
        receipt.committed_product_publication().composite_commit().clone(), receipt.commit_reference().clone()).coordinates();
    drop(receipt);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("before correction close: {refused:?}"))
        .reclaim();
    let home = open(location.clone(), execution);
    let request = scope();
    let limits =
        WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::new(8).unwrap());
    {
        let runtime = home.runtime();
        let principal = label::authentication::authenticate_external(
            runtime.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &request,
        );
        let corrections = runtime.request(&principal, &request).corrections();
        let source = corrections.select_coordinates(&coordinate, limits).unwrap();
        let prepared = runtime
            .request(&principal, &request)
            .corrections()
            .reverse(&source)
            .idempotency("reverse-label")
            .limits(limits)
            .prepare()
            .unwrap();
        let Prepared::Prepared(prepared) = prepared else {
            panic!("first correction prepares exactly once");
        };
        match prepared.publish().unwrap() {
            Outcome::Committed { result, .. } => {
                assert_eq!(result.downcast_ref::<LabelEdit>().unwrap().label, "primary")
            }
            other => panic!("actual ordinary inverse: {other:?}"),
        }
        drop(source);
    }
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("after correction close: {refused:?}"))
        .reclaim();
    let home = open(location, execution);
    let runtime = home.runtime();
    let principal = label::authentication::authenticate_external(
        runtime.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &request,
    );
    let source = runtime
        .request(&principal, &request)
        .corrections()
        .select_coordinates(&coordinate, limits)
        .unwrap();
    let retry = runtime
        .request(&principal, &request)
        .corrections()
        .reverse(&source)
        .idempotency("reverse-label")
        .limits(limits)
        .prepare()
        .unwrap();
    assert!(
        matches!(retry, Prepared::Settled(Outcome::RecoveredCommitted(_))),
        "the same key observes the original durable inverse"
    );
    drop(retry);
    drop(source);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("retry close: {refused:?}"))
        .reclaim();
}
