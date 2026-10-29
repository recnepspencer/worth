pub(crate) use {worth_foundational::canonicalization, worth_foundational::performance, worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind, worth_foundational::CanonicalComparisonOutcome, worth_foundational::CanonicalDerivedDigest, worth_foundational::CanonicalDigestAlgorithmId, worth_foundational::CanonicalEquivalenceBasis, worth_foundational::FoundationalPerformanceAccessPatternPosture, worth_foundational::FoundationalPerformanceBoundary, worth_foundational::FoundationalPerformanceBreadthLocalityPosture, worth_foundational::FoundationalPerformanceEvidenceStrength, worth_foundational::FoundationalPerformanceExecutionTemperature, worth_foundational::FoundationalPerformanceFallbackDebtPosture, worth_foundational::FoundationalPerformanceFreshnessRetentionPosture, worth_foundational::FoundationalPerformanceWorkClass};

pub(crate) fn authoritative_claim() -> worth_foundational::FoundationalAuthoritativePerformanceClaim {
    performance()
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("authoritative claim should build")
}

pub(crate) fn exact_compare(
    left: worth_foundational::CanonicalBasisReadyArtifact,
    right: worth_foundational::CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match worth_foundational::prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };
    worth_foundational::compare_canonical_basis(&ready)
}

pub(crate) fn derive_digest(
    ready: worth_foundational::CanonicalBasisReadyArtifact,
) -> CanonicalDerivedDigest {
    let digest_ready = match canonicalization()
        .digest()
        .for_sequence(ready, worth_foundational::CanonicalDigestAlgorithmId::sha256())
    {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected digest readiness"),
    };
    canonicalization().digest().derive(digest_ready)
}

pub(crate) fn performance_text_entry(
    kind: CanonicalBasisEntryKind,
    locus: &str,
    value: &str,
) -> CanonicalBasisEntry {
    worth_foundational::CanonicalBasisEntry::new(
        worth_foundational::CanonicalBasisDomain::Performance,
        worth_foundational::CanonicalBasisLocus::Named(locus.into()),
        kind,
        worth_foundational::CanonicalBasisValue::ExactText(value.into()),
    )
}

pub(crate) fn assert_entries_present(entries: &[CanonicalBasisEntry], expected: &[CanonicalBasisEntry]) {
    for entry in expected {
        assert!(
            entries.contains(entry),
            "expected canonical entry missing: {entry:?}"
        );
    }
}