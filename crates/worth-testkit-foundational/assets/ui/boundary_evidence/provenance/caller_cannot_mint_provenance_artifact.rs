use worth_foundational::{
    FoundationalBoundaryEvidenceAuthorityPath, FoundationalBoundaryEvidenceCanonicalDigestBasis,
    FoundationalBoundaryEvidenceComparisonBasis, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLocality, FoundationalBoundaryEvidenceProfileBasis,
    FoundationalBoundaryEvidenceProvenanceArtifact, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalBoundaryEvidenceStrategyBasis, FoundationalBoundaryEvidenceSupportContextAttachment,
};

fn impossible<T>() -> T {
    panic!("unreachable")
}

fn main() {
    let _artifact = FoundationalBoundaryEvidenceProvenanceArtifact {
        locality: FoundationalBoundaryEvidenceLocality::Current,
        freshness_posture: FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
        source_basis: impossible::<FoundationalBoundaryEvidenceSourceBasis>(),
        authority_path: Some(impossible::<FoundationalBoundaryEvidenceAuthorityPath>()),
        strategy_basis: Some(impossible::<FoundationalBoundaryEvidenceStrategyBasis>()),
        profile_basis: Some(impossible::<FoundationalBoundaryEvidenceProfileBasis>()),
        comparison_basis: Some(impossible::<FoundationalBoundaryEvidenceComparisonBasis>()),
        canonical_digest_basis: Some(impossible::<FoundationalBoundaryEvidenceCanonicalDigestBasis>()),
        support_context_attachments: vec![
            impossible::<FoundationalBoundaryEvidenceSupportContextAttachment>(),
        ],
    };
}
