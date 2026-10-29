use worth_foundational::{
    canonicalization, performance, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalComparisonOutcome, CanonicalDerivedDigest,
    CanonicalDigestAlgorithmId, CanonicalEquivalenceBasis,
    FoundationalPerformanceAccessPatternPosture, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceEvidenceStrength,
    FoundationalPerformanceExecutionTemperature, FoundationalPerformanceFallbackDebtPosture,
    FoundationalPerformanceFreshnessRetentionPosture, FoundationalPerformanceWorkClass,
};

pub fn authoritative_claim() -> worth_foundational::FoundationalAuthoritativePerformanceClaim {
    performance()
        .claim()
        .authoritative_execution()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("authoritative claim should build")
}

pub fn exact_compare(
    left: worth_foundational::CanonicalBasisReadyArtifact,
    right: worth_foundational::CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match worth_foundational::prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };
    worth_foundational::compare_canonical_basis(&ready)
}

pub fn derive_digest(
    ready: worth_foundational::CanonicalBasisReadyArtifact,
) -> CanonicalDerivedDigest {
    let digest_ready = match canonicalization()
        .digest()
        .for_sequence(ready, CanonicalDigestAlgorithmId::sha256())
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected digest readiness"),
    };
    canonicalization().digest().derive(digest_ready)
}

pub fn performance_text_entry(
    kind: CanonicalBasisEntryKind,
    locus: &str,
    value: &str,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        CanonicalBasisDomain::Performance,
        worth_foundational::CanonicalBasisLocus::Named(locus.into()),
        kind,
        worth_foundational::CanonicalBasisValue::ExactText(value.into()),
    )
}

pub fn assert_entries_present(entries: &[CanonicalBasisEntry], expected: &[CanonicalBasisEntry]) {
    for entry in expected {
        assert!(
            entries.contains(entry),
            "expected canonical entry missing: {entry:?}"
        );
    }
}
