//! Real swallowed decision interruption cannot reach candidate authoring.
use super::*;
use crate::handler::checkpoint_polling::PollObservation;
use std::time::{Duration, Instant};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationOutputDemandDenial,
    primary_graph::{
        fact_verification_on_this_thread_for_test as take_facts, HandlerInterruption,
        WorthQueryExecutionObservation, WorthQueryInputCutoffGateForTest as Gate,
        WorthQueryInputCutoffOutcomeForTest as Outcome,
    },
};

#[test]
fn swallowed_cancellation_refuses_before_candidate_and_a_fresh_request_retries() {
    interrupted(false, false);
}

#[test]
fn swallowed_deadline_refuses_before_candidate_and_a_fresh_request_retries() {
    interrupted(true, false);
}

#[test]
fn decision_poll_retains_the_first_deadline_when_cancellation_arrives_later() {
    interrupted(true, true);
}

fn interrupted(deadline: bool, later_cancel: bool) {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let before = support::retain_read(&application, &request).unwrap();
    let output_before = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let expires = deadline.then(|| Instant::now() + Duration::from_secs(2));
    let interrupted_scope =
        authentication::WorthQueryRequestScope::new(expires, cancellation.token());
    let stopped_request = application.request(&principal, &interrupted_scope);
    let signal = cancellation.clone();
    let second_signal = cancellation.clone();
    let polling = PollObservation::arm(
        move || {
            if let Some(expires) = expires {
                std::thread::sleep(expires.saturating_duration_since(Instant::now()));
            } else {
                signal.cancel();
            }
        },
        move || {
            if later_cancel {
                second_signal.cancel();
            }
        },
    );
    let mut demand = stopped_request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls())
        .start_in_program::<CheckpointProgram, CheckpointRoot>(&application)
        .expect("the real request is live at admission");
    let refusal = (0..256)
        .find_map(|_| match demand.advance(&stopped_request) {
            Ok(
                WorthQueryApplicationOutputDemandProgress::Pending
                | WorthQueryApplicationOutputDemandProgress::Progressed,
            ) => None,
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(_)) => {
                panic!("an interrupted decision published")
            }
            Err(denial) => Some(denial),
        })
        .expect("the interrupted decision refuses");
    let expected = if deadline {
        HandlerInterruption::DeadlineExceeded
    } else {
        HandlerInterruption::Cancelled
    };
    assert_eq!(
        polling.polls(),
        [Err(expected), Err(expected)],
        "both real polls preserve the first cause"
    );
    assert_eq!(
        polling.candidates(),
        0,
        "the owner stops before build_candidate"
    );
    use worth_query_host::facade::application_contribution::{
        WorthQueryAdvancementDenial as Advancement,
        WorthQueryManagedComputationInterruption as Stop,
    };
    let expected_kind = if deadline && !later_cancel {
        WorthQueryOutputDemandDenialKind::TimedOut
    } else {
        WorthQueryOutputDemandDenialKind::Cancelled
    };
    let outer_stop = if deadline && !later_cancel {
        Stop::DeadlineExceeded
    } else {
        Stop::Cancelled
    };
    assert!(
        matches!(&refusal, WorthQueryApplicationOutputDemandDenial::Demand(cause)
        if cause.kind() == expected_kind || cause.kind() == WorthQueryOutputDemandDenialKind::ExecutionRequest(Advancement::Interrupted(outer_stop))),
        "{refusal:?}"
    );
    drop((polling, demand));
    let after = support::retain_read(&application, &request).unwrap();
    let output_after = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(before.selected_commit(), after.selected_commit());
    assert_eq!(output_before.rows(), output_after.rows());
    assert_eq!(application.world_active_publication_attempts_for_test(), 0);
    assert_eq!(application.active_application_attempts_for_test(), 0);
    drop((before, after, output_before, output_after));
    let polling = PollObservation::arm(|| {}, || {});
    let retried = direct_demand(&request, &application, "anchor-a");
    assert_eq!(
        retried.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    assert_eq!(retried.producer_contacts_in_this_demand(), 1);
    assert!(retried.application_commit_receipt().is_some());
    assert_eq!(polling.polls(), [Ok(()), Ok(())]);
    assert_eq!(polling.candidates(), 1);
}

#[test]
fn successful_polling_still_executes_and_publishes_changed_input() {
    let _guard = checkpoint_recovery_test_guard();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    take_facts();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let first = direct_demand(&request, &application, "anchor-a");
    assert_eq!(first.producer_contacts_in_this_demand(), 1);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(3),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&991_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(source);
    let polling = PollObservation::arm(|| {}, || {});
    let before = support::retain_read(&application, &request).unwrap();
    let changed = direct_demand(&request, &application, "anchor-a");
    let after = support::retain_read(&application, &request).unwrap();
    assert_eq!(
        changed.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    assert_eq!(changed.producer_contacts_in_this_demand(), 1);
    assert!(changed.application_commit_receipt().is_some());
    assert_ne!(before.selected_commit(), after.selected_commit());
    assert_eq!(polling.polls(), [Ok(()), Ok(())]);
    assert_eq!(polling.candidates(), 1);
    let observed = take_facts();
    assert_eq!(observed.input_cutoffs_omitted, 0, "{observed:?}");
    assert!(observed.input_cutoffs.iter().flatten().any(|receipt|
        receipt.gate == Gate::InputIdentity && receipt.outcome == Outcome::Fresh),
        "the actual changed input must select fresh execution: {observed:?}");
    let output = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        output.rows()[0].value,
        length(4),
        "the new declared input reaches the real author"
    );
}

/// Exercise the existing equal-input native producer without another fixture.
pub(super) fn equal_input_receipt(run: impl FnOnce()) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    take_facts();
    run();
    let observed = take_facts();
    assert_eq!(observed.input_cutoffs_omitted, 0, "{observed:?}");
    let receipts = observed.input_cutoffs.iter().flatten().collect::<Vec<_>>();
    assert!(
        receipts
            .iter()
            .any(|receipt| receipt.outcome == Outcome::Eligible),
        "{observed:?}"
    );
    assert!(
        receipts
            .iter()
            .any(|receipt| receipt.outcome == Outcome::Published
                && receipt.gate == Gate::PublicationApply),
        "{observed:?}"
    );
}
