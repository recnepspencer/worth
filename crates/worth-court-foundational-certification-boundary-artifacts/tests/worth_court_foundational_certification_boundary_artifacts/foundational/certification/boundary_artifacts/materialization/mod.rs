use crate::certification::ordinary_context::boundary_artifacts::support::materialized_profile;
use {
    worth_foundational::admit_authoritative_current_boundary_surface,
    worth_foundational::claim_derived_projection_boundary_surface,
    worth_foundational::claim_receipt_evidence_boundary_surface,
    worth_foundational::claim_support_only_boundary_surface,
    worth_foundational::foundational_boundary_authority_admission,
    worth_foundational::materialize_authoritative_boundary_surface,
    worth_foundational::materialize_descriptive_boundary_surface,
    worth_foundational::plan_artifact_boundary_bundle,
    worth_foundational::plan_authoritative_boundary_materialization,
    worth_foundational::plan_descriptive_boundary_materialization,
};

#[test]
fn descriptive_materialization_keeps_seam_cost_and_attachment_decisions_visible() {
    let profile = materialized_profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    );
    let claim = claim_derived_projection_boundary_surface(
        worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![1_u8, 2, 3], 2),
    );
    let plan = plan_descriptive_boundary_materialization(
        claim,
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("descriptive plan");

    assert_eq!(
        plan.role(),
        worth_foundational::FoundationalBoundaryArtifactRole::DerivedProjection
    );
    assert_eq!(
        plan.disposition().delivery_class(),
        worth_foundational::FoundationalBoundaryDeliveryClass::CanDefer
    );
    assert_eq!(
        plan.disposition().availability(),
        worth_foundational::FoundationalBoundaryAvailability::Present
    );
    assert!(plan
        .attachments()
        .iter()
        .any(|attachment| attachment.point()
            == worth_foundational::FoundationalBoundaryAttachmentPoint::DiagnosticsAttachment
            && attachment.is_included()));
    assert!(plan
        .attachments()
        .iter()
        .any(|attachment| attachment.point()
            == worth_foundational::FoundationalBoundaryAttachmentPoint::CanonicalBasis
            && !attachment.is_included()));
    assert!(plan.decision_rows().iter().any(|row| row.subject()
        == worth_foundational::FoundationalBoundaryDecisionSubject::AttachmentElision
        && row.cause()
            == worth_foundational::FoundationalBoundaryDecisionCause::DeniedByMilestoneBoundary));
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
        worth_foundational::DiagnosticRichnessProfile::OperationalMinimal,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Ephemeral,
        worth_foundational::CertificationPostureProfile::Uncertified,
    );
    let claim = claim_support_only_boundary_surface(
        worth_foundational::FoundationalBoundaryReportSurface::new(vec!["row"], 1).expect("report"),
    );
    let plan = plan_descriptive_boundary_materialization(
        claim,
        worth_foundational::FoundationalBoundaryMaterializationSource::DerivedSupport,
        worth_foundational::FoundationalBoundaryMaterializationSeam::SupportMaterialization,
        profile,
    )
    .expect("support plan");

    assert_eq!(
        plan.disposition().availability(),
        worth_foundational::FoundationalBoundaryAvailability::Deferred
    );
    assert_eq!(
        plan.decision_rows()
            .iter()
            .find(|row| row.subject()
                == worth_foundational::FoundationalBoundaryDecisionSubject::DeliveryAvailabilityResolution)
            .expect("availability decision")
            .cause(),
        worth_foundational::FoundationalBoundaryDecisionCause::DeferredBySupportPosture
    );
    assert_eq!(
        materialize_descriptive_boundary_surface(
            claim_support_only_boundary_surface(
                worth_foundational::FoundationalBoundaryReportSurface::new(vec!["row"], 1)
                    .expect("report"),
            ),
            worth_foundational::FoundationalBoundaryMaterializationSource::DerivedSupport,
            worth_foundational::FoundationalBoundaryMaterializationSeam::SupportMaterialization,
            profile,
        ),
        Err(worth_foundational::FoundationalBoundaryMaterializationDenial::SurfaceDeferred)
    );
}

#[test]
fn authoritative_materialization_lane_stays_distinct_from_descriptive_paths() {
    let profile = materialized_profile(
        worth_foundational::DiagnosticRichnessProfile::Forensic,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
        worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::ProductionCertified,
    );
    let claim = admit_authoritative_current_boundary_surface(
        worth_foundational::FoundationalBoundaryArtifactSurface::new(vec!["committed"], 3),
        foundational_boundary_authority_admission(),
    );
    let plan = plan_authoritative_boundary_materialization(
        claim,
        worth_foundational::FoundationalBoundaryMaterializationSource::NativeAuthority,
        worth_foundational::FoundationalBoundaryMaterializationSeam::PersistenceExport,
        profile,
    )
    .expect("authoritative plan");

    assert!(plan.is_authority_claim());
    assert_eq!(
        plan.disposition().delivery_class(),
        worth_foundational::FoundationalBoundaryDeliveryClass::MustBeHot
    );
    assert!(plan.decision_rows().iter().any(|row| row.subject()
        == worth_foundational::FoundationalBoundaryDecisionSubject::CategoryRoleAdmission
        && row.cause()
            == worth_foundational::FoundationalBoundaryDecisionCause::NarrowedByAuthority));

    let materialized = materialize_authoritative_boundary_surface(
        admit_authoritative_current_boundary_surface(
            worth_foundational::FoundationalBoundaryArtifactSurface::new(vec!["committed"], 3),
            foundational_boundary_authority_admission(),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::NativeAuthority,
        worth_foundational::FoundationalBoundaryMaterializationSeam::PersistenceExport,
        profile,
    )
    .expect("authoritative materialized");
    assert_eq!(
        materialized.role(),
        worth_foundational::FoundationalBoundaryArtifactRole::AuthoritativeCurrent
    );
}

#[test]
fn delivery_and_source_seam_legality_remain_explicit_and_fail_closed() {
    assert_eq!(
        worth_foundational::evaluate_boundary_surface_disposition_legality(
            worth_foundational::FoundationalBoundaryDeliveryClass::MustBeHot,
            worth_foundational::FoundationalBoundaryAvailability::Deferred,
        ),
        Err(worth_foundational::FoundationalBoundarySurfaceDispositionDenial::MustBeHotCannotDefer)
    );
    assert_eq!(
        worth_foundational::evaluate_boundary_surface_disposition_legality(
            worth_foundational::FoundationalBoundaryDeliveryClass::ReconstructableFromRetainedBasis,
            worth_foundational::FoundationalBoundaryAvailability::Present,
        ),
        Err(
            worth_foundational::FoundationalBoundarySurfaceDispositionDenial::ReconstructableDeliveryCannotAppearPresent
        )
    );

    let profile = materialized_profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    );
    let illegal = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![1_u8], 0),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::SupportMaterialization,
        profile,
    );
    assert_eq!(
        illegal,
        Err(
            worth_foundational::FoundationalBoundaryPlanningDenial::CompatibilityLoweredCannotUseSupportMaterialization
        )
    );
}

#[test]
fn coordinated_bundle_materialization_preserves_member_categories_and_membership_rows() {
    let profile = materialized_profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    );
    let primary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![7_u8, 8, 9], 2),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("primary plan");
    let summary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundarySummarySurface::new("summary", 2)
                .expect("summary"),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("summary plan");
    let report = plan_descriptive_boundary_materialization(
        claim_support_only_boundary_surface(
            worth_foundational::FoundationalBoundaryReportSurface::new(vec!["row-a", "row-b"], 2)
                .expect("report"),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("report plan");
    let receipt = plan_descriptive_boundary_materialization(
        claim_receipt_evidence_boundary_surface(
            worth_foundational::FoundationalBoundaryReceiptSurface::new("exchange complete", 1)
                .expect("receipt"),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
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
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered
    );
    assert_eq!(
        bundle_plan.seam(),
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange
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
            .filter(|row| row.subject()
                == worth_foundational::FoundationalBoundaryDecisionSubject::BundleMembership)
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
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered
    );
    assert_eq!(
        bundle.seam(),
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange
    );
    assert_eq!(bundle.profile(), bundle.primary().profile());
    assert!(bundle.summary().is_some());
    assert!(bundle.report().is_some());
    assert!(bundle.receipt().is_some());
}

#[test]
fn bundle_members_must_match_primary_profile_source_and_seam() {
    let primary_profile = materialized_profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    );
    let secondary_profile = materialized_profile(
        worth_foundational::DiagnosticRichnessProfile::OperationalMinimal,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::Uncertified,
    );
    let primary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![1_u8], 1),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        primary_profile,
    )
    .expect("primary plan");
    let wrong_summary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundarySummarySurface::new("summary", 1)
                .expect("summary"),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        secondary_profile,
    )
    .expect("summary plan");

    assert_eq!(
        plan_artifact_boundary_bundle(primary).with_summary(wrong_summary),
        Err(worth_foundational::FoundationalBoundaryBundlePlanningDenial::SummaryProfileMismatch)
    );
}
