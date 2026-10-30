#[allow(unused_imports)]
use crate::certification::ordinary_context::profiles::materialization::*;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::boundary_artifact_surface_inventory,
    worth_foundational::foundational_profile_applicability,
    worth_foundational::plan_foundational_profile_materialization,
    worth_foundational::plan_foundational_profile_materialization_with_elision,
    worth_foundational::plan_selected_foundational_profile_materialization,
    worth_foundational::proof_bearing_artifact_surface_inventory,
    worth_foundational::support_artifact_surface_inventory,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::BoundaryArtifactTarget,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalDescriptiveElisionProfile,
    worth_foundational::FoundationalDescriptiveSurface,
    worth_foundational::FoundationalMaterializationPlanningDenial,
    worth_foundational::FoundationalProfileDecisionKind,
    worth_foundational::FoundationalProfileFamily,
    worth_foundational::FoundationalSurfaceAbsenceCause,
    worth_foundational::MaterializedFoundationalProfileSet,
    worth_foundational::ProofBearingArtifactTarget, worth_foundational::RetentionDeliveryProfile,
    worth_foundational::SupportArtifactTarget, worth_foundational::SupportPostureProfile,
};

use {
    crate::certification::ordinary_context::profiles::support::admit_same_profile,
    crate::certification::ordinary_context::profiles::support::profile,
};

#[test]
fn target_surface_inventories_and_selected_plans_stay_closed_and_explicit() {
    assert_eq!(
        boundary_artifact_surface_inventory().surfaces(),
        &[
            worth_foundational::FoundationalDescriptiveSurface::History,
            worth_foundational::FoundationalDescriptiveSurface::Replay,
            worth_foundational::FoundationalDescriptiveSurface::Lineage,
            worth_foundational::FoundationalDescriptiveSurface::Provenance,
            worth_foundational::FoundationalDescriptiveSurface::ForensicDiagnostics,
        ]
    );
    assert_eq!(
        support_artifact_surface_inventory().surfaces(),
        &[
            worth_foundational::FoundationalDescriptiveSurface::History,
            worth_foundational::FoundationalDescriptiveSurface::Replay,
            worth_foundational::FoundationalDescriptiveSurface::Provenance,
            worth_foundational::FoundationalDescriptiveSurface::ForensicDiagnostics,
        ]
    );
    assert_eq!(
        proof_bearing_artifact_surface_inventory().surfaces(),
        &[
            worth_foundational::FoundationalDescriptiveSurface::Provenance,
            worth_foundational::FoundationalDescriptiveSurface::ForensicDiagnostics,
        ]
    );

    let full = materialized_profile(profile(
        worth_foundational::DiagnosticRichnessProfile::Forensic,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::ProductionCertified,
    ));
    let exhaustive = plan_foundational_profile_materialization::<BoundaryArtifactTarget>(&full)
        .expect("continuous profile should carry its default disposition");
    let selected = plan_selected_foundational_profile_materialization::<BoundaryArtifactTarget>(
        &full,
        &[
            worth_foundational::FoundationalDescriptiveSurface::History,
            worth_foundational::FoundationalDescriptiveSurface::Replay,
        ],
    )
    .expect("selected boundary plan");

    assert_eq!(exhaustive.cost().inventory_surface_count(), 5);
    assert_eq!(selected.cost().requested_surface_count(), 2);
    assert_eq!(
        selected
            .decision_for(worth_foundational::FoundationalDescriptiveSurface::Lineage)
            .expect("lineage decision")
            .absence_cause(),
        Some(worth_foundational::FoundationalSurfaceAbsenceCause::DeniedByBudget)
    );
    assert_eq!(
        plan_selected_foundational_profile_materialization::<SupportArtifactTarget>(&full, &[]),
        Err(worth_foundational::FoundationalMaterializationPlanningDenial::EmptySelectedSurfaceSet)
    );
    assert_eq!(
        plan_selected_foundational_profile_materialization::<SupportArtifactTarget>(
            &full,
            &[
                worth_foundational::FoundationalDescriptiveSurface::History,
                worth_foundational::FoundationalDescriptiveSurface::History,
            ],
        ),
        Err(
            worth_foundational::FoundationalMaterializationPlanningDenial::DuplicateSelectedSurface
        )
    );
    assert_eq!(
        plan_selected_foundational_profile_materialization::<SupportArtifactTarget>(
            &full,
            &[worth_foundational::FoundationalDescriptiveSurface::Lineage],
        ),
        Err(worth_foundational::FoundationalMaterializationPlanningDenial::SurfaceIllegalForTarget)
    );

    let operational_summary =
        plan_foundational_profile_materialization_with_elision::<SupportArtifactTarget>(
            &full,
            worth_foundational::FoundationalDescriptiveElisionProfile::OperationalSummary,
        )
        .expect("continuous profile should carry its default disposition");
    assert_eq!(operational_summary.cost().requested_surface_count(), 2);
    assert!(operational_summary
        .decision_for(worth_foundational::FoundationalDescriptiveSurface::History)
        .expect("history decision")
        .is_available());
    assert!(operational_summary
        .decision_for(worth_foundational::FoundationalDescriptiveSurface::Provenance)
        .expect("provenance decision")
        .is_available());
    assert_eq!(
        operational_summary
            .decision_for(worth_foundational::FoundationalDescriptiveSurface::Replay)
            .expect("replay decision")
            .absence_cause(),
        Some(worth_foundational::FoundationalSurfaceAbsenceCause::DeniedByBudget)
    );
}

#[test]
fn absence_causes_and_target_applicability_remain_structurally_distinct() {
    let omitted = materialized_profile(profile(
        worth_foundational::DiagnosticRichnessProfile::OperationalMinimal,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    ));
    let not_retained = materialized_profile(profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Ephemeral,
        worth_foundational::CertificationPostureProfile::Uncertified,
    ));
    let not_reconstructable = materialized_profile(profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::NativeOnly,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    ));
    let deferred = materialized_profile(profile(
        worth_foundational::DiagnosticRichnessProfile::Forensic,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    ));
    let uncertified = materialized_profile(profile(
        worth_foundational::DiagnosticRichnessProfile::Forensic,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    ));

    assert_eq!(
        plan_foundational_profile_materialization::<BoundaryArtifactTarget>(&omitted)
            .expect("continuous profile should carry its default disposition")
            .decision_for(worth_foundational::FoundationalDescriptiveSurface::History)
            .expect("history")
            .absence_cause(),
        Some(worth_foundational::FoundationalSurfaceAbsenceCause::OmittedByActiveRichness)
    );
    assert_eq!(
        plan_foundational_profile_materialization::<BoundaryArtifactTarget>(&not_retained)
            .expect("continuous profile should carry its default disposition")
            .decision_for(worth_foundational::FoundationalDescriptiveSurface::History)
            .expect("history")
            .absence_cause(),
        Some(worth_foundational::FoundationalSurfaceAbsenceCause::NotRetained)
    );
    assert_eq!(
        plan_foundational_profile_materialization::<BoundaryArtifactTarget>(&not_reconstructable)
            .expect("continuous profile should carry its default disposition")
            .decision_for(worth_foundational::FoundationalDescriptiveSurface::Replay)
            .expect("replay")
            .absence_cause(),
        Some(worth_foundational::FoundationalSurfaceAbsenceCause::NotReconstructable)
    );
    assert_eq!(
        plan_foundational_profile_materialization::<SupportArtifactTarget>(&deferred)
            .expect("continuous profile should carry its default disposition")
            .decision_for(worth_foundational::FoundationalDescriptiveSurface::Provenance)
            .expect("provenance")
            .absence_cause(),
        Some(worth_foundational::FoundationalSurfaceAbsenceCause::DeferredBySupportPosture)
    );
    assert_eq!(
        plan_foundational_profile_materialization::<SupportArtifactTarget>(&uncertified)
            .expect("continuous profile should carry its default disposition")
            .decision_for(worth_foundational::FoundationalDescriptiveSurface::ForensicDiagnostics)
            .expect("forensic")
            .absence_cause(),
        Some(worth_foundational::FoundationalSurfaceAbsenceCause::UncertifiedForRequestedPosture)
    );

    let support = foundational_profile_applicability::<SupportArtifactTarget>();
    let proof = foundational_profile_applicability::<ProofBearingArtifactTarget>();
    assert!(support.governs(
        worth_foundational::FoundationalDescriptiveSurface::ForensicDiagnostics,
        worth_foundational::FoundationalProfileFamily::CertificationPosture
    ));
    assert!(support.governs(
        worth_foundational::FoundationalDescriptiveSurface::Provenance,
        worth_foundational::FoundationalProfileFamily::RetentionDelivery
    ));
    assert!(support
        .governing_families(worth_foundational::FoundationalDescriptiveSurface::Lineage)
        .is_none());
    assert!(proof
        .governing_families(worth_foundational::FoundationalDescriptiveSurface::Lineage)
        .is_none());
    assert!(support.governs_decision(
        worth_foundational::FoundationalDescriptiveSurface::Provenance,
        worth_foundational::FoundationalProfileDecisionKind::SupportPostureDeferral
    ));
    assert!(support.governs_decision(
        worth_foundational::FoundationalDescriptiveSurface::Provenance,
        worth_foundational::FoundationalProfileDecisionKind::RetentionAvailability
    ));
    assert!(support.governs_decision(
        worth_foundational::FoundationalDescriptiveSurface::ForensicDiagnostics,
        worth_foundational::FoundationalProfileDecisionKind::CertificationPostureRequirement
    ));
    assert!(!proof.governs_decision(
        worth_foundational::FoundationalDescriptiveSurface::Provenance,
        worth_foundational::FoundationalProfileDecisionKind::SupportPostureDeferral
    ));
}
