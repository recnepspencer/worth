use super::super::super::inspection::{
    causal_evidence_reference_index, causal_evidence_reference_index_record,
};
use super::super::super::*;

fn changed_anchor_with_route_signal_and_inspection() -> CausalObservationAnchor {
    anchor_causal_observation(
        QueryObservationReceipt::fixture(
            CausalObservationOutcome::Changed,
            vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "indexed-query-inspection-reference",
                    ),
                )
                .expect("fixture digest must fit"),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgeRoute,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "indexed-bridge-route-reference",
                    ),
                )
                .expect("fixture digest must fit"),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::SignalInvalidation,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "indexed-signal-invalidation-reference",
                    ),
                )
                .expect("fixture digest must fit"),
            ],
        ),
        CausalInspectionReason::ChangedResult,
    )
    .unwrap()
}

#[test]
fn indexed_evidence_reference_resolution_uses_index_records_without_scan_fallback() {
    let anchor = changed_anchor_with_route_signal_and_inspection();
    let anchor_digest = anchor.anchor_digest().clone();
    let index = causal_evidence_reference_index([
        causal_evidence_reference_index_record(
            CausalEvidenceOwner::RuntimeBridge,
            CausalEvidenceFamily::BridgeRoute,
            crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                "indexed-bridge-route-reference",
            ),
        )
        .unwrap(),
        causal_evidence_reference_index_record(
            CausalEvidenceOwner::Signal,
            CausalEvidenceFamily::SignalInvalidation,
            crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                "indexed-signal-invalidation-reference",
            ),
        )
        .unwrap(),
    ])
    .expect("fixture digest must fit");

    let resolution = resolve_indexed_causal_evidence_references(
        anchor,
        &[
            CausalEvidenceFamily::BridgeRoute,
            CausalEvidenceFamily::SignalInvalidation,
        ],
        &index,
    )
    .expect("fixture digest must fit");

    let CausalEvidenceReferenceResolution::Resolved {
        reference_set,
        counters,
    } = resolution
    else {
        panic!("expected indexed causal references to resolve");
    };

    assert_eq!(reference_set.anchor().anchor_digest(), &anchor_digest);
    assert_eq!(reference_set.references().len(), 2);
    assert!(reference_set.references().iter().any(|reference| {
        reference.owner() == CausalEvidenceOwner::RuntimeBridge
            && reference.family() == CausalEvidenceFamily::BridgeRoute
            && reference.reference_digest()
                == &crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "indexed-bridge-route-reference",
                )
    }));
    assert!(reference_set.references().iter().any(|reference| {
        reference.owner() == CausalEvidenceOwner::Signal
            && reference.family() == CausalEvidenceFamily::SignalInvalidation
            && reference.reference_digest()
                == &crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "indexed-signal-invalidation-reference",
                )
    }));
    assert_eq!(index.record_count(), 2);
    assert_eq!(index.family_count(), 2);
    assert!(!index.index_digest().is_empty());
    assert_eq!(counters.requested_family_count(), 2);
    assert_eq!(counters.anchor_reference_width(), 3);
    assert_eq!(counters.indexed_record_count(), 2);
    assert_eq!(counters.index_lookup_count(), 2);
    assert_eq!(counters.resolved_reference_count(), 2);
    assert_eq!(counters.missing_required_reference_count(), 0);
    assert_eq!(counters.bridge_record_unindexed_scan_count(), 0);
    assert_eq!(counters.retained_record_scan_count(), 0);
    assert_eq!(counters.runtime_graph_scan_count(), 0);
}

#[test]
fn indexed_reference_resolution_denies_missing_index_record_for_anchor_carried_family() {
    let anchor = changed_anchor_with_route_signal_and_inspection();
    let anchor_digest = anchor.anchor_digest().clone();
    let index = causal_evidence_reference_index([causal_evidence_reference_index_record(
        CausalEvidenceOwner::Query,
        CausalEvidenceFamily::QueryInspection,
        crate::runtime::tests::causal_inspection::causal_test_reference_digest(
            "indexed-query-inspection-reference",
        ),
    )
    .unwrap()])
    .expect("fixture digest must fit");

    let resolution = resolve_indexed_causal_evidence_references(
        anchor,
        &[CausalEvidenceFamily::BridgeRoute],
        &index,
    )
    .expect("fixture digest must fit");

    let CausalEvidenceReferenceResolution::MissingRequiredEvidence { denial, counters } =
        resolution
    else {
        panic!("expected missing indexed bridge record to deny resolution");
    };

    assert_eq!(denial.anchor_digest(), &anchor_digest);
    assert!(denial.missing_families().is_empty());
    assert_eq!(denial.missing_indexed_reference_count(), 1);
    assert_eq!(counters.requested_family_count(), 1);
    assert_eq!(counters.anchor_reference_width(), 3);
    assert_eq!(counters.indexed_record_count(), 1);
    assert_eq!(counters.index_lookup_count(), 1);
    assert_eq!(counters.resolved_reference_count(), 0);
    assert_eq!(counters.missing_required_reference_count(), 1);
    assert_eq!(counters.bridge_record_unindexed_scan_count(), 0);
    assert_eq!(counters.retained_record_scan_count(), 0);
    assert_eq!(counters.runtime_graph_scan_count(), 0);
}

#[test]
fn indexed_reference_resolution_ignores_unrelated_retained_index_records() {
    let anchor = changed_anchor_with_route_signal_and_inspection();
    let index = causal_evidence_reference_index([
        causal_evidence_reference_index_record(
            CausalEvidenceOwner::RuntimeBridge,
            CausalEvidenceFamily::BridgeRoute,
            crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                "stale-bridge-route-reference",
            ),
        )
        .unwrap(),
        causal_evidence_reference_index_record(
            CausalEvidenceOwner::RuntimeBridge,
            CausalEvidenceFamily::BridgeRoute,
            crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                "indexed-bridge-route-reference",
            ),
        )
        .unwrap(),
        causal_evidence_reference_index_record(
            CausalEvidenceOwner::Signal,
            CausalEvidenceFamily::SignalEvaluation,
            crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                "unrelated-signal-evaluation-reference",
            ),
        )
        .unwrap(),
    ])
    .expect("fixture digest must fit");

    let resolution = resolve_indexed_causal_evidence_references(
        anchor,
        &[CausalEvidenceFamily::BridgeRoute],
        &index,
    )
    .expect("fixture digest must fit");

    let CausalEvidenceReferenceResolution::Resolved {
        reference_set,
        counters,
    } = resolution
    else {
        panic!(
            "expected exact indexed bridge record to resolve despite unrelated retained records"
        );
    };

    assert_eq!(reference_set.references().len(), 1);
    assert_eq!(index.record_count(), 3);
    assert_eq!(counters.requested_family_count(), 1);
    assert_eq!(counters.anchor_reference_width(), 3);
    assert_eq!(counters.indexed_record_count(), 1);
    assert_eq!(counters.index_lookup_count(), 1);
    assert_eq!(counters.resolved_reference_count(), 1);
    assert_eq!(counters.missing_required_reference_count(), 0);
    assert_eq!(counters.bridge_record_unindexed_scan_count(), 0);
    assert_eq!(counters.retained_record_scan_count(), 0);
    assert_eq!(counters.runtime_graph_scan_count(), 0);
}

#[test]
fn indexed_reference_resolution_lookup_cost_follows_anchor_reference_width_not_retention_width() {
    for unrelated_retained_record_count in [0, 8, 64] {
        let anchor = changed_anchor_with_route_signal_and_inspection();
        let mut index_records = vec![causal_evidence_reference_index_record(
            CausalEvidenceOwner::RuntimeBridge,
            CausalEvidenceFamily::BridgeRoute,
            crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                "indexed-bridge-route-reference",
            ),
        )
        .unwrap()];
        for retained_index in 0..unrelated_retained_record_count {
            index_records.push(
                causal_evidence_reference_index_record(
                    CausalEvidenceOwner::RuntimeBridge,
                    CausalEvidenceFamily::BridgeRoute,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        format!("unrelated-retained-bridge-route-{retained_index}"),
                    ),
                )
                .unwrap(),
            );
        }
        let index =
            causal_evidence_reference_index(index_records).expect("fixture digest must fit");

        let resolution = resolve_indexed_causal_evidence_references(
            anchor,
            &[CausalEvidenceFamily::BridgeRoute],
            &index,
        )
        .expect("fixture digest must fit");

        let CausalEvidenceReferenceResolution::Resolved { counters, .. } = resolution else {
            panic!("expected exact indexed bridge route reference to resolve");
        };

        assert_eq!(index.record_count(), 1 + unrelated_retained_record_count);
        assert_eq!(counters.requested_family_count(), 1);
        assert_eq!(counters.anchor_reference_width(), 3);
        assert_eq!(counters.indexed_record_count(), 1);
        assert_eq!(counters.index_lookup_count(), 1);
        assert_eq!(counters.resolved_reference_count(), 1);
        assert_eq!(counters.missing_required_reference_count(), 0);
        assert_eq!(counters.bridge_record_unindexed_scan_count(), 0);
        assert_eq!(counters.retained_record_scan_count(), 0);
        assert_eq!(counters.runtime_graph_scan_count(), 0);
    }
}

#[test]
fn reference_index_record_denies_owner_mismatch_and_seals_fixture_reference_labels() {
    let owner_mismatch = causal_evidence_reference_index_record(
        CausalEvidenceOwner::Signal,
        CausalEvidenceFamily::BridgeRoute,
        crate::runtime::tests::causal_inspection::causal_test_reference_digest(
            "bridge-route-reference",
        ),
    )
    .unwrap_err();
    let owner_mismatch=match owner_mismatch { worth_query_execution::facade::causal_inspection::CausalInspectionConstructionStop::ReferenceIndex(denial)=>denial, other=>panic!("expected a semantic denial, got {other:?}"), };

    assert_eq!(
        owner_mismatch.kind(),
        CausalEvidenceReferenceIndexErrorKind::EvidenceOwnerMismatch
    );
    assert_eq!(owner_mismatch.family(), CausalEvidenceFamily::BridgeRoute);
    assert_eq!(owner_mismatch.supplied_owner(), CausalEvidenceOwner::Signal);
    assert_eq!(
        owner_mismatch.expected_owner(),
        CausalEvidenceOwner::RuntimeBridge
    );
    assert!(!owner_mismatch.failure_digest().is_empty());

    let sealed_empty_label = causal_evidence_reference_index_record(
        CausalEvidenceOwner::RuntimeBridge,
        CausalEvidenceFamily::BridgeRoute,
        crate::runtime::tests::causal_inspection::causal_test_reference_digest(""),
    )
    .expect("typed reference labels should seal before index admission");

    assert_eq!(
        sealed_empty_label.family(),
        CausalEvidenceFamily::BridgeRoute
    );
    assert!(!sealed_empty_label.reference_digest().as_str().is_empty());
}
