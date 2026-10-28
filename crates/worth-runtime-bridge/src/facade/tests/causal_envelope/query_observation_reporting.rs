//! Caller-supplied reporting associations never supply original Query admission.
use super::*;
use crate::facade::{
    BridgeCausalInspectionAdmissionSummaryKind as Kind, BridgeIdentityEvidence as Identity,
    BridgeQueryObservationReportingRequest as Request,
    BridgeQueryObservationReportingStop as Failure,
};
use std::cell::Cell;

#[test]
fn query_observation_reporting_preserves_exact_reference_without_lower_evidence_claims() {
    let admission = Identity::from_bridge_owner_external_authority("reporting:admission");
    let anchor = Identity::from_bridge_owner_external_authority("reporting:anchor");
    let observation = Identity::from_bridge_owner_external_authority("reporting:observation");
    for kind in [Kind::Admitted, Kind::Advisory] {
        let request = || Request::new(kind, &admission, &anchor, &observation).unwrap();
        let charged = Cell::new(0usize);
        let first = request()
            .assemble_with_admission(&mut |_, bytes| {
                charged.set(charged.get().checked_add(bytes).unwrap());
                Ok::<_, ()>(())
            })
            .unwrap();
        let second = request()
            .assemble_with_admission(&mut |_, _| Ok::<_, ()>(()))
            .unwrap();
        assert!(
            charged.get() > 0,
            "constructor and canonical backing reach the admission owner"
        );
        assert_eq!(
            first, second,
            "admission observation cannot change reporting meaning"
        );
        assert_eq!(first.admission_summary_kind(), kind);
        assert_eq!(first.bindings().len(), 1);
        let binding = &first.bindings()[0];
        assert_eq!(binding.owner(), BridgeCausalEvidenceOwner::Query);
        assert_eq!(
            binding.family(),
            BridgeCausalEvidenceFamily::QueryObservation
        );
        assert_eq!(binding.reference_evidence_identity(), observation);
        assert_eq!(
            binding.binding_class(),
            BridgeCausalEvidenceBindingClass::QueryObservationReport
        );
        assert_eq!(first.counters().lower_runtime_family_count(), 0);
        assert_eq!(
            first.counters().retained_native_observation_binding_count(),
            0
        );
        assert_eq!(first.counters().bridge_retained_lookup_count(), 0);
        assert_eq!(first.counters().retained_bridge_binding_count(), 0);
    }
}

#[test]
fn query_observation_reporting_preserves_the_first_allocation_refusal() {
    let identity = Identity::from_bridge_owner_external_authority("reporting:association");
    let request = Request::new(Kind::Admitted, &identity, &identity, &identity).unwrap();
    let reached = Cell::new(0usize);
    let failure = request
        .assemble_with_admission(&mut |_, _| {
            reached.set(reached.get() + 1);
            Err("original reporting allocation refusal")
        })
        .unwrap_err();
    assert!(matches!(
        failure,
        Failure::Resource("original reporting allocation refusal")
    ));
    assert_eq!(
        reached.get(),
        1,
        "no later canonical or envelope work follows refusal"
    );
}

#[test]
fn performed_source_reporting_stays_query_owned_and_distinct_from_a_query_result() {
    let admission = Identity::from_bridge_owner_external_authority("reporting:source-admission");
    let anchor = Identity::from_bridge_owner_external_authority("reporting:source-anchor");
    let original = Identity::from_bridge_owner_external_authority("reporting:source-original");
    let source = Request::from_performed_source(Kind::Admitted, &admission, &anchor, &original)
        .unwrap()
        .assemble_with_admission(&mut |_, _| Ok::<_, ()>(()))
        .unwrap();
    let read = Request::new(Kind::Admitted, &admission, &anchor, &original)
        .unwrap()
        .assemble_with_admission(&mut |_, _| Ok::<_, ()>(()))
        .unwrap();
    assert_ne!(
        source.envelope_for_reporting(),
        read.envelope_for_reporting()
    );
    assert_eq!(source.bindings().len(), 1);
    assert_eq!(
        source.bindings()[0].owner(),
        BridgeCausalEvidenceOwner::Query
    );
    assert_eq!(
        source.bindings()[0].binding_class(),
        BridgeCausalEvidenceBindingClass::QueryObservationReport
    );
    assert_eq!(source.bindings()[0].reference_evidence_identity(), original);
    assert_eq!(source.counters().lower_runtime_family_count(), 0);
    assert_eq!(
        source
            .counters()
            .retained_native_observation_binding_count(),
        0
    );
    let stopped = Request::from_performed_source(Kind::Admitted, &admission, &anchor, &original)
        .unwrap()
        .assemble_with_admission(&mut |_, _| Err("source report allocation stopped"))
        .unwrap_err();
    assert!(matches!(
        stopped,
        Failure::Resource("source report allocation stopped")
    ));
}
