use super::*;

#[test]
fn evidence_inventory_names_existing_lower_runtime_reference_identities() {
    let rows = causal_evidence_inventory_rows();

    assert!(rows.iter().any(|row| {
        row.owner() == CausalEvidenceOwner::RuntimeBridge
            && row.family() == CausalEvidenceFamily::BridgeRoute
            && row
                .authority_surface()
                .contains("BridgeDiagnosticsFacade::route_records")
            && row.query_reference_identity() == "route_digest"
    }));
    assert!(rows.iter().any(|row| {
        row.owner() == CausalEvidenceOwner::Signal
            && row.family() == CausalEvidenceFamily::SignalForensicAvailability
    }));
    assert!(rows.iter().any(|row| {
        row.owner() == CausalEvidenceOwner::Relational
            && row.family() == CausalEvidenceFamily::RelationalAuthority
    }));
    for family in [
        CausalEvidenceFamily::QueryInspection,
        CausalEvidenceFamily::QueryMutationCausality,
        CausalEvidenceFamily::QueryMutationProvenance,
        CausalEvidenceFamily::RelationalAuthority,
        CausalEvidenceFamily::RelationalDecision,
        CausalEvidenceFamily::BridgeRoute,
        CausalEvidenceFamily::BridgeEvaluation,
        CausalEvidenceFamily::BridgeSourceMaterialization,
        CausalEvidenceFamily::BridgeSourceFailure,
        CausalEvidenceFamily::BridgeContinuity,
        CausalEvidenceFamily::BridgeMerge,
        CausalEvidenceFamily::BridgeStructural,
        CausalEvidenceFamily::BridgeStream,
        CausalEvidenceFamily::BridgePreview,
        CausalEvidenceFamily::BridgeWriteback,
        CausalEvidenceFamily::BridgeMapper,
        CausalEvidenceFamily::BridgeReplay,
        CausalEvidenceFamily::SignalInvalidation,
        CausalEvidenceFamily::SignalEvaluation,
        CausalEvidenceFamily::SignalForensicAvailability,
        CausalEvidenceFamily::SignalReplayCursor,
        CausalEvidenceFamily::SignalLineage,
        CausalEvidenceFamily::SignalProvenance,
        CausalEvidenceFamily::Lineage,
        CausalEvidenceFamily::Provenance,
        CausalEvidenceFamily::Policy,
        CausalEvidenceFamily::Redaction,
    ] {
        assert!(
            rows.iter().any(|row| row.family() == family),
            "missing causal evidence inventory row for {family:?}"
        );
    }

    let unique_families = rows
        .iter()
        .map(CausalEvidenceInventoryRow::family)
        .collect::<BTreeSet<_>>();
    assert_eq!(unique_families.len(), rows.len());
}

#[test]
fn evidence_reference_resolution_consumes_anchor_and_emits_sealed_reference_set() {
    let anchor = anchor_causal_observation(
        QueryObservationReceipt::fixture(
            CausalObservationOutcome::Changed,
            vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "query-inspection-reference",
                    ),
                )
                .expect("fixture digest must fit"),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::RelationalAuthority,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "relational-authority-reference",
                    ),
                )
                .expect("fixture digest must fit"),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgeRoute,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "bridge-route-reference",
                    ),
                )
                .expect("fixture digest must fit"),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::SignalInvalidation,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "signal-invalidation-reference",
                    ),
                )
                .expect("positive fixture identity fits canonical digest bounds"),
            ],
        )
        .expect("positive fixture identity fits canonical digest bounds"),
        CausalInspectionReason::ChangedResult,
    )
    .unwrap();
    let anchor_digest = anchor.anchor_digest().clone();

    let resolution = resolve_causal_evidence_references(
        anchor,
        &[
            CausalEvidenceFamily::BridgeRoute,
            CausalEvidenceFamily::RelationalAuthority,
            CausalEvidenceFamily::SignalInvalidation,
        ],
    )
    .expect("fixture digest must fit");

    let CausalEvidenceReferenceResolution::Resolved {
        reference_set,
        counters,
    } = resolution
    else {
        panic!("expected causal evidence references to resolve");
    };

    assert_eq!(reference_set.anchor().anchor_digest(), &anchor_digest);
    assert_eq!(reference_set.references().len(), 3);
    assert!(reference_set.references().iter().any(|reference| {
        reference.owner() == CausalEvidenceOwner::RuntimeBridge
            && reference.family() == CausalEvidenceFamily::BridgeRoute
            && reference.reference_digest()
                == &crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "bridge-route-reference",
                )
    }));
    assert!(reference_set.references().iter().any(|reference| {
        reference.owner() == CausalEvidenceOwner::Relational
            && reference.family() == CausalEvidenceFamily::RelationalAuthority
    }));
    assert!(reference_set.references().iter().any(|reference| {
        reference.owner() == CausalEvidenceOwner::Signal
            && reference.family() == CausalEvidenceFamily::SignalInvalidation
    }));
    assert!(!reference_set.reference_set_digest().as_str().is_empty());
    assert!(!reference_set.receipt().receipt_digest().is_empty());
    assert_eq!(reference_set.receipt().resolved_reference_count(), 3);
    assert_eq!(reference_set.receipt().missing_reference_family_count(), 0);
    assert_eq!(counters.requested_family_count(), 3);
    assert_eq!(counters.anchor_reference_width(), 4);
    assert_eq!(counters.index_lookup_count(), 3);
    assert_eq!(counters.resolved_reference_count(), 3);
    assert_eq!(counters.missing_required_reference_count(), 0);
    assert_eq!(counters.bridge_record_unindexed_scan_count(), 0);
    assert_eq!(counters.retained_record_scan_count(), 0);
    assert_eq!(counters.runtime_graph_scan_count(), 0);
}

#[test]
fn evidence_reference_resolution_denies_requested_family_missing_from_anchor() {
    let anchor = anchor_causal_observation(
        QueryObservationReceipt::fixture(
            CausalObservationOutcome::Denied,
            vec![CausalObservationEvidenceIdentity::new(
                CausalEvidenceFamily::QueryInspection,
                crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                    "query-denial-reference",
                ),
            )
            .expect("fixture digest must fit")],
        )
        .expect("positive fixture identity fits canonical digest bounds"),
        CausalInspectionReason::DeniedResult,
    )
    .unwrap();
    let anchor_digest = anchor.anchor_digest().clone();

    let resolution = resolve_causal_evidence_references(
        anchor,
        &[
            CausalEvidenceFamily::QueryInspection,
            CausalEvidenceFamily::BridgeRoute,
        ],
    )
    .expect("fixture digest must fit");

    let CausalEvidenceReferenceResolution::MissingRequiredEvidence { denial, counters } =
        resolution
    else {
        panic!("expected missing bridge route evidence to deny resolution");
    };

    assert_eq!(denial.anchor_digest(), &anchor_digest);
    assert_eq!(
        denial.missing_families(),
        &[CausalEvidenceFamily::BridgeRoute]
    );
    assert!(!denial.failure_digest().is_empty());
    assert_eq!(counters.requested_family_count(), 2);
    assert_eq!(counters.anchor_reference_width(), 1);
    assert_eq!(counters.index_lookup_count(), 2);
    assert_eq!(counters.resolved_reference_count(), 1);
    assert_eq!(counters.missing_required_reference_count(), 1);
    assert_eq!(counters.bridge_record_unindexed_scan_count(), 0);
    assert_eq!(counters.retained_record_scan_count(), 0);
    assert_eq!(counters.runtime_graph_scan_count(), 0);
}

#[test]
fn evidence_reference_resolution_defaults_to_anchor_carried_families() {
    let anchor = anchor_causal_observation(
        QueryObservationReceipt::fixture(
            CausalObservationOutcome::Replayed,
            vec![
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::QueryInspection,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "replay-query-inspection-reference",
                    ),
                )
                .expect("fixture digest must fit"),
                CausalObservationEvidenceIdentity::new(
                    CausalEvidenceFamily::BridgeReplay,
                    crate::runtime::tests::causal_inspection::causal_test_reference_digest(
                        "bridge-replay-reference",
                    ),
                )
                .expect("fixture digest must fit"),
            ],
        )
        .expect("positive fixture identity fits canonical digest bounds"),
        CausalInspectionReason::HistoricalReplayResult,
    )
    .unwrap();

    let resolution =
        resolve_causal_evidence_references(anchor, &[]).expect("fixture digest must fit");

    let CausalEvidenceReferenceResolution::Resolved {
        reference_set,
        counters,
    } = resolution
    else {
        panic!("expected carried anchor families to resolve by default");
    };

    assert_eq!(reference_set.references().len(), 2);
    assert!(reference_set.references().iter().any(|reference| {
        reference.owner() == CausalEvidenceOwner::RuntimeBridge
            && reference.family() == CausalEvidenceFamily::BridgeReplay
    }));
    assert_eq!(counters.requested_family_count(), 2);
    assert_eq!(counters.anchor_reference_width(), 2);
    assert_eq!(counters.index_lookup_count(), 2);
    assert_eq!(counters.resolved_reference_count(), 2);
    assert_eq!(counters.missing_required_reference_count(), 0);
}
