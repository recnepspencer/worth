#[allow(unused_imports)] use crate::certification::ordinary_context::profiles::certification_posture::*;
use {worth_foundational::admit_requested_foundational_profile, worth_foundational::attach_proof_bearing_profiled_artifact, worth_foundational::bridge_evidence_backed_proof_bearing_artifact_trust_boundary, worth_foundational::bridge_production_certified_proof_bearing_artifact_trust_boundary, worth_foundational::certify_evidence_backed_proof_bearing_artifact, worth_foundational::certify_production_certified_proof_bearing_artifact, worth_foundational::foundational_profile_certification_authority, worth_foundational::foundational_profile_certification_proof_lane, worth_foundational::foundational_profile_certification_readmission_authority, worth_foundational::foundational_profile_progression_authority, worth_foundational::readmit_evidence_backed_proof_bearing_artifact_after_boundary, worth_foundational::readmit_production_certified_proof_bearing_artifact_after_boundary, worth_foundational::request_foundational_profile_set, worth_foundational::AdmissionReadinessProfile, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalProfileCertificationDenial, worth_foundational::FoundationalProfileCertificationProofLane, worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
use {worth_proof::TransitionOutcome};

#[test]
fn certification_strengthening_requires_explicit_proof_bearing_progression() {
    assert_eq!(
        foundational_profile_certification_proof_lane(),
        worth_foundational::FoundationalProfileCertificationProofLane::CurrentBasisArtifactWithBoundaryReadmission
    );

    let uncertified = proof_bearing_payload(profile(
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::Uncertified,
    ));
    assert!(matches!(
        certify_evidence_backed_proof_bearing_artifact(
            uncertified,
            foundational_profile_certification_authority(),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalProfileCertificationDenial::EvidenceBackedStrengtheningRequiresEvidenceBackedProfile
        )
    ));

    let evidence_backed = match certify_evidence_backed_proof_bearing_artifact(
        proof_bearing_payload(profile(
            worth_foundational::SupportPostureProfile::SupportReady,
            worth_foundational::AdmissionReadinessProfile::Admitted,
            worth_foundational::RetentionDeliveryProfile::Retained,
            worth_foundational::CertificationPostureProfile::EvidenceBacked,
        )),
        foundational_profile_certification_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected evidence-backed strengthening"),
    };
    assert_eq!(evidence_backed.payload(), &"payload");
    assert_eq!(
        evidence_backed
            .profiled()
            .profile()
            .materialized()
            .certification_posture(),
        worth_foundational::CertificationPostureProfile::EvidenceBacked
    );

    assert!(matches!(
        certify_production_certified_proof_bearing_artifact(
            evidence_backed,
            foundational_profile_certification_authority(),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalProfileCertificationDenial::ProductionCertifiedStrengtheningRequiresProductionCertifiedProfile
        )
    ));
}

#[test]
fn production_certified_strengthening_and_boundary_readmission_remain_explicit() {
    let evidence_backed = match certify_evidence_backed_proof_bearing_artifact(
        proof_bearing_payload(profile(
            worth_foundational::SupportPostureProfile::CertificationReady,
            worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
            worth_foundational::RetentionDeliveryProfile::Retained,
            worth_foundational::CertificationPostureProfile::ProductionCertified,
        )),
        foundational_profile_certification_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected evidence-backed strengthening"),
    };

    let production_certified = match certify_production_certified_proof_bearing_artifact(
        evidence_backed,
        foundational_profile_certification_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected production-certified strengthening"),
    };
    assert_eq!(production_certified.payload(), &"payload");
    assert_eq!(
        production_certified
            .profiled()
            .profile()
            .materialized()
            .certification_posture(),
        worth_foundational::CertificationPostureProfile::ProductionCertified
    );

    let readmitted_evidence = readmit_evidence_backed_proof_bearing_artifact_after_boundary(
        bridge_evidence_backed_proof_bearing_artifact_trust_boundary(
            match certify_evidence_backed_proof_bearing_artifact(
                proof_bearing_payload(profile(
                    worth_foundational::SupportPostureProfile::CertificationReady,
                    worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
                    worth_foundational::RetentionDeliveryProfile::Retained,
                    worth_foundational::CertificationPostureProfile::ProductionCertified,
                )),
                foundational_profile_certification_authority(),
            ) {
                worth_proof::TransitionOutcome::Success(certified) => certified,
                _ => panic!("expected evidence-backed strengthening"),
            },
        ),
        foundational_profile_certification_readmission_authority(),
    );
    assert_eq!(readmitted_evidence.payload(), &"payload");

    let readmitted_production = readmit_production_certified_proof_bearing_artifact_after_boundary(
        bridge_production_certified_proof_bearing_artifact_trust_boundary(production_certified),
        foundational_profile_certification_readmission_authority(),
    );
    assert_eq!(readmitted_production.payload(), &"payload");
}
