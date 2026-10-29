use worth_foundational::{
    boundary_artifact_surface_inventory, foundational_profile_applicability,
    plan_foundational_profile_materialization,
    plan_foundational_profile_materialization_with_elision,
    plan_selected_foundational_profile_materialization, proof_bearing_artifact_surface_inventory,
    support_artifact_surface_inventory, AdmissionReadinessProfile, BoundaryArtifactTarget,
    CertificationPostureProfile, CompatibilityPostureProfile, DiagnosticRichnessProfile,
    FoundationalDescriptiveElisionProfile, FoundationalDescriptiveSurface,
    FoundationalMaterializationPlanningDenial, FoundationalProfileDecisionKind,
    FoundationalProfileFamily, FoundationalSurfaceAbsenceCause, MaterializedFoundationalProfileSet,
    ProofBearingArtifactTarget, RetentionDeliveryProfile, SupportArtifactTarget,
    SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use super::support::{admit_same_profile, profile};

fn materialized_profile(
    materialized: worth_foundational::FoundationalProfileSet,
) -> MaterializedFoundationalProfileSet {
    let admitted = admit_same_profile(*materialized.admitted_or_self());

    match worth_foundational::materialize_admitted_foundational_profile(
        admitted,
        materialized,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(materialized) => *materialized.payload(),
        outcome => panic!("expected materialized profile, got {outcome:?}"),
    }
}

trait MaterializedSeed {
    fn admitted_or_self(&self) -> &worth_foundational::FoundationalProfileSet;
}

impl MaterializedSeed for worth_foundational::FoundationalProfileSet {
    fn admitted_or_self(&self) -> &worth_foundational::FoundationalProfileSet {
        self
    }
}

#[test]
fn target_surface_inventories_and_selected_plans_stay_closed_and_explicit() {
    assert_eq!(
        boundary_artifact_surface_inventory().surfaces(),
        &[
            FoundationalDescriptiveSurface::History,
            FoundationalDescriptiveSurface::Replay,
            FoundationalDescriptiveSurface::Lineage,
            FoundationalDescriptiveSurface::Provenance,
            FoundationalDescriptiveSurface::ForensicDiagnostics,
        ]
    );
    assert_eq!(
        support_artifact_surface_inventory().surfaces(),
        &[
            FoundationalDescriptiveSurface::History,
            FoundationalDescriptiveSurface::Replay,
            FoundationalDescriptiveSurface::Provenance,
            FoundationalDescriptiveSurface::ForensicDiagnostics,
        ]
    );
    assert_eq!(
        proof_bearing_artifact_surface_inventory().surfaces(),
        &[
            FoundationalDescriptiveSurface::Provenance,
            FoundationalDescriptiveSurface::ForensicDiagnostics,
        ]
    );

    let full = materialized_profile(profile(
        DiagnosticRichnessProfile::Forensic,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityRequired,
        AdmissionReadinessProfile::ProductionGateReady,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::ProductionCertified,
    ));
    let exhaustive = plan_foundational_profile_materialization::<BoundaryArtifactTarget>(&full)
        .expect("continuous profile should carry its default disposition");
    let selected = plan_selected_foundational_profile_materialization::<BoundaryArtifactTarget>(
        &full,
        &[
            FoundationalDescriptiveSurface::History,
            FoundationalDescriptiveSurface::Replay,
        ],
    )
    .expect("selected boundary plan");

    assert_eq!(exhaustive.cost().inventory_surface_count(), 5);
    assert_eq!(selected.cost().requested_surface_count(), 2);
    assert_eq!(
        selected
            .decision_for(FoundationalDescriptiveSurface::Lineage)
            .expect("lineage decision")
            .absence_cause(),
        Some(FoundationalSurfaceAbsenceCause::DeniedByBudget)
    );
    assert_eq!(
        plan_selected_foundational_profile_materialization::<SupportArtifactTarget>(&full, &[]),
        Err(FoundationalMaterializationPlanningDenial::EmptySelectedSurfaceSet)
    );
    assert_eq!(
        plan_selected_foundational_profile_materialization::<SupportArtifactTarget>(
            &full,
            &[
                FoundationalDescriptiveSurface::History,
                FoundationalDescriptiveSurface::History,
            ],
        ),
        Err(FoundationalMaterializationPlanningDenial::DuplicateSelectedSurface)
    );
    assert_eq!(
        plan_selected_foundational_profile_materialization::<SupportArtifactTarget>(
            &full,
            &[FoundationalDescriptiveSurface::Lineage],
        ),
        Err(FoundationalMaterializationPlanningDenial::SurfaceIllegalForTarget)
    );

    let operational_summary =
        plan_foundational_profile_materialization_with_elision::<SupportArtifactTarget>(
            &full,
            FoundationalDescriptiveElisionProfile::OperationalSummary,
        )
        .expect("continuous profile should carry its default disposition");
    assert_eq!(operational_summary.cost().requested_surface_count(), 2);
    assert!(operational_summary
        .decision_for(FoundationalDescriptiveSurface::History)
        .expect("history decision")
        .is_available());
    assert!(operational_summary
        .decision_for(FoundationalDescriptiveSurface::Provenance)
        .expect("provenance decision")
        .is_available());
    assert_eq!(
        operational_summary
            .decision_for(FoundationalDescriptiveSurface::Replay)
            .expect("replay decision")
            .absence_cause(),
        Some(FoundationalSurfaceAbsenceCause::DeniedByBudget)
    );
}

#[test]
fn absence_causes_and_target_applicability_remain_structurally_distinct() {
    let omitted = materialized_profile(profile(
        DiagnosticRichnessProfile::OperationalMinimal,
        SupportPostureProfile::SupportReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Retained,
        CertificationPostureProfile::EvidenceBacked,
    ));
    let not_retained = materialized_profile(profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::SupportReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Ephemeral,
        CertificationPostureProfile::Uncertified,
    ));
    let not_reconstructable = materialized_profile(profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::SupportReady,
        CompatibilityPostureProfile::NativeOnly,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Retained,
        CertificationPostureProfile::EvidenceBacked,
    ));
    let deferred = materialized_profile(profile(
        DiagnosticRichnessProfile::Forensic,
        SupportPostureProfile::SupportReady,
        CompatibilityPostureProfile::CompatibilityRequired,
        AdmissionReadinessProfile::ProductionGateReady,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::EvidenceBacked,
    ));
    let uncertified = materialized_profile(profile(
        DiagnosticRichnessProfile::Forensic,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityRequired,
        AdmissionReadinessProfile::ProductionGateReady,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::EvidenceBacked,
    ));

    assert_eq!(
        plan_foundational_profile_materialization::<BoundaryArtifactTarget>(&omitted)
            .expect("continuous profile should carry its default disposition")
            .decision_for(FoundationalDescriptiveSurface::History)
            .expect("history")
            .absence_cause(),
        Some(FoundationalSurfaceAbsenceCause::OmittedByActiveRichness)
    );
    assert_eq!(
        plan_foundational_profile_materialization::<BoundaryArtifactTarget>(&not_retained)
            .expect("continuous profile should carry its default disposition")
            .decision_for(FoundationalDescriptiveSurface::History)
            .expect("history")
            .absence_cause(),
        Some(FoundationalSurfaceAbsenceCause::NotRetained)
    );
    assert_eq!(
        plan_foundational_profile_materialization::<BoundaryArtifactTarget>(&not_reconstructable)
            .expect("continuous profile should carry its default disposition")
            .decision_for(FoundationalDescriptiveSurface::Replay)
            .expect("replay")
            .absence_cause(),
        Some(FoundationalSurfaceAbsenceCause::NotReconstructable)
    );
    assert_eq!(
        plan_foundational_profile_materialization::<SupportArtifactTarget>(&deferred)
            .expect("continuous profile should carry its default disposition")
            .decision_for(FoundationalDescriptiveSurface::Provenance)
            .expect("provenance")
            .absence_cause(),
        Some(FoundationalSurfaceAbsenceCause::DeferredBySupportPosture)
    );
    assert_eq!(
        plan_foundational_profile_materialization::<SupportArtifactTarget>(&uncertified)
            .expect("continuous profile should carry its default disposition")
            .decision_for(FoundationalDescriptiveSurface::ForensicDiagnostics)
            .expect("forensic")
            .absence_cause(),
        Some(FoundationalSurfaceAbsenceCause::UncertifiedForRequestedPosture)
    );

    let support = foundational_profile_applicability::<SupportArtifactTarget>();
    let proof = foundational_profile_applicability::<ProofBearingArtifactTarget>();
    assert!(support.governs(
        FoundationalDescriptiveSurface::ForensicDiagnostics,
        FoundationalProfileFamily::CertificationPosture
    ));
    assert!(support.governs(
        FoundationalDescriptiveSurface::Provenance,
        FoundationalProfileFamily::RetentionDelivery
    ));
    assert!(support
        .governing_families(FoundationalDescriptiveSurface::Lineage)
        .is_none());
    assert!(proof
        .governing_families(FoundationalDescriptiveSurface::Lineage)
        .is_none());
    assert!(support.governs_decision(
        FoundationalDescriptiveSurface::Provenance,
        FoundationalProfileDecisionKind::SupportPostureDeferral
    ));
    assert!(support.governs_decision(
        FoundationalDescriptiveSurface::Provenance,
        FoundationalProfileDecisionKind::RetentionAvailability
    ));
    assert!(support.governs_decision(
        FoundationalDescriptiveSurface::ForensicDiagnostics,
        FoundationalProfileDecisionKind::CertificationPostureRequirement
    ));
    assert!(!proof.governs_decision(
        FoundationalDescriptiveSurface::Provenance,
        FoundationalProfileDecisionKind::SupportPostureDeferral
    ));
}
