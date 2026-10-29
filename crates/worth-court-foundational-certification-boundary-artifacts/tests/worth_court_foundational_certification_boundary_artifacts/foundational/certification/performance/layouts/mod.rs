use {worth_foundational::performance};
use {worth_foundational::FoundationalLayoutAnnotatedClaimConstructionDenial, worth_foundational::FoundationalPerformanceAccessPatternPosture, worth_foundational::FoundationalPerformanceAllocationPosture, worth_foundational::FoundationalPerformanceLayoutIntent, worth_foundational::FoundationalPerformanceWorkClass};

#[test]
fn layout_intent_claim_preserves_declarative_representation_and_allocation_meaning() {
    let layout_claim = performance().define_layout_intent(
        worth_foundational::FoundationalPerformanceLayoutIntent::Packed,
        worth_foundational::FoundationalPerformanceAccessPatternPosture::ScanHeavy,
        worth_foundational::FoundationalPerformanceAllocationPosture::BatchLocal,
    );

    assert_eq!(
        layout_claim.layout_intent(),
        worth_foundational::FoundationalPerformanceLayoutIntent::Packed
    );
    assert_eq!(
        layout_claim.access_pattern(),
        worth_foundational::FoundationalPerformanceAccessPatternPosture::ScanHeavy
    );
    assert_eq!(
        layout_claim.allocation_posture(),
        worth_foundational::FoundationalPerformanceAllocationPosture::BatchLocal
    );
    assert_eq!(layout_claim.layout_definition().name(), "packed");
    assert_eq!(layout_claim.access_definition().name(), "scan_heavy");
    assert_eq!(layout_claim.allocation_definition().name(), "batch_local");
}

#[test]
fn distinct_layouts_can_attach_to_same_claim_meaning_without_changing_the_claim() {
    let claim = performance()
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
        )
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("authoritative claim should build");

    let aos_layout = performance().define_layout_intent(
        worth_foundational::FoundationalPerformanceLayoutIntent::AoS,
        worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        worth_foundational::FoundationalPerformanceAllocationPosture::ActionLocal,
    );
    let soa_layout = performance().define_layout_intent(
        worth_foundational::FoundationalPerformanceLayoutIntent::SoA,
        worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        worth_foundational::FoundationalPerformanceAllocationPosture::ArenaLocal,
    );

    let aos_attached = performance()
        .attach_layout_intent(claim.clone(), aos_layout)
        .expect("matching access posture should attach");
    let soa_attached = performance()
        .attach_layout_intent(claim.clone(), soa_layout)
        .expect("matching access posture should attach");

    assert_eq!(aos_attached.claim(), &claim);
    assert_eq!(soa_attached.claim(), &claim);
    assert_eq!(aos_attached.claim(), soa_attached.claim());
    assert_ne!(aos_attached.layout_intent(), soa_attached.layout_intent());
    assert_ne!(
        aos_attached.allocation_posture(),
        soa_attached.allocation_posture()
    );
}

#[test]
fn layout_attachment_rejects_access_pattern_mismatch() {
    let claim = performance()
        .claim()
        .support_derived()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::SupportAssembly)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::SupportDerivedPerformanceClaim,
        )
        .breadth_locality(
            worth_foundational::FoundationalPerformanceBreadthLocalityPosture::FamilyLocalBatch,
        )
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::TraversalLocal)
        .execution_temperature(
            worth_foundational::FoundationalPerformanceExecutionTemperature::SupportOnly,
        )
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::StaleSupport,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Debt)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .finish()
        .expect("support claim should build");

    let mismatched_layout = performance().define_layout_intent(
        worth_foundational::FoundationalPerformanceLayoutIntent::Sparse,
        worth_foundational::FoundationalPerformanceAccessPatternPosture::ScanHeavy,
        worth_foundational::FoundationalPerformanceAllocationPosture::ManifestScoped,
    );

    let denial = performance()
        .attach_layout_intent(claim, mismatched_layout)
        .expect_err("mismatched access posture must not attach");

    assert_eq!(
        denial,
        worth_foundational::FoundationalLayoutAnnotatedClaimConstructionDenial::AccessPatternMismatch {
            claim_access_pattern: worth_foundational::FoundationalPerformanceAccessPatternPosture::TraversalLocal,
            layout_access_pattern: worth_foundational::FoundationalPerformanceAccessPatternPosture::ScanHeavy,
        }
    );
}
