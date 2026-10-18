//! Failed producer custody remains in the actual admitted assessment demand.
use super::{host, model};
use model::{operator_identity::*, retention_entry::*, schema::*, workflow::*};
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{application_entry::*, primary_graph::*};
#[test]
fn workflow_owning_assessment_retains_failed_producer_until_fresh_owner_close() {
    let host = host::install();
    let runtime = host.workflow.runtime();
    let instance = host.begin(reviewed_document_definition("applied"), 8, 0x9176c700);
    let mut owner = host.retain_assessment(instance.clone(), 0x9176c703);
    let scope = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &scope);
    let record = runtime
        .with_host_advancement(|phase| {
            runtime
                .on_branch(instance.branch())
                .select(&phase)
                .unwrap()
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::reference(),
                    DOCUMENT_IDENTITY.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Certification,
                )
                .unwrap()
                .relational_record_identity_parts()
        })
        .unwrap();
    let _cleanup = runtime
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let pause = runtime.pause_after_application_candidate_preparation_for_test(NonZeroUsize::MIN);
    std::thread::scope(|threads| {
        let host = &host;
        let pause = &pause;
        let scope = &scope;
        let instance = &instance;
        let competitor = threads.spawn(move || {
            assert!(pause.wait_until_reached(Duration::from_secs(10)));
            host.workflow
                .runtime()
                .publish_native_field_write_for_test(
                    instance.branch(),
                    record,
                    DocumentRetentionField::reference(),
                    9_u64,
                    scope,
                )
                .unwrap();
            pause.release();
        });
        assert!(
            owner
                .advance(&runtime.request(&actor, scope).on_branch(instance.branch()))
                .is_err(),
            "actual native stale producer refuses settlement"
        );
        competitor.join().unwrap();
    });
    let failure = owner
        .managed_cleanup_failure()
        .expect("unique failure stays in admitted demand, not diagnostic clone");
    let run = failure.run_identity().to_owned();
    let session = failure.session_identity();
    let cancel=worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    cancel.cancel();
    let stopped =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancel.token(),
        );
    let (mut owner, _) = owner
        .close(
            &runtime
                .request(&actor, &stopped)
                .on_branch(instance.branch()),
        )
        .err()
        .expect("outer refusal must return actual observer before callback");
    let failure = owner.managed_cleanup_failure().unwrap();
    assert_eq!(failure.run_identity(), &run);
    assert_eq!(failure.session_identity(), session);
    let fresh = request_scope();
    let actor = authenticate_operator(runtime.installed_schema(), &fresh);
    let closed = owner
        .close(&runtime.request(&actor, &fresh).on_branch(instance.branch()))
        .unwrap_or_else(|_| panic!("same-thread fresh issuer releases actual failed producer"));
    assert!(closed.cleanup_receipt().is_some());
    assert_eq!(
        model::readback::read_retention(runtime, instance.branch()),
        9
    );
    assert!(
        matches!(
            host.advance(instance, 0x9176c704),
            WorkflowProgressOutcome::AwaitingAssessment(_)
        ),
        "physical release/observer close did not accept workflow evidence"
    );
}
