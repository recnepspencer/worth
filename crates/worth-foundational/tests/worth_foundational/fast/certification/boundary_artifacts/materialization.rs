use super::support::materialized_profile;
use worth_foundational::{
    admit_authoritative_current_boundary_surface, claim_derived_projection_boundary_surface,
    claim_receipt_evidence_boundary_surface, claim_support_only_boundary_surface,
    foundational_boundary_authority_admission, materialize_authoritative_boundary_surface,
    materialize_descriptive_boundary_surface, plan_artifact_boundary_bundle,
    plan_authoritative_boundary_materialization, plan_descriptive_boundary_materialization,
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, FoundationalBoundaryArtifactRole,
    FoundationalBoundaryArtifactSurface, FoundationalBoundaryAttachmentPoint,
    FoundationalBoundaryAvailability, FoundationalBoundaryBundlePlanningDenial,
    FoundationalBoundaryDecisionCause, FoundationalBoundaryDecisionSubject,
    FoundationalBoundaryDeliveryClass, FoundationalBoundaryMaterializationDenial,
    FoundationalBoundaryMaterializationSeam, FoundationalBoundaryMaterializationSource,
    FoundationalBoundaryPlanningDenial, FoundationalBoundaryReceiptSurface,
    FoundationalBoundaryReportSurface, FoundationalBoundarySummarySurface,
    FoundationalBoundarySurfaceDispositionDenial, RetentionDeliveryProfile, SupportPostureProfile,
};

#[test]
fn descriptive_materialization_keeps_seam_cost_and_attachment_decisions_visible() {
    let profile = materialized_profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::EvidenceBacked,
    );
    let claim = claim_derived_projection_boundary_surface(
        FoundationalBoundaryArtifactSurface::new(vec![1_u8, 2, 3], 2),
    );
    let plan = plan_descriptive_boundary_materialization(
        claim,
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("descriptive plan");

    assert_eq!(
        plan.role(),
        FoundationalBoundaryArtifactRole::DerivedProjection
    );
    assert_eq!(
        plan.disposition().delivery_class(),
        FoundationalBoundaryDeliveryClass::CanDefer
    );
    assert_eq!(
        plan.disposition().availability(),
        FoundationalBoundaryAvailability::Present
    );
    assert!(plan
        .attachments()
        .iter()
        .any(|attachment| attachment.point()
            == FoundationalBoundaryAttachmentPoint::DiagnosticsAttachment
            && attachment.is_included()));
    assert!(plan
        .attachments()
        .iter()
        .any(|attachment| attachment.point()
            == FoundationalBoundaryAttachmentPoint::CanonicalBasis
            && !attachment.is_included()));
    assert!(plan.decision_rows().iter().any(|row| row.subject()
        == FoundationalBoundaryDecisionSubject::AttachmentElision
        && row.cause() == FoundationalBoundaryDecisionCause::DeniedByMilestoneBoundary));
    assert!(plan
        .decision_rows()
        .iter()
        .all(|row| row.category() == Some(plan.category())));
    assert!(plan.cost().decision_row_count() >= 4);

    let materialized = plan.clone().materialize().expect("materialized artifact");
    assert_eq!(materialized.seam(), plan.seam());
    assert_eq!(materialized.source(), plan.source());
    assert_eq!(materialized.cost(), plan.cost());
}

#[test]
fn support_materialization_stays_plannable_before_becoming_deferred_runtime_work() {
    let profile = materialized_profile(
        DiagnosticRichnessProfile::OperationalMinimal,
        SupportPostureProfile::SupportReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Ephemeral,
        CertificationPostureProfile::Uncertified,
    );
    let claim = claim_support_only_boundary_surface(
        FoundationalBoundaryReportSurface::new(vec!["row"], 1).expect("report"),
    );
    let plan = plan_descriptive_boundary_materialization(
        claim,
        FoundationalBoundaryMaterializationSource::DerivedSupport,
        FoundationalBoundaryMaterializationSeam::SupportMaterialization,
        profile,
    )
    .expect("support plan");

    assert_eq!(
        plan.disposition().availability(),
        FoundationalBoundaryAvailability::Deferred
    );
    assert_eq!(
        plan.decision_rows()
            .iter()
            .find(|row| row.subject()
                == FoundationalBoundaryDecisionSubject::DeliveryAvailabilityResolution)
            .expect("availability decision")
            .cause(),
        FoundationalBoundaryDecisionCause::DeferredBySupportPosture
    );
    assert_eq!(
        materialize_descriptive_boundary_surface(
            claim_support_only_boundary_surface(
                FoundationalBoundaryReportSurface::new(vec!["row"], 1).expect("report"),
            ),
            FoundationalBoundaryMaterializationSource::DerivedSupport,
            FoundationalBoundaryMaterializationSeam::SupportMaterialization,
            profile,
        ),
        Err(FoundationalBoundaryMaterializationDenial::SurfaceDeferred)
    );
}

#[test]
fn authoritative_materialization_lane_stays_distinct_from_descriptive_paths() {
    let profile = materialized_profile(
        DiagnosticRichnessProfile::Forensic,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityRequired,
        AdmissionReadinessProfile::ProductionGateReady,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::ProductionCertified,
    );
    let claim = admit_authoritative_current_boundary_surface(
        FoundationalBoundaryArtifactSurface::new(vec!["committed"], 3),
        foundational_boundary_authority_admission(),
    );
    let plan = plan_authoritative_boundary_materialization(
        claim,
        FoundationalBoundaryMaterializationSource::NativeAuthority,
        FoundationalBoundaryMaterializationSeam::PersistenceExport,
        profile,
    )
    .expect("authoritative plan");

    assert!(plan.is_authority_claim());
    assert_eq!(
        plan.disposition().delivery_class(),
        FoundationalBoundaryDeliveryClass::MustBeHot
    );
    assert!(plan.decision_rows().iter().any(|row| row.subject()
        == FoundationalBoundaryDecisionSubject::CategoryRoleAdmission
        && row.cause() == FoundationalBoundaryDecisionCause::NarrowedByAuthority));

    let materialized = materialize_authoritative_boundary_surface(
        admit_authoritative_current_boundary_surface(
            FoundationalBoundaryArtifactSurface::new(vec!["committed"], 3),
            foundational_boundary_authority_admission(),
        ),
        FoundationalBoundaryMaterializationSource::NativeAuthority,
        FoundationalBoundaryMaterializationSeam::PersistenceExport,
        profile,
    )
    .expect("authoritative materialized");
    assert_eq!(
        materialized.role(),
        FoundationalBoundaryArtifactRole::AuthoritativeCurrent
    );
}

#[test]
fn delivery_and_source_seam_legality_remain_explicit_and_fail_closed() {
    assert_eq!(
        worth_foundational::evaluate_boundary_surface_disposition_legality(
            FoundationalBoundaryDeliveryClass::MustBeHot,
            FoundationalBoundaryAvailability::Deferred,
        ),
        Err(FoundationalBoundarySurfaceDispositionDenial::MustBeHotCannotDefer)
    );
    assert_eq!(
        worth_foundational::evaluate_boundary_surface_disposition_legality(
            FoundationalBoundaryDeliveryClass::ReconstructableFromRetainedBasis,
            FoundationalBoundaryAvailability::Present,
        ),
        Err(
            FoundationalBoundarySurfaceDispositionDenial::ReconstructableDeliveryCannotAppearPresent
        )
    );

    let profile = materialized_profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::EvidenceBacked,
    );
    let illegal = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![1_u8],
            0,
        )),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::SupportMaterialization,
        profile,
    );
    assert_eq!(
        illegal,
        Err(
            FoundationalBoundaryPlanningDenial::CompatibilityLoweredCannotUseSupportMaterialization
        )
    );
}

#[test]
fn coordinated_bundle_materialization_preserves_member_categories_and_membership_rows() {
    let profile = materialized_profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::EvidenceBacked,
    );
    let primary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![7_u8, 8, 9],
            2,
        )),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("primary plan");
    let summary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            FoundationalBoundarySummarySurface::new("summary", 2).expect("summary"),
        ),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("summary plan");
    let report = plan_descriptive_boundary_materialization(
        claim_support_only_boundary_surface(
            FoundationalBoundaryReportSurface::new(vec!["row-a", "row-b"], 2).expect("report"),
        ),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("report plan");
    let receipt = plan_descriptive_boundary_materialization(
        claim_receipt_evidence_boundary_surface(
            FoundationalBoundaryReceiptSurface::new("exchange complete", 1).expect("receipt"),
        ),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("receipt plan");

    let bundle_plan = plan_artifact_boundary_bundle(primary)
        .with_summary(summary)
        .expect("summary legality")
        .with_report(report)
        .expect("report legality")
        .with_receipt(receipt)
        .expect("receipt legality");

    assert_eq!(bundle_plan.cost().member_count(), 4);
    assert_eq!(
        bundle_plan.source(),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered
    );
    assert_eq!(
        bundle_plan.seam(),
        FoundationalBoundaryMaterializationSeam::BoundaryExchange
    );
    assert_eq!(bundle_plan.profile(), bundle_plan.primary().profile());
    assert_eq!(
        bundle_plan.summary().expect("summary").category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Summary
    );
    assert_eq!(
        bundle_plan.report().expect("report").category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Report
    );
    assert_eq!(
        bundle_plan.receipt().expect("receipt").category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Receipt
    );
    assert_eq!(
        bundle_plan
            .membership_decision_rows()
            .iter()
            .filter(|row| row.subject() == FoundationalBoundaryDecisionSubject::BundleMembership)
            .count(),
        3
    );
    assert!(bundle_plan
        .membership_decision_rows()
        .iter()
        .any(|row| row.category()
            == Some(worth_foundational::FoundationalBoundaryArtifactCategory::Summary)));
    assert!(bundle_plan
        .membership_decision_rows()
        .iter()
        .any(|row| row.category()
            == Some(worth_foundational::FoundationalBoundaryArtifactCategory::Report)));
    assert!(bundle_plan
        .membership_decision_rows()
        .iter()
        .any(|row| row.category()
            == Some(worth_foundational::FoundationalBoundaryArtifactCategory::Receipt)));

    let bundle = bundle_plan.materialize().expect("bundle materialized");
    assert_eq!(
        bundle.primary().category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Artifact
    );
    assert_eq!(
        bundle.source(),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered
    );
    assert_eq!(
        bundle.seam(),
        FoundationalBoundaryMaterializationSeam::BoundaryExchange
    );
    assert_eq!(bundle.profile(), bundle.primary().profile());
    assert!(bundle.summary().is_some());
    assert!(bundle.report().is_some());
    assert!(bundle.receipt().is_some());
}

#[test]
fn bundle_members_must_match_primary_profile_source_and_seam() {
    let primary_profile = materialized_profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::EvidenceBacked,
    );
    let secondary_profile = materialized_profile(
        DiagnosticRichnessProfile::OperationalMinimal,
        SupportPostureProfile::SupportReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::Uncertified,
    );
    let primary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![1_u8],
            1,
        )),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        primary_profile,
    )
    .expect("primary plan");
    let wrong_summary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            FoundationalBoundarySummarySurface::new("summary", 1).expect("summary"),
        ),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        secondary_profile,
    )
    .expect("summary plan");

    assert_eq!(
        plan_artifact_boundary_bundle(primary).with_summary(wrong_summary),
        Err(FoundationalBoundaryBundlePlanningDenial::SummaryProfileMismatch)
    );
}
