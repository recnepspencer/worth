use worth_foundational::{
    admit_requested_foundational_profile, attach_boundary_profiled_artifact,
    attach_proof_bearing_profiled_artifact, attach_support_profiled_artifact,
    foundational_profile_progression_authority, request_foundational_profile_set,
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, FoundationalProfileAttachmentDenial,
    FoundationalProfileNarrowingKind, FoundationalProfileNarrowingRecord,
    FoundationalProfileProgressionDenial, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use super::support::profile;

#[test]
fn requested_profiles_require_explicit_narrowing_to_reach_admitted_and_materialized_forms() {
    let requested = request_foundational_profile_set(profile(
        DiagnosticRichnessProfile::Forensic,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityRequired,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Retained,
        CertificationPostureProfile::EvidenceBacked,
    ));
    let admitted_candidate = profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityRequired,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Retained,
        CertificationPostureProfile::EvidenceBacked,
    );

    let missing_record = admit_requested_foundational_profile(
        requested.clone(),
        admitted_candidate,
        None,
        foundational_profile_progression_authority(),
    );
    assert_eq!(
        missing_record,
        TransitionOutcome::Denied(
            FoundationalProfileProgressionDenial::MissingExplicitNarrowingRecord
        )
    );

    let admitted = match admit_requested_foundational_profile(
        requested,
        admitted_candidate,
        Some(FoundationalProfileNarrowingRecord::new(
            FoundationalProfileNarrowingKind::RichnessReduced,
            "support artifacts should not require forensic richness",
        )),
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted success, got {other:?}"),
    };

    assert_eq!(
        admitted.payload().requested().diagnostic_richness(),
        DiagnosticRichnessProfile::Forensic
    );
    assert_eq!(
        admitted.payload().admitted().diagnostic_richness(),
        DiagnosticRichnessProfile::Standard
    );
    assert_eq!(
        admitted.payload().requested_to_admitted_narrowing(),
        Some(FoundationalProfileNarrowingRecord::new(
            FoundationalProfileNarrowingKind::RichnessReduced,
            "support artifacts should not require forensic richness",
        ))
    );
}

#[test]
fn reduced_richness_support_and_proof_targets_preserve_payload_truth() {
    let payload = String::from("authoritative payload");
    let requested = request_foundational_profile_set(profile(
        DiagnosticRichnessProfile::Forensic,
        SupportPostureProfile::SupportReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Retained,
        CertificationPostureProfile::EvidenceBacked,
    ));
    let admitted = match admit_requested_foundational_profile(
        requested,
        profile(
            DiagnosticRichnessProfile::Forensic,
            SupportPostureProfile::SupportReady,
            CompatibilityPostureProfile::CompatibilityLowered,
            AdmissionReadinessProfile::Admitted,
            RetentionDeliveryProfile::Retained,
            CertificationPostureProfile::EvidenceBacked,
        ),
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted success, got {other:?}"),
    };

    let support = match attach_support_profiled_artifact(
        admitted.clone(),
        profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::SupportReady,
            CompatibilityPostureProfile::CompatibilityLowered,
            AdmissionReadinessProfile::Admitted,
            RetentionDeliveryProfile::Retained,
            CertificationPostureProfile::EvidenceBacked,
        ),
        Some(FoundationalProfileNarrowingRecord::new(
            FoundationalProfileNarrowingKind::RichnessReduced,
            "support payloads may omit forensic richness",
        )),
        payload.clone(),
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected support attachment success, got {other:?}"),
    };
    assert_eq!(support.payload().payload(), &payload);
    assert_eq!(
        support
            .payload()
            .profile()
            .materialized()
            .diagnostic_richness(),
        DiagnosticRichnessProfile::Standard
    );

    let proof_bearing = match attach_proof_bearing_profiled_artifact(
        admitted,
        profile(
            DiagnosticRichnessProfile::OperationalMinimal,
            SupportPostureProfile::SupportReady,
            CompatibilityPostureProfile::CompatibilityLowered,
            AdmissionReadinessProfile::Admitted,
            RetentionDeliveryProfile::Retained,
            CertificationPostureProfile::EvidenceBacked,
        ),
        Some(FoundationalProfileNarrowingRecord::new(
            FoundationalProfileNarrowingKind::RichnessReduced,
            "proof-bearing consumers may still receive the same truth with less diagnostics",
        )),
        payload.clone(),
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(profiled) => profiled,
        other => panic!("expected proof-bearing attachment success, got {other:?}"),
    };
    assert_eq!(proof_bearing.payload().payload(), &payload);
    assert_eq!(
        proof_bearing
            .payload()
            .profile()
            .materialized()
            .diagnostic_richness(),
        DiagnosticRichnessProfile::OperationalMinimal
    );
}

#[test]
fn target_specific_attachment_denials_and_blind_consumer_reads_stay_explicit() {
    let internal_only = match admit_requested_foundational_profile(
        request_foundational_profile_set(profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::InternalOnly,
            CompatibilityPostureProfile::NativeOnly,
            AdmissionReadinessProfile::CandidateOnly,
            RetentionDeliveryProfile::Ephemeral,
            CertificationPostureProfile::Uncertified,
        )),
        profile(
            DiagnosticRichnessProfile::Standard,
            SupportPostureProfile::InternalOnly,
            CompatibilityPostureProfile::NativeOnly,
            AdmissionReadinessProfile::CandidateOnly,
            RetentionDeliveryProfile::Ephemeral,
            CertificationPostureProfile::Uncertified,
        ),
        None,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
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
        TransitionOutcome::Denied(
            FoundationalProfileAttachmentDenial::SupportArtifactsCannotCarryInternalOnlySupportPosture
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
        TransitionOutcome::Denied(
            FoundationalProfileAttachmentDenial::ProofBearingArtifactsRequireAdmittedReadiness
        )
    );

    let boundary = match attach_boundary_profiled_artifact(
        internal_only,
        profile(
            DiagnosticRichnessProfile::OperationalMinimal,
            SupportPostureProfile::InternalOnly,
            CompatibilityPostureProfile::NativeOnly,
            AdmissionReadinessProfile::CandidateOnly,
            RetentionDeliveryProfile::Ephemeral,
            CertificationPostureProfile::Uncertified,
        ),
        Some(FoundationalProfileNarrowingRecord::new(
            FoundationalProfileNarrowingKind::RichnessReduced,
            "boundary consumers may receive the same authority with less richness",
        )),
        11_u8,
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(profiled) => profiled,
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
        Some(FoundationalProfileNarrowingRecord::new(
            FoundationalProfileNarrowingKind::RichnessReduced,
            "boundary consumers may receive the same authority with less richness",
        ))
    );
}
