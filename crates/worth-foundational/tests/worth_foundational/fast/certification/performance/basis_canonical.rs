use worth_foundational::{
    performance, performance_api, CanonicalBasisDomain, CanonicalBasisEntryKind,
    CanonicalComparisonOutcome, FoundationalPerformanceAccessPatternPosture,
    FoundationalPerformanceAllocationPosture, FoundationalPerformanceCounterRow,
    FoundationalPerformanceWorkClass,
};

use super::basis_support::{
    assert_entries_present, authoritative_claim, derive_digest, exact_compare,
    performance_text_entry,
};

#[test]
fn canonical_basis_and_digest_lowering_preserve_semantic_bundle_identity() {
    let counter_name = performance_api::lower_lane::basis::FoundationalPerformanceCounterName::new(
        "authoritative_mutation.rows",
    )
    .expect("valid counter name");
    let contract_name =
        performance_api::lower_lane::basis::FoundationalPerformanceContractName::new(
            "query.snapshot_explicit_targets",
        )
        .expect("valid contract name");
    let support_code =
        performance_api::lower_lane::basis::FoundationalPerformanceSupportingEvidenceCode::new(
            "support.snapshot.audit",
        )
        .expect("valid support code");

    let left = performance_api::lower_lane::basis::performance_bundle(authoritative_claim())
        .attach_contract_name(contract_name.clone())
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name.clone(),
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .attach_supporting_evidence_row(
            worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                support_code.clone(),
                FoundationalPerformanceWorkClass::SupportReportAssembly,
            ),
        )
        .finish()
        .expect("left bundle should build");

    let right = performance_api::lower_lane::basis::performance_bundle(authoritative_claim())
        .attach_supporting_evidence_row(
            worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                support_code,
                FoundationalPerformanceWorkClass::SupportReportAssembly,
            ),
        )
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name,
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .attach_contract_name(contract_name)
        .finish()
        .expect("right bundle should build");

    let left_ready =
        match performance_api::lower_lane::basis::prepare_performance_bundle_for_canonical_basis(
            performance_api::lower_lane::basis::performance_basis_rule_version(),
            &left,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected left ready basis"),
        };
    let right_ready =
        match performance_api::lower_lane::basis::prepare_performance_bundle_for_canonical_basis(
            performance_api::lower_lane::basis::performance_basis_rule_version(),
            &right,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected right ready basis"),
        };

    assert!(matches!(
        exact_compare(left_ready, right_ready),
        CanonicalComparisonOutcome::Equivalent(_)
    ));
    let left_digest = derive_digest(
        match performance_api::lower_lane::basis::prepare_performance_bundle_for_canonical_basis(
            performance_api::lower_lane::basis::performance_basis_rule_version(),
            &left,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected left digest basis"),
        },
    );
    let right_digest = derive_digest(
        match performance_api::lower_lane::basis::prepare_performance_bundle_for_canonical_basis(
            performance_api::lower_lane::basis::performance_basis_rule_version(),
            &right,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected right digest basis"),
        },
    );
    assert_eq!(left_digest.value(), right_digest.value());
}

#[test]
fn canonical_basis_discloses_claim_layout_counter_and_support_meaning() {
    let counter_name = performance_api::lower_lane::basis::FoundationalPerformanceCounterName::new(
        "authoritative_mutation.rows",
    )
    .expect("valid counter name");
    let support_code =
        performance_api::lower_lane::basis::FoundationalPerformanceSupportingEvidenceCode::new(
            "support.snapshot.audit",
        )
        .expect("valid support code");
    let layout = performance().define_layout_intent(
        worth_foundational::FoundationalPerformanceLayoutIntent::SoA,
        FoundationalPerformanceAccessPatternPosture::PointLookup,
        FoundationalPerformanceAllocationPosture::ActionLocal,
    );

    let bundle = performance_api::lower_lane::basis::performance_bundle(authoritative_claim())
        .attach_layout_intent_claim(layout)
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name,
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .attach_supporting_evidence_row(
            worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                support_code,
                FoundationalPerformanceWorkClass::SupportReportAssembly,
            ),
        )
        .finish()
        .expect("bundle should build");

    let ready =
        match performance_api::lower_lane::basis::prepare_performance_bundle_for_canonical_basis(
            performance_api::lower_lane::basis::performance_basis_rule_version(),
            &bundle,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected ready basis"),
        };
    let entries =
        performance_api::lower_lane::basis::foundational_performance_canonical_basis_entries(
            &ready,
        );

    assert_eq!(ready.payload().domain(), CanonicalBasisDomain::Performance);
    assert_entries_present(
        entries,
        &[
            performance_text_entry(
                CanonicalBasisEntryKind::PerformanceClaim,
                "claim.boundary",
                "authoritative-execution",
            ),
            performance_text_entry(
                CanonicalBasisEntryKind::PerformanceClaim,
                "claim.execution_temperature",
                "hot-path",
            ),
            performance_text_entry(
                CanonicalBasisEntryKind::PerformanceLayout,
                "layout.intent",
                "soa",
            ),
            performance_text_entry(
                CanonicalBasisEntryKind::PerformanceCounter,
                "counter_spec.0.name",
                "authoritative_mutation.rows",
            ),
            performance_text_entry(
                CanonicalBasisEntryKind::PerformanceSupport,
                "support_row.0.code",
                "support.snapshot.audit",
            ),
        ],
    );
}

#[test]
fn counter_backed_receipt_reuses_bundle_basis_and_adds_observed_rows() {
    let counter_name = performance_api::lower_lane::basis::FoundationalPerformanceCounterName::new(
        "authoritative_mutation.rows",
    )
    .expect("valid counter name");
    let bundle = performance_api::lower_lane::basis::performance_bundle(authoritative_claim())
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name.clone(),
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .finish()
        .expect("bundle should build");
    let receipt = performance_api::lower_lane::receipts::counter_backed_performance_receipt(bundle)
        .attach_counter_row(FoundationalPerformanceCounterRow::new(counter_name, 3))
        .finish()
        .expect("receipt should build");

    let ready = match performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        performance_api::lower_lane::basis::performance_basis_rule_version(),
        &receipt,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected receipt ready basis"),
    };
    let entries =
        performance_api::lower_lane::basis::foundational_performance_canonical_basis_entries(
            &ready,
        );

    assert_entries_present(
        entries,
        &[
            performance_text_entry(
                CanonicalBasisEntryKind::PerformanceClaim,
                "shape",
                "counter-backed-performance-receipt",
            ),
            performance_text_entry(
                CanonicalBasisEntryKind::PerformanceCounter,
                "counter_row.0.name",
                "authoritative_mutation.rows",
            ),
        ],
    );
}
