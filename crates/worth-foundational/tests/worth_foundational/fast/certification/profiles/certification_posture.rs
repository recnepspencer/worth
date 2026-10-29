use worth_foundational::{
    admit_requested_foundational_profile, attach_proof_bearing_profiled_artifact,
    bridge_evidence_backed_proof_bearing_artifact_trust_boundary,
    bridge_production_certified_proof_bearing_artifact_trust_boundary,
    certify_evidence_backed_proof_bearing_artifact,
    certify_production_certified_proof_bearing_artifact,
    foundational_profile_certification_authority, foundational_profile_certification_proof_lane,
    foundational_profile_certification_readmission_authority,
    foundational_profile_progression_authority,
    readmit_evidence_backed_proof_bearing_artifact_after_boundary,
    readmit_production_certified_proof_bearing_artifact_after_boundary,
    request_foundational_profile_set, AdmissionReadinessProfile, CertificationPostureProfile,
    CompatibilityPostureProfile, DiagnosticRichnessProfile, FoundationalProfileCertificationDenial,
    FoundationalProfileCertificationProofLane, FoundationalProfileSet, FoundationalProfileSetInput,
    RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

fn profile(
    support_posture: SupportPostureProfile,
    admission_readiness: AdmissionReadinessProfile,
    retention_delivery: RetentionDeliveryProfile,
    certification_posture: CertificationPostureProfile,
) -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Forensic,
        support_posture,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityRequired,
        admission_readiness,
        retention_delivery,
        certification_posture,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("coherent profile")
}

fn proof_bearing_payload(
    profile: FoundationalProfileSet,
) -> worth_foundational::ProofBearingProfiledArtifact<&'static str> {
    let requested = request_foundational_profile_set(profile);
    let admitted = match admit_requested_foundational_profile(
        requested,
        profile,
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted profile, got {outcome:?}"),
    };

    match attach_proof_bearing_profiled_artifact(
        admitted,
        profile,
        None,
        "payload",
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(profiled) => profiled,
        outcome => panic!("expected proof-bearing attachment, got {outcome:?}"),
    }
}

#[test]
fn certification_strengthening_requires_explicit_proof_bearing_progression() {
    assert_eq!(
        foundational_profile_certification_proof_lane(),
        FoundationalProfileCertificationProofLane::CurrentBasisArtifactWithBoundaryReadmission
    );

    let uncertified = proof_bearing_payload(profile(
        SupportPostureProfile::SupportReady,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Retained,
        CertificationPostureProfile::Uncertified,
    ));
    assert!(matches!(
        certify_evidence_backed_proof_bearing_artifact(
            uncertified,
            foundational_profile_certification_authority(),
        ),
        TransitionOutcome::Denied(
            FoundationalProfileCertificationDenial::EvidenceBackedStrengtheningRequiresEvidenceBackedProfile
        )
    ));

    let evidence_backed = match certify_evidence_backed_proof_bearing_artifact(
        proof_bearing_payload(profile(
            SupportPostureProfile::SupportReady,
            AdmissionReadinessProfile::Admitted,
            RetentionDeliveryProfile::Retained,
            CertificationPostureProfile::EvidenceBacked,
        )),
        foundational_profile_certification_authority(),
    ) {
        TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected evidence-backed strengthening"),
    };
    assert_eq!(evidence_backed.payload(), &"payload");
    assert_eq!(
        evidence_backed
            .profiled()
            .profile()
            .materialized()
            .certification_posture(),
        CertificationPostureProfile::EvidenceBacked
    );

    assert!(matches!(
        certify_production_certified_proof_bearing_artifact(
            evidence_backed,
            foundational_profile_certification_authority(),
        ),
        TransitionOutcome::Denied(
            FoundationalProfileCertificationDenial::ProductionCertifiedStrengtheningRequiresProductionCertifiedProfile
        )
    ));
}

#[test]
fn production_certified_strengthening_and_boundary_readmission_remain_explicit() {
    let evidence_backed = match certify_evidence_backed_proof_bearing_artifact(
        proof_bearing_payload(profile(
            SupportPostureProfile::CertificationReady,
            AdmissionReadinessProfile::ProductionGateReady,
            RetentionDeliveryProfile::Retained,
            CertificationPostureProfile::ProductionCertified,
        )),
        foundational_profile_certification_authority(),
    ) {
        TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected evidence-backed strengthening"),
    };

    let production_certified = match certify_production_certified_proof_bearing_artifact(
        evidence_backed,
        foundational_profile_certification_authority(),
    ) {
        TransitionOutcome::Success(certified) => certified,
        _ => panic!("expected production-certified strengthening"),
    };
    assert_eq!(production_certified.payload(), &"payload");
    assert_eq!(
        production_certified
            .profiled()
            .profile()
            .materialized()
            .certification_posture(),
        CertificationPostureProfile::ProductionCertified
    );

    let readmitted_evidence = readmit_evidence_backed_proof_bearing_artifact_after_boundary(
        bridge_evidence_backed_proof_bearing_artifact_trust_boundary(
            match certify_evidence_backed_proof_bearing_artifact(
                proof_bearing_payload(profile(
                    SupportPostureProfile::CertificationReady,
                    AdmissionReadinessProfile::ProductionGateReady,
                    RetentionDeliveryProfile::Retained,
                    CertificationPostureProfile::ProductionCertified,
                )),
                foundational_profile_certification_authority(),
            ) {
                TransitionOutcome::Success(certified) => certified,
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
