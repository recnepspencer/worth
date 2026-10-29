use worth_foundational::{
    classify_foundational_profile_compatibility, compare_foundational_profile_identities,
    derive_foundational_profile_identity, foundational_profile_canonical_basis_entries,
    prepare_admitted_foundational_profile_for_canonical_basis, AdmissionReadinessProfile,
    CanonicalBasisDomain, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalComparisonOutcome,
    CanonicalizationRuleVersion, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, FoundationalProfileCompatibilityClass, RetentionDeliveryProfile,
    SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use super::support::{admit_same_profile, profile};

fn version() -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new("m3.profile.identity").expect("valid test version")
}

#[test]
fn admitted_profile_basis_uses_profile_domain_and_stable_family_order() {
    let admitted = admit_same_profile(profile(
        DiagnosticRichnessProfile::Forensic,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityRequired,
        AdmissionReadinessProfile::ProductionGateReady,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::ProductionCertified,
    ));

    let ready =
        match prepare_admitted_foundational_profile_for_canonical_basis(version(), &admitted) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected ready profile basis success"),
        };

    assert_eq!(ready.payload().domain(), CanonicalBasisDomain::Profile);
    let entries = foundational_profile_canonical_basis_entries(ready.payload());
    let loci: Vec<_> = entries.iter().map(|entry| entry.locus().clone()).collect();
    assert_eq!(loci.len(), 8);
    assert!(entries
        .iter()
        .all(|entry| entry.kind() == CanonicalBasisEntryKind::Profile));
    assert_eq!(
        loci,
        vec![
            CanonicalBasisLocus::Named("admission_readiness".into()),
            CanonicalBasisLocus::Named("certification_posture".into()),
            CanonicalBasisLocus::Named("compatibility_posture".into()),
            CanonicalBasisLocus::Named("diagnostic_richness".into()),
            CanonicalBasisLocus::Named("execution_objective".into()),
            CanonicalBasisLocus::Named("observation_activation".into()),
            CanonicalBasisLocus::Named("retention_delivery".into()),
            CanonicalBasisLocus::Named("support_posture".into()),
        ]
    );
}

#[test]
fn semantically_identical_admitted_profiles_derive_identical_identity() {
    let left = admit_same_profile(profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::SupportReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Retained,
        CertificationPostureProfile::EvidenceBacked,
    ));
    let right = admit_same_profile(profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::SupportReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Retained,
        CertificationPostureProfile::EvidenceBacked,
    ));

    let left_identity = match derive_foundational_profile_identity(version(), &left) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected left identity, got {other:?}"),
    };
    let right_identity = match derive_foundational_profile_identity(version(), &right) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected right identity, got {other:?}"),
    };

    assert_eq!(left_identity, right_identity);
    assert_eq!(left.payload().admitted(), right.payload().admitted());
}

#[test]
fn semantic_profile_changes_shift_identity_and_classify_structurally() {
    let exact = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
            CompatibilityPostureProfile::CompatibilityLowered,
            AdmissionReadinessProfile::Admitted,
            RetentionDeliveryProfile::Retained,
            CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected exact identity, got {other:?}"),
    };
    let richness = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            DiagnosticRichnessProfile::OperationalMinimal,
            SupportPostureProfile::SupportReady,
            CompatibilityPostureProfile::CompatibilityLowered,
            AdmissionReadinessProfile::Admitted,
            RetentionDeliveryProfile::Retained,
            CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected richness identity, got {other:?}"),
    };
    let retention = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
            CompatibilityPostureProfile::CompatibilityLowered,
            AdmissionReadinessProfile::Admitted,
            RetentionDeliveryProfile::Durable,
            CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected retention identity, got {other:?}"),
    };
    let support = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::CertificationReady,
            CompatibilityPostureProfile::CompatibilityLowered,
            AdmissionReadinessProfile::Admitted,
            RetentionDeliveryProfile::Retained,
            CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected support identity, got {other:?}"),
    };
    let certification_base = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::CertificationReady,
            CompatibilityPostureProfile::CompatibilityLowered,
            AdmissionReadinessProfile::ProductionGateReady,
            RetentionDeliveryProfile::Durable,
            CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected certification-base identity, got {other:?}"),
    };
    let certification = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::CertificationReady,
            CompatibilityPostureProfile::CompatibilityLowered,
            AdmissionReadinessProfile::ProductionGateReady,
            RetentionDeliveryProfile::Durable,
            CertificationPostureProfile::ProductionCertified,
        )),
    ) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected certification identity, got {other:?}"),
    };
    let incompatible = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
            CompatibilityPostureProfile::CompatibilityRequired,
            AdmissionReadinessProfile::Admitted,
            RetentionDeliveryProfile::Retained,
            CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected incompatible identity, got {other:?}"),
    };

    assert_ne!(exact.digest().value(), richness.digest().value());
    assert_ne!(exact.digest().value(), retention.digest().value());
    assert_eq!(
        classify_foundational_profile_compatibility(&exact, &richness),
        FoundationalProfileCompatibilityClass::RichnessOnlyChange
    );
    assert_eq!(
        classify_foundational_profile_compatibility(&exact, &retention),
        FoundationalProfileCompatibilityClass::RetentionOnlyNarrowing
    );
    assert_eq!(
        classify_foundational_profile_compatibility(&exact, &support),
        FoundationalProfileCompatibilityClass::SupportPostureChange
    );
    assert_eq!(
        classify_foundational_profile_compatibility(&certification_base, &certification),
        FoundationalProfileCompatibilityClass::CertificationPostureChange
    );
    assert_eq!(
        classify_foundational_profile_compatibility(&exact, &incompatible),
        FoundationalProfileCompatibilityClass::Incompatible
    );

    let (_, comparison) = compare_foundational_profile_identities(&exact, &richness);
    assert!(matches!(
        comparison,
        CanonicalComparisonOutcome::Mismatched(_)
    ));
}
