//! Closing a real deferred demand retires its undelivered publication.
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
};

fn case(cancel: bool, check_entry: bool) {
    let application = support::install_program_with_seed::<application::Program>(
        None,
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard(),
        4096,
        128 * 1024 * 1024,
        1_000_000,
        |graph| {
            diamond::seed(graph);
            source_world::seed(graph);
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut other = request
        .demand(PlanarOutputDemand::new("anchor-source-b"))
        .start_in_program::<application::Program, program::ChainRoot>(&application)
        .unwrap();
    assert!(matches!(
        other.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let baseline = application.required_custody_bytes_for_test();
    let selected = request
        .query(PlanarRead {
            body_key: "diamond-left".into(),
        })
        .execute()
        .unwrap();
    let root = selected.observed_sources()[0].root_entity_for_test();
    let mut owner = request
        .demand(PlanarOutputDemand::new("diamond-left"))
        .start_in_program::<application::Program, program::ChainRoot>(&application)
        .unwrap();
    application.delay_next_output_readiness_delivery_for_test();
    assert!(matches!(
        owner.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Pending
    ));
    let published = application.published_required_rows_for_test();
    assert_eq!(
        published.len(),
        1,
        "the owner's ordinary entry really deferred delivery"
    );
    let issued = published[0].0.clone();
    assert!(application
        .queued_required_rows_for_root_for_test(root)
        .contains(&issued));
    if check_entry {
        assert_eq!(
            application.undelivered_publication_counts_for_test(),
            (1, 1)
        );
        let entered = other.advance(&request).unwrap();
        assert_eq!(
            application.entered_required_selection_is_frozen_for_test(),
            Some(true),
            "the Ready caller enters with the registry-issued Frozen selection; result is Settled={}",
            matches!(entered, WorthQueryApplicationOutputDemandProgress::Settled(_))
        );
        assert!(
            matches!(
                entered,
                WorthQueryApplicationOutputDemandProgress::Settled(_)
            ),
            "freezing unrelated rows preserves this already Ready caller's result"
        );
    }
    if cancel {
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let stopped_scope =
            authentication::WorthQueryRequestScope::new(scope.deadline(), cancellation.token());
        let stopped_request = application.request(&principal, &stopped_scope);
        cancellation.cancel();
        let stopped = owner.advance(&stopped_request);
        assert!(
            matches!(&stopped,
            Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(stop))
                if stop.kind() == WorthQueryOutputDemandDenialKind::ExecutionRequest(
                    WorthQueryAdvancementDenial::Interrupted(WorthQueryManagedComputationInterruption::Cancelled))),
            "the owner's request was cancelled: {:?}",
            stopped.as_ref().err()
        );
    }
    // A cancellation ends the request, not the reusable demand handle. Closing
    // that handle abandons its last interest rather than retrying delivery.
    drop(owner);
    assert!(
        application.published_required_rows_for_test().is_empty(),
        "last-interest release retires the authentic Published row: {published:?}"
    );
    assert_eq!(
        application.required_custody_bytes_for_test(),
        baseline,
        "the abandoned row's reserved custody returns to the baseline"
    );
    assert!(matches!(
        other.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    assert!(
        !application
            .queued_required_rows_for_root_for_test(root)
            .contains(&issued),
        "the next advance resolves this exact retired issued row through the stale-row path"
    );
}

#[test]
fn dropping_the_deferred_owner_does_not_orphan_a_published_checkpoint() {
    let _guard = checkpoint_recovery_test_guard();
    case(false, false);
}

#[test]
fn cancelling_the_deferred_owner_does_not_orphan_a_published_checkpoint() {
    let _guard = checkpoint_recovery_test_guard();
    case(true, false);
}

#[test]
fn a_different_ready_caller_enters_frozen_without_losing_its_result() {
    let _guard = checkpoint_recovery_test_guard();
    case(false, true);
}
