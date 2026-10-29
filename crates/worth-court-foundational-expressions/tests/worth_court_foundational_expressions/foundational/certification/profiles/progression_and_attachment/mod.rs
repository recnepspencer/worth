use {worth_foundational::admit_requested_foundational_profile, worth_foundational::attach_boundary_profiled_artifact, worth_foundational::attach_proof_bearing_profiled_artifact, worth_foundational::attach_support_profiled_artifact, worth_foundational::foundational_profile_progression_authority, worth_foundational::request_foundational_profile_set, worth_foundational::AdmissionReadinessProfile, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalProfileAttachmentDenial, worth_foundational::FoundationalProfileNarrowingKind, worth_foundational::FoundationalProfileNarrowingRecord, worth_foundational::FoundationalProfileProgressionDenial, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::profiles::support::profile};

#[test]
fn requested_profiles_require_explicit_narrowing_to_reach_admitted_and_materialized_forms() {
    let requested = request_foundational_profile_set(profile(
        worth_foundational::DiagnosticRichnessProfile::Forensic,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    ));
    let admitted_candidate = profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    );

    let missing_record = admit_requested_foundational_profile(
        requested.clone(),
        admitted_candidate,
        None,
        foundational_profile_progression_authority(),
    );
    assert_eq!(
        missing_record,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalProfileProgressionDenial::MissingExplicitNarrowingRecord
        )
    );

    let admitted = match admit_requested_foundational_profile(
        requested,
        admitted_candidate,
        Some(worth_foundational::FoundationalProfileNarrowingRecord::new(
            worth_foundational::FoundationalProfileNarrowingKind::RichnessReduced,
            "support artifacts should not require forensic richness",
        )),
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted success, got {other:?}"),
    };

    assert_eq!(
        admitted.payload().requested().diagnostic_richness(),
        worth_foundational::DiagnosticRichnessProfile::Forensic
    );
    assert_eq!(
        admitted.payload().admitted().diagnostic_richness(),
        worth_foundational::DiagnosticRichnessProfile::Standard
    );
    assert_eq!(
        admitted.payload().requested_to_admitted_narrowing(),
        Some(worth_foundational::FoundationalProfileNarrowingRecord::new(
            worth_foundational::FoundationalProfileNarrowingKind::RichnessReduced,
            "support artifacts should not require forensic richness",
        ))
    );
}

#[test]
fn reduced_richness_support_and_proof_targets_preserve_payload_truth() {
    let payload = String::from("authoritative payload");
    let requested = request_foundational_profile_set(profile(
        worth_foundational::DiagnosticRichnessProfile::Forensic,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    ));
    let admitted = match admit_requested_foundational_profile(
        requested,
        profile(
            worth_foundational::DiagnosticRichnessProfile::Forensic,
            worth_foundational::SupportPostureProfile::SupportReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
            worth_foundational::AdmissionReadinessProfile::Admitted,
            worth_foundational::RetentionDeliveryProfile::Retained,
            worth_foundational::CertificationPostureProfile::EvidenceBacked,
        ),
        None,
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted success, got {other:?}"),
    };

    let support = match attach_support_profiled_artifact(
        admitted.clone(),
        profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::SupportReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
            worth_foundational::AdmissionReadinessProfile::Admitted,
            worth_foundational::RetentionDeliveryProfile::Retained,
            worth_foundational::CertificationPostureProfile::EvidenceBacked,
        ),
        Some(worth_foundational::FoundationalProfileNarrowingRecord::new(
            worth_foundational::FoundationalProfileNarrowingKind::RichnessReduced,
            "support payloads may omit forensic richness",
        )),
        payload.clone(),
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected support attachment success, got {other:?}"),
    };
    assert_eq!(support.payload().payload(), &payload);
    assert_eq!(
        support
            .payload()
            .profile()
            .materialized()
            .diagnostic_richness(),
        worth_foundational::DiagnosticRichnessProfile::Standard
    );

    let proof_bearing = match attach_proof_bearing_profiled_artifact(
        admitted,
        profile(
            worth_foundational::DiagnosticRichnessProfile::OperationalMinimal,
            worth_foundational::SupportPostureProfile::SupportReady,
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
            worth_foundational::AdmissionReadinessProfile::Admitted,
            worth_foundational::RetentionDeliveryProfile::Retained,
            worth_foundational::CertificationPostureProfile::EvidenceBacked,
        ),
        Some(worth_foundational::FoundationalProfileNarrowingRecord::new(
            worth_foundational::FoundationalProfileNarrowingKind::RichnessReduced,
            "proof-bearing consumers may still receive the same truth with less diagnostics",
        )),
        payload.clone(),
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected proof-bearing attachment success, got {other:?}"),
    };
    assert_eq!(proof_bearing.payload().payload(), &payload);
    assert_eq!(
        proof_bearing
            .payload()
            .profile()
            .materialized()
            .diagnostic_richness(),
        worth_foundational::DiagnosticRichnessProfile::OperationalMinimal
    );
}

#[test]
fn target_specific_attachment_denials_and_blind_consumer_reads_stay_explicit() {
    let internal_only = match admit_requested_foundational_profile(
        request_foundational_profile_set(profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::InternalOnly,
            worth_foundational::CompatibilityPostureProfile::NativeOnly,
            worth_foundational::AdmissionReadinessProfile::CandidateOnly,
            worth_foundational::RetentionDeliveryProfile::Ephemeral,
            worth_foundational::CertificationPostureProfile::Uncertified,
        )),
        profile(
            worth_foundational::DiagnosticRichnessProfile::Standard,
            worth_foundational::SupportPostureProfile::InternalOnly,
            worth_foundational::CompatibilityPostureProfile::NativeOnly,
            worth_foundational::AdmissionReadinessProfile::CandidateOnly,
            worth_foundational::RetentionDeliveryProfile::Ephemeral,
            worth_foundational::CertificationPostureProfile::Uncertified,
        ),
        None,
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted success, got {other:?}"),
    };

    assert_eq!(
        attach_support_profiled_artifact(
            internal_only.clone(),
            *internal_only.payload().admitted(),
            None,
            "payload",
            foundational_profile_progression_authority(),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalProfileAttachmentDenial::SupportArtifactsCannotCarryInternalOnlySupportPosture
        )
    );
    assert_eq!(
        attach_proof_bearing_profiled_artifact(
            internal_only.clone(),
            *internal_only.payload().admitted(),
            None,
            "payload",
            foundational_profile_progression_authority(),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalProfileAttachmentDenial::ProofBearingArtifactsRequireAdmittedReadiness
        )
    );

    let boundary = match attach_boundary_profiled_artifact(
        internal_only,
        profile(
            worth_foundational::DiagnosticRichnessProfile::OperationalMinimal,
            worth_foundational::SupportPostureProfile::InternalOnly,
            worth_foundational::CompatibilityPostureProfile::NativeOnly,
            worth_foundational::AdmissionReadinessProfile::CandidateOnly,
            worth_foundational::RetentionDeliveryProfile::Ephemeral,
            worth_foundational::CertificationPostureProfile::Uncertified,
        ),
        Some(worth_foundational::FoundationalProfileNarrowingRecord::new(
            worth_foundational::FoundationalProfileNarrowingKind::RichnessReduced,
            "boundary consumers may receive the same authority with less richness",
        )),
        11_u8,
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected boundary attachment success, got {other:?}"),
    };

    assert_eq!(boundary.payload().payload(), &11_u8);
    assert_eq!(
        boundary.payload().target_kind(),
        worth_foundational::FoundationalProfileAttachmentTargetKind::BoundaryArtifact
    );
    assert_eq!(
        boundary
            .payload()
            .profile()
            .requested_to_admitted_narrowing(),
        None
    );
    assert_eq!(
        boundary
            .payload()
            .profile()
            .admitted_to_materialized_narrowing(),
        Some(worth_foundational::FoundationalProfileNarrowingRecord::new(
            worth_foundational::FoundationalProfileNarrowingKind::RichnessReduced,
            "boundary consumers may receive the same authority with less richness",
        ))
    );
}
