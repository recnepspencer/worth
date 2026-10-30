//! A head read cannot borrow a supplier, and custody alone is not a causal claim.
use super::*;

pub fn original_evidence_availability_requires_fresh_metadata() {
    use WorthQueryInspectionRetainedEvidence::{NotRetained, Retained};
    use WorthQueryInspectionSupplierEvidence as Supplier;
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let request = application.runtime().request(&principal, &scope);
    let head = request
        .query(query::Read {
            key: "numeric".into(),
        })
        .execute_governed(
            capability::Inspect::reference(),
            capability::InspectCausalEvidence::reference(),
            capability::Input::at("numeric"),
        )
        .unwrap();
    assert_eq!((head.rows()[0].input, head.rows()[0].calculated), (5, 0));
    let original = head.receipt().inspect().observation().unwrap().clone();
    drop(head);
    let denied = request
        .inspect::<query::Binding>(&original)
        .why_observed()
        .reference_only()
        .execute()
        .err()
        .expect("payload access cannot lend metadata admission");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    assert!(denied.resource_refusal().is_none());
    drop(denied);
    let report = request
        .inspect::<query::Binding>(&original)
        .why_observed()
        .reference_only()
        .execute_governed(access(&application, &principal, &scope))
        .unwrap();
    let availability = report.evidence_availability();
    assert_eq!(availability.projection(), NotRetained);
    assert_eq!(availability.supplier(), Supplier::NotRetained);
    assert_eq!(availability.native_observation(), NotRetained);
    report.close();

    let (before, before_row) = read(&application, &principal, &scope, None);
    assert_eq!((before_row.input, before_row.calculated), (5, 0));
    let (source, producer, read_basis) = perform(&application, 11, "availability-original-eleven");
    let (observed, row) = read(&application, &principal, &scope, Some(&read_basis));
    assert_eq!((row.input, row.calculated), (11, 23));
    drop((producer, read_basis));
    let report = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&observed)
        .why_observed()
        .reference_only()
        .execute_governed(access(&application, &principal, &scope))
        .unwrap();
    let availability = report.evidence_availability();
    assert_eq!(availability.projection(), Retained);
    assert_eq!(availability.supplier(), Supplier::CommittedOutput);
    assert_eq!(availability.native_observation(), Retained);
    assert_eq!(report.reason(), CausalInspectionReason::ObservedResult);
    assert_eq!(
        report.lower_evidence_posture(),
        CausalObservationMissingReferencePosture::MissingOptional
    );
    assert_eq!(readers.observe().active(), baseline);
    report.close();

    // A current, readable original without captured payload still cannot become
    // a comparison body. Its availability report did not materialize one.
    let unavailable = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&observed)
        .why_changed(&original)
        .reference_only()
        .execute_governed(
            access(&application, &principal, &scope),
            access(&application, &principal, &scope),
        )
        .err()
        .expect("missing original projection stays unavailable");
    assert_eq!(
        unavailable.kind(),
        WorthQueryApplicationInspectionStopKind::Unavailable
    );
    assert!(unavailable.resource_refusal().is_none());
    assert!(unavailable.canonical_refusal().is_none());
    assert!(unavailable.producer_connection_refusal().is_none());
    drop(unavailable);

    let compared = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&observed)
        .why_changed(&before)
        .reference_only()
        .execute_governed(
            access(&application, &principal, &scope),
            access(&application, &principal, &scope),
        )
        .unwrap();
    assert_eq!(compared.reason(), CausalInspectionReason::ChangedResult);
    assert_eq!(
        compared.before_evidence_availability().projection(),
        Retained
    );
    assert_eq!(
        compared.before_evidence_availability().supplier(),
        Supplier::NotRetained
    );
    assert_eq!(compared.after_evidence_availability(), availability);
    compared.close();

    let connected = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<query::Binding>(&observed)
        .why_observed()
        .caused_by::<super::super::SourceDisclosure>(&source)
        .reference_only()
        .execute_governed(
            access(&application, &principal, &scope),
            access(&application, &principal, &scope),
            access(&application, &principal, &scope),
            access(&application, &principal, &scope),
        )
        .unwrap();
    assert_eq!(connected.evidence_availability(), availability);
    assert_eq!(
        connected.producer_connection().source_commit(),
        source.commit_reference()
    );
    assert_eq!(
        connected.producer_connection().output_role(),
        "calculated-number"
    );
    connected.close();
    drop(source);
    original.close();
    before.close();
    observed.close();
    assert_eq!(readers.observe().active(), baseline);
}
