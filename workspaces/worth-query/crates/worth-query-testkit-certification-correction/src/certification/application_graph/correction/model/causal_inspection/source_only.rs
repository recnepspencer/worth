//! Actual source-only admission with no authored input in the disclosure query.
use super::{
    super::{
        authentication::*, calculation_query, edit::Value, host, other_edit::OtherEdit, schema::*,
    },
    capability, journey, SourceOnlyDisclosure,
};
use worth_query_host::facade::{
    application_entry::*,
    causal_inspection::*,
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
};

mod custody;
pub use custody::source_report_retains_fresh_request_funding_until_close;

pub(super) fn metadata(
) -> ApplicationQueryInspectionDefinition<NumericSchema, calculation_query::Binding> {
    let contract = ApplicationQueryInspectionMetadataFacet::ALL
        .into_iter()
        .fold(
            ApplicationQueryDisclosureContract::governed_by(
                "numeric-source-metadata",
                capability::Inspect::reference(),
            ),
            |contract, facet| {
                contract.disclose_inspection_metadata_by(
                    ApplicationQueryInspectionMetadataRef::<
                        NumericSchema,
                        calculation_query::Binding,
                    >::new(facet),
                    capability::text("calculation"),
                    ApplicationQueryInfluenceContract::permit([facet.influence()]),
                )
            },
        );
    ApplicationQueryInspectionDefinition::new(contract).unwrap()
}
pub(super) fn source(application: &host::Host, key: &str) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let key = key.to_owned();
    match application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(OtherEdit {
            key: "numeric".to_owned(),
            value: Value::Primary(13),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("ordinary source must commit: {other:?}"),
    }
}
pub fn source_without_a_completed_query_or_projected_input() {
    let application = host::install();
    let source = source(&application, "source-only-original");
    let expected = source.commit_reference().clone();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let readers = application.application_query_basis_observer();
    let before = readers.observe().active();
    // The declared disclosure query projects Calculated only: no Primary field,
    // completed query receipt or retained projection is supplied to inspection.
    let report = request
        .inspect_source::<SourceOnlyDisclosure>(&source)
        .why_performed()
        .reference_only()
        .execute_governed(journey::access(&application, &principal, &scope))
        .unwrap();
    assert_eq!(report.source_commit(), &expected);
    assert_eq!(
        report.lower_evidence_posture(),
        CausalObservationMissingReferencePosture::MissingOptional
    );
    assert!(!report.source_observation_identity().is_empty());
    assert_eq!(readers.observe().active(), before);
    drop(source);
    assert_eq!(report.source_commit(), &expected);
    report.close();
    assert_eq!(readers.observe().active(), before);
}
pub fn source_metadata_is_independent_of_payload_and_query_metadata() {
    let application = host::install();
    let source = source(&application, "source-only-governance");
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    // This public business query runs, but supplies no metadata capability.
    let result = request
        .query(calculation_query::Read {
            key: "numeric".into(),
        })
        .execute()
        .unwrap();
    assert!(request
        .inspect_source::<SourceOnlyDisclosure>(&source)
        .why_performed()
        .reference_only()
        .execute()
        .is_err());
    let independently_observed = request
        .query(super::query::Read {
            key: "numeric".into(),
        })
        .execute_governed(
            capability::Inspect::reference(),
            capability::InspectCausalEvidence::reference(),
            capability::Input::at("numeric"),
        )
        .unwrap();
    request
        .inspect::<super::query::Binding>(
            independently_observed
                .receipt()
                .inspect()
                .observation()
                .unwrap(),
        )
        .why_observed()
        .reference_only()
        .execute_governed(journey::access(&application, &principal, &scope))
        .unwrap()
        .close();
    assert!(
        request
            .inspect_source::<SourceOnlyDisclosure>(&source)
            .why_performed()
            .reference_only()
            .execute()
            .is_err(),
        "the other binding's admitted metadata supplies no source admission"
    );
    // Nor may the same original query observation bypass its own metadata gate.
    let original = result.receipt().inspect().observation().unwrap();
    assert!(request
        .inspect::<calculation_query::Binding>(original)
        .why_observed()
        .reference_only()
        .execute()
        .is_err());
    request
        .inspect_source::<SourceOnlyDisclosure>(&source)
        .why_performed()
        .reference_only()
        .execute_governed(journey::access(&application, &principal, &scope))
        .unwrap()
        .close();
    request
        .inspect::<calculation_query::Binding>(original)
        .why_observed()
        .reference_only()
        .execute_governed(journey::access(&application, &principal, &scope))
        .unwrap()
        .close();
    // A different source declaration cannot reinterpret this actual receipt.
    assert!(request
        .inspect_source::<super::SourceDisclosure>(&source)
        .why_performed()
        .reference_only()
        .execute()
        .is_err());
    let foreign = host::install();
    let foreign_source = self::source(&foreign, "source-only-foreign");
    assert!(request
        .inspect_source::<SourceOnlyDisclosure>(&foreign_source)
        .why_performed()
        .reference_only()
        .execute_governed(journey::access(&application, &principal, &scope))
        .is_err());
}
pub fn revoked_cancelled_and_expired_requests_release_source_inspection() {
    use worth_query_host::facade::admission::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    let application = host::install();
    let source = source(&application, "source-only-currentness");
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let old = journey::access(&application, &principal, &scope);
    cancellation.cancel();
    let refused = request
        .inspect_source::<SourceOnlyDisclosure>(&source)
        .why_performed()
        .reference_only()
        .execute_governed(old)
        .err()
        .expect("actual request cancellation refuses");
    assert_eq!(
        refused.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    drop(refused);
    assert_eq!(readers.observe().active(), baseline);
    let expired = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() - std::time::Duration::from_secs(1)),
        WorthQueryCancellationSource::new().token(),
    );
    let refused = application
        .runtime()
        .request(&principal, &expired)
        .inspect_source::<SourceOnlyDisclosure>(&source)
        .why_performed()
        .reference_only()
        .execute()
        .err()
        .expect("expired request refuses before report discovery");
    assert_eq!(
        refused.kind(),
        WorthQueryApplicationInspectionStopKind::Request
    );
    drop(refused);
    let fresh = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &fresh);
    let capability = journey::access(&application, &principal, &fresh);
    let request = application.runtime().request(&principal, &fresh);
    request
        .advanced()
        .mutate(super::super::principal_status::DisablePrincipal { identity: 1 })
        .without_source()
        .idempotency(&"source-inspection-disable".to_owned())
        .execute_in_program(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(request
        .inspect_source::<SourceOnlyDisclosure>(&source)
        .why_performed()
        .reference_only()
        .execute_governed(capability)
        .is_err());
    assert_eq!(readers.observe().active(), baseline);
}
