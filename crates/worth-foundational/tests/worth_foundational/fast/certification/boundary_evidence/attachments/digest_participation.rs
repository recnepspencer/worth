use super::*;

#[test]
fn mixed_family_attachment_bundle_preserves_semantics_across_canonical_and_digest_participation() {
    let materialized = materialized_mixed_bundle_from_direct_path(55);

    assert!(materialized.support().is_some());
    assert!(materialized.diagnostic().is_some());

    let basis =
        worth_foundational::prepare_boundary_evidence_attachment_bundle_for_canonical_basis(
            worth_foundational::CanonicalizationRuleVersion::new("m7.phase6.mixed-bundle")
                .expect("version"),
            &materialized,
        )
        .expect_success("mixed attachment basis");
    let digest = worth_foundational::derive_boundary_evidence_attachment_bundle_digest(
        worth_foundational::CanonicalizationRuleVersion::new("m7.phase6.mixed-bundle")
            .expect("version"),
        &materialized,
        CanonicalDigestAlgorithmId::sha256(),
    )
    .expect_success("mixed attachment digest");

    assert_eq!(
        basis.payload().domain(),
        CanonicalBasisDomain::BoundaryArtifact
    );
    assert_eq!(
        digest.metadata().algorithm().id(),
        &CanonicalDigestAlgorithmId::sha256()
    );
}

#[test]
fn mixed_family_attachment_bundle_is_canonical_across_independent_attachment_orderings() {
    let common_path_materialized = materialized_mixed_bundle_from_common_path(65);
    let direct_path_materialized = materialized_mixed_bundle_from_direct_path(65);

    let common_path_basis =
        worth_foundational::prepare_boundary_evidence_attachment_bundle_for_canonical_basis(
            worth_foundational::CanonicalizationRuleVersion::new("m7.phase6.ordering-hostility")
                .expect("version"),
            &common_path_materialized,
        )
        .expect_success("common path basis");
    let direct_path_basis =
        worth_foundational::prepare_boundary_evidence_attachment_bundle_for_canonical_basis(
            worth_foundational::CanonicalizationRuleVersion::new("m7.phase6.ordering-hostility")
                .expect("version"),
            &direct_path_materialized,
        )
        .expect_success("direct path basis");
    let common_path_digest = worth_foundational::derive_boundary_evidence_attachment_bundle_digest(
        worth_foundational::CanonicalizationRuleVersion::new("m7.phase6.ordering-hostility")
            .expect("version"),
        &common_path_materialized,
        CanonicalDigestAlgorithmId::sha256(),
    )
    .expect_success("common path digest");
    let direct_path_digest = worth_foundational::derive_boundary_evidence_attachment_bundle_digest(
        worth_foundational::CanonicalizationRuleVersion::new("m7.phase6.ordering-hostility")
            .expect("version"),
        &direct_path_materialized,
        CanonicalDigestAlgorithmId::sha256(),
    )
    .expect_success("direct path digest");

    assert_eq!(common_path_basis.payload(), direct_path_basis.payload());
    assert_eq!(common_path_digest.metadata(), direct_path_digest.metadata());
    assert_eq!(
        common_path_digest.value().bytes(),
        direct_path_digest.value().bytes()
    );
}
