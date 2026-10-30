//! No subject can lend current permissions or identity to the other subject.
use super::super::super::{authored_identity_model, query::full};
use super::*;

pub fn neither_subject_lends_capabilities_and_revocation_rechecks_each_member() {
    for withdrawn in ["inspection-grant", "observer-inspection-grant"] {
        let application = install();
        link(&application, seed::OBSERVER, "numeric", "revocation-link");
        let scope = request_scope();
        let principal = authenticate_operator(application.installed_schema(), &scope);
        let (source, _, read) = journey::perform(&application, 11, "revocation-source");
        let (after, row) =
            observation::read(&application, &principal, &scope, &read, seed::OBSERVER);
        assert_eq!(row.references, vec![("numeric".to_owned(), 23)]);
        drop(read);
        let readers = application.application_query_basis_observer();
        let baseline = readers.observe().active();
        for (query_subject, source_subject) in
            [("numeric", "numeric"), (seed::OBSERVER, seed::OBSERVER)]
        {
            let denial = application
                .runtime()
                .request(&principal, &scope)
                .inspect::<query::Binding>(&after)
                .why_observed()
                .caused_by::<SourceDisclosure>(&source)
                .reference_only()
                .execute_governed(
                    access(&application, &principal, &scope, query_subject),
                    access(&application, &principal, &scope, query_subject),
                    access(&application, &principal, &scope, source_subject),
                    access(&application, &principal, &scope, source_subject),
                )
                .err()
                .expect("valid authority for one genuine subject cannot fund the other's policy");
            admission_refusal(denial);
        }
        let qm = access(&application, &principal, &scope, seed::OBSERVER);
        let qd = access(&application, &principal, &scope, seed::OBSERVER);
        let sm = access(&application, &principal, &scope, "numeric");
        let sd = access(&application, &principal, &scope, "numeric");
        let outcome = application
            .runtime()
            .request(&principal, &scope)
            .advanced()
            .mutate(grant_status::Withdraw {
                grant: withdrawn.to_owned(),
            })
            .without_source()
            .idempotency(&format!("withdraw-{withdrawn}"))
            .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "genuine grant withdrawal: {outcome:?}"
        );
        let denial = application
            .runtime()
            .request(&principal, &scope)
            .inspect::<query::Binding>(&after)
            .why_observed()
            .caused_by::<SourceDisclosure>(&source)
            .reference_only()
            .execute_governed(qm, qd, sm, sd)
            .err()
            .expect("current source or query revocation precedes cause lookup");
        admission_refusal(denial);
        assert_eq!(readers.observe().active(), baseline);
        after.close();
        assert_eq!(readers.observe().active(), baseline);
    }
}

pub fn original_query_subject_refuses_replacement_generation_and_cancelled_inspection() {
    let application = install();
    link(&application, seed::OBSERVER, "numeric", "generation-link");
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (source, _, read) = journey::perform(&application, 11, "generation-source");
    let (after, row) = observation::read(&application, &principal, &scope, &read, seed::OBSERVER);
    assert_eq!(row.key, seed::OBSERVER);
    drop(read);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let request = application.runtime().request(&principal, &scope);
    let before = request
        .query(full::Read {
            key: seed::OBSERVER.to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .entity_id;
    let cancellation = WorthQueryCancellationSource::new();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token());
    let qm = access(&application, &principal, &scope, seed::OBSERVER);
    let qd = access(&application, &principal, &scope, seed::OBSERVER);
    let sm = access(&application, &principal, &scope, "numeric");
    let sd = access(&application, &principal, &scope, "numeric");
    cancellation.cancel();
    let denied = application
        .runtime()
        .request(&principal, &cancelled)
        .inspect::<query::Binding>(&after)
        .why_observed()
        .caused_by::<SourceDisclosure>(&source)
        .reference_only()
        .execute_governed(qm, qd, sm, sd)
        .err()
        .expect("cancelled inspection cannot expose cause details");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    assert!(denied.producer_connection_refusal().is_none());
    drop(denied);
    let qm = access(&application, &principal, &scope, seed::OBSERVER);
    let qd = access(&application, &principal, &scope, seed::OBSERVER);
    let sm = access(&application, &principal, &scope, "numeric");
    let sd = access(&application, &principal, &scope, "numeric");
    let retired = request
        .advanced()
        .mutate(authored_identity_model::Retire {
            key: "numeric".to_owned(),
            target: seed::OBSERVER.to_owned(),
            wrong_restore_key: false,
        })
        .without_source()
        .idempotency(&"retire-observer".to_owned())
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            retired,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "actual retirement: {retired:?}"
    );
    let recreated = request
        .advanced()
        .mutate(authored_identity_model::Restore {
            key: "numeric".to_owned(),
            name: seed::OBSERVER.to_owned(),
            primary: 17,
            secondary: 0,
            read_enabled: true,
            ceiling: 1000,
            calculated: 0,
        })
        .without_source()
        .idempotency(&"restore-observer".to_owned())
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            recreated,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "actual replacement: {recreated:?}"
    );
    let current = request
        .query(full::Read {
            key: seed::OBSERVER.to_owned(),
        })
        .execute()
        .unwrap();
    assert_ne!(
        current.rows()[0].entity_id,
        before,
        "same selector acquired a genuinely different native generation"
    );
    drop(current);
    let denied = request
        .inspect::<query::Binding>(&after)
        .why_observed()
        .caused_by::<SourceDisclosure>(&source)
        .reference_only()
        .execute_governed(qm, qd, sm, sd)
        .err()
        .expect("a replacement generation cannot stand in for the original query subject");
    admission_refusal(denied);
    assert_eq!(readers.observe().active(), baseline);
    after.close();
    assert_eq!(readers.observe().active(), baseline);
}
fn admission_refusal(denial: WorthQueryApplicationInspectionDenial) {
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    assert!(denial.producer_connection_refusal().is_none());
    assert!(denial.producer_output_refusal().is_none());
}
