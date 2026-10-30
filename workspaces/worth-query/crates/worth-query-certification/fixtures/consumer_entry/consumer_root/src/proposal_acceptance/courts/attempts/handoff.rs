//! A real late request stop retains Fresh completion and cannot turn retry into replay.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
};
#[test]
fn source_completion_retry_preserves_fresh_evidence_after_a_late_request_stop() {
    let host = host::install();
    let scope = authentication::request_scope();
    let author =
        authentication::authenticate(host.installed_schema(), Actor::Author, &scope).unwrap();
    let model = Model::create(&host, &author, "model-1", 1101, 7).unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let original_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let caller =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &original_scope)
            .unwrap();
    let original = edit(&model, 3);
    let mut operation = super::super::super::application::retention_action::construct(
        &host,
        model.branch(),
        &caller,
        &original,
        super::super::super::retention::StepKey {
            attempt: 1103,
            ordinal: 0,
        },
    )
    .unwrap();
    let pause = operation.pause_after_source_completion_for_test();
    let request = host
        .request(&caller.principal, caller.scope)
        .on_branch(model.branch());
    let advanced = std::thread::scope(|workers| {
        let worker = workers.spawn(move || operation.advance(&request));
        let reached = pause.wait_until_reached(std::time::Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        pause.release();
        let advanced = worker
            .join()
            .expect("real source completion returns its owner");
        assert!(
            reached,
            "source reaches its actual completion before request handoff"
        );
        advanced
    });
    let WorthQuerySourceOnlyAdvance::Refused { operation, denial } = advanced else {
        panic!("late handoff must retain completed source custody")
    };
    assert!(matches!(
        denial,
        WorthQuerySourceOnlyDenial::Execution(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        ))
    ));
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::RetainedDeliveryPending
    );
    let fresh_scope = authentication::request_scope();
    let fresh =
        authentication::authenticate(host.installed_schema(), Actor::Reviewer, &fresh_scope)
            .unwrap();
    let before = describe(&host, &fresh, model.branch());
    let request = host
        .request(&fresh.principal, fresh.scope)
        .on_branch(model.branch());
    let WorthQuerySourceOnlyAdvance::Completed(completed) = operation.advance(&request) else {
        panic!("fresh scope recovers the original completion")
    };
    assert_eq!(completed.original_input(), &original);
    assert!(
        matches!(completed.evidence(),WorthQuerySourceCompletion::Fresh{result,..} if result.days==3)
    );
    assert_eq!(describe(&host, &fresh, model.branch()), before);
    assert_eq!(model.read(&author).unwrap().days, 3);
    completed
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("close original completion: {cause:?}"));
    model.close(&author).unwrap();
    assert!(host.branches().pending_cleanup().is_empty());
}
