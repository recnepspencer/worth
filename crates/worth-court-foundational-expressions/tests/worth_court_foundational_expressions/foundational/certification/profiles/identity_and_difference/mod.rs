#[allow(unused_imports)]
use crate::certification::ordinary_context::profiles::identity_and_difference::*;
use {
    worth_foundational::classify_foundational_profile_compatibility,
    worth_foundational::compare_foundational_profile_identities,
    worth_foundational::derive_foundational_profile_identity,
    worth_foundational::foundational_profile_canonical_basis_entries,
    worth_foundational::prepare_admitted_foundational_profile_for_canonical_basis,
};

use {
    crate::certification::ordinary_context::profiles::support::admit_same_profile,
    crate::certification::ordinary_context::profiles::support::profile,
};

#[test]
fn admitted_profile_basis_uses_profile_domain_and_stable_family_order() {
    let admitted = admit_same_profile(profile(
        worth_foundational::DiagnosticRichnessProfile::Forensic,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::ProductionCertified,
    ));

    let ready =
        match prepare_admitted_foundational_profile_for_canonical_basis(version(), &admitted) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected ready profile basis success"),
        };

    assert_eq!(
        ready.payload().domain(),
        worth_foundational::CanonicalBasisDomain::Profile
    );
    let entries = foundational_profile_canonical_basis_entries(ready.payload());
    let loci: Vec<_> = entries.iter().map(|entry| entry.locus().clone()).collect();
    assert_eq!(loci.len(), 8);
    assert!(entries
        .iter()
        .all(|entry| entry.kind() == worth_foundational::CanonicalBasisEntryKind::Profile));
    assert_eq!(
        loci,
        vec![
            worth_foundational::CanonicalBasisLocus::Named("admission_readiness".into()),
            worth_foundational::CanonicalBasisLocus::Named("certification_posture".into()),
            worth_foundational::CanonicalBasisLocus::Named("compatibility_posture".into()),
            worth_foundational::CanonicalBasisLocus::Named("diagnostic_richness".into()),
            worth_foundational::CanonicalBasisLocus::Named("execution_objective".into()),
            worth_foundational::CanonicalBasisLocus::Named("observation_activation".into()),
            worth_foundational::CanonicalBasisLocus::Named("retention_delivery".into()),
            worth_foundational::CanonicalBasisLocus::Named("support_posture".into()),
        ]
    );
}

#[test]
fn semantically_identical_admitted_profiles_derive_identical_identity() {
    let left = admit_same_profile(profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    ));
    let right = admit_same_profile(profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    ));

    let left_identity = match derive_foundational_profile_identity(version(), &left) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected left identity, got {other:?}"),
    };
    let right_identity = match derive_foundational_profile_identity(version(), &right) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
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
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
            worth_foundational::AdmissionReadinessProfile::Admitted,
            worth_foundational::RetentionDeliveryProfile::Retained,
            worth_foundational::CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected exact identity, got {other:?}"),
    };
    let richness = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            worth_foundational::DiagnosticRichnessProfile::OperationalMinimal,
            worth_foundational::SupportPostureProfile::SupportReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
            worth_foundational::AdmissionReadinessProfile::Admitted,
            worth_foundational::RetentionDeliveryProfile::Retained,
            worth_foundational::CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected richness identity, got {other:?}"),
    };
    let retention = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
            worth_foundational::AdmissionReadinessProfile::Admitted,
            worth_foundational::RetentionDeliveryProfile::Durable,
            worth_foundational::CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected retention identity, got {other:?}"),
    };
    let support = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::CertificationReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
            worth_foundational::AdmissionReadinessProfile::Admitted,
            worth_foundational::RetentionDeliveryProfile::Retained,
            worth_foundational::CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected support identity, got {other:?}"),
    };
    let certification_base = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::CertificationReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
            worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
            worth_foundational::RetentionDeliveryProfile::Durable,
            worth_foundational::CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected certification-base identity, got {other:?}"),
    };
    let certification = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::CertificationReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
            worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
            worth_foundational::RetentionDeliveryProfile::Durable,
            worth_foundational::CertificationPostureProfile::ProductionCertified,
        )),
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected certification identity, got {other:?}"),
    };
    let incompatible = match derive_foundational_profile_identity(
        version(),
        &admit_same_profile(profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
            worth_foundational::AdmissionReadinessProfile::Admitted,
            worth_foundational::RetentionDeliveryProfile::Retained,
            worth_foundational::CertificationPostureProfile::EvidenceBacked,
        )),
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected incompatible identity, got {other:?}"),
    };

    assert_ne!(exact.digest().value(), richness.digest().value());
    assert_ne!(exact.digest().value(), retention.digest().value());
    assert_eq!(
        classify_foundational_profile_compatibility(&exact, &richness),
        worth_foundational::FoundationalProfileCompatibilityClass::RichnessOnlyChange
    );
    assert_eq!(
        classify_foundational_profile_compatibility(&exact, &retention),
        worth_foundational::FoundationalProfileCompatibilityClass::RetentionOnlyNarrowing
    );
    assert_eq!(
        classify_foundational_profile_compatibility(&exact, &support),
        worth_foundational::FoundationalProfileCompatibilityClass::SupportPostureChange
    );
    assert_eq!(
        classify_foundational_profile_compatibility(&certification_base, &certification),
        worth_foundational::FoundationalProfileCompatibilityClass::CertificationPostureChange
    );
    assert_eq!(
        classify_foundational_profile_compatibility(&exact, &incompatible),
        worth_foundational::FoundationalProfileCompatibilityClass::Incompatible
    );

    let (_, comparison) = compare_foundational_profile_identities(&exact, &richness);
    assert!(matches!(
        comparison,
        worth_foundational::CanonicalComparisonOutcome::Mismatched(_)
    ));
}
