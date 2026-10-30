#[allow(unused_imports)]
use crate::certification::ordinary_context::boundary_artifacts::canonical_basis::*;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::claim_derived_projection_boundary_surface,
    worth_foundational::claim_receipt_evidence_boundary_surface,
    worth_foundational::claim_support_only_boundary_surface,
    worth_foundational::foundational_boundary_canonical_basis_entries,
    worth_foundational::plan_artifact_boundary_bundle,
    worth_foundational::plan_descriptive_boundary_materialization,
    worth_foundational::prepare_materialized_boundary_artifact_for_canonical_basis,
    worth_foundational::prepare_materialized_boundary_bundle_for_canonical_basis,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::CanonicalBasisDomain,
    worth_foundational::CanonicalBasisEntry, worth_foundational::CanonicalBasisEntryKind,
    worth_foundational::CanonicalBasisLocus, worth_foundational::CanonicalBasisValue,
    worth_foundational::CanonicalComparisonOutcome,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalBoundaryArtifactSurface,
    worth_foundational::FoundationalBoundaryMaterializationSeam,
    worth_foundational::FoundationalBoundaryMaterializationSource,
    worth_foundational::FoundationalBoundaryReceiptSurface,
    worth_foundational::FoundationalBoundaryReportSurface,
    worth_foundational::FoundationalBoundarySummarySurface,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

use {
    crate::certification::ordinary_context::boundary_artifacts::support::exact_compare,
    crate::certification::ordinary_context::boundary_artifacts::support::materialize_projection_artifact,
    crate::certification::ordinary_context::boundary_artifacts::support::materialized_profile,
    crate::certification::ordinary_context::boundary_artifacts::support::version,
};

#[test]
fn materialized_boundary_artifacts_lower_shared_semantics_not_payload_layout() {
    let profile = materialized_profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    );
    let left = materialize_projection_artifact(vec![1_u8, 2, 3], 2, profile);
    let right = materialize_projection_artifact(vec![9_u8, 8, 7], 2, profile);

    let left_ready = match prepare_materialized_boundary_artifact_for_canonical_basis(
        version("m4.phase4.single"),
        &left,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready boundary basis"),
    };
    let right_ready = match prepare_materialized_boundary_artifact_for_canonical_basis(
        version("m4.phase4.single"),
        &right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready boundary basis"),
    };

    assert_eq!(
        left_ready.payload().domain(),
        worth_foundational::CanonicalBasisDomain::BoundaryArtifact
    );
    match exact_compare(
        match prepare_materialized_boundary_artifact_for_canonical_basis(
            version("m4.phase4.single"),
            &left,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected ready boundary basis"),
        },
        right_ready,
    ) {
        worth_foundational::CanonicalComparisonOutcome::Equivalent(equivalent) => {
            assert_eq!(
                equivalent.domain(),
                worth_foundational::CanonicalBasisDomain::BoundaryArtifact
            );
        }
        outcome => panic!("same boundary semantics should compare equivalent, got {outcome:?}"),
    }

    let entries = foundational_boundary_canonical_basis_entries(&left_ready);
    assert_eq!(
        named_entry(entries, "shape").value(),
        &worth_foundational::CanonicalBasisValue::ExactText("single-surface".into())
    );
    assert_eq!(
        named_entry(entries, "surface.category").value(),
        &worth_foundational::CanonicalBasisValue::ExactText("artifact".into())
    );
    assert_eq!(
        named_entry(entries, "surface.role").value(),
        &worth_foundational::CanonicalBasisValue::ExactText("derived-projection".into())
    );
    assert_eq!(
        named_entry(entries, "surface.source").value(),
        &worth_foundational::CanonicalBasisValue::ExactText("compatibility-lowered".into())
    );
    assert_eq!(
        named_entry(entries, "surface.seam").value(),
        &worth_foundational::CanonicalBasisValue::ExactText("boundary-exchange".into())
    );
    assert_eq!(
        named_entry(entries, "surface.attachment.diagnostics-attachment").kind(),
        worth_foundational::CanonicalBasisEntryKind::BoundaryAttachment
    );
    assert!(!entries
        .iter()
        .any(|entry| entry.kind() == worth_foundational::CanonicalBasisEntryKind::Cost));
}

#[test]
fn coordinated_boundary_bundles_reuse_one_boundary_artifact_basis_domain() {
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
            worth_foundational::FoundationalBoundarySummarySurface::new("summary alpha", 2)
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
    let first_bundle = plan_artifact_boundary_bundle(primary)
        .with_summary(summary)
        .expect("summary legality")
        .with_report(report)
        .expect("report legality")
        .with_receipt(receipt)
        .expect("receipt legality")
        .materialize()
        .expect("bundle materialized");

    let second_bundle = plan_artifact_boundary_bundle(
        plan_descriptive_boundary_materialization(
            claim_derived_projection_boundary_surface(
                worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![0_u8, 1, 2], 2),
            ),
            worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("primary plan"),
    )
    .with_summary(
        plan_descriptive_boundary_materialization(
            claim_derived_projection_boundary_surface(
                worth_foundational::FoundationalBoundarySummarySurface::new("summary beta", 2)
                    .expect("summary"),
            ),
            worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("summary plan"),
    )
    .expect("summary legality")
    .with_report(
        plan_descriptive_boundary_materialization(
            claim_support_only_boundary_surface(
                worth_foundational::FoundationalBoundaryReportSurface::new(
                    vec!["row-x", "row-y"],
                    2,
                )
                .expect("report"),
            ),
            worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("report plan"),
    )
    .expect("report legality")
    .with_receipt(
        plan_descriptive_boundary_materialization(
            claim_receipt_evidence_boundary_surface(
                worth_foundational::FoundationalBoundaryReceiptSurface::new("exchange stored", 1)
                    .expect("receipt"),
            ),
            worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("receipt plan"),
    )
    .expect("receipt legality")
    .materialize()
    .expect("bundle materialized");

    let left_ready = match prepare_materialized_boundary_bundle_for_canonical_basis(
        version("m4.phase4.bundle"),
        &first_bundle,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready bundle basis"),
    };
    let right_ready = match prepare_materialized_boundary_bundle_for_canonical_basis(
        version("m4.phase4.bundle"),
        &second_bundle,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready bundle basis"),
    };

    assert_eq!(
        left_ready.payload().domain(),
        worth_foundational::CanonicalBasisDomain::BoundaryArtifact
    );
    match exact_compare(
        match prepare_materialized_boundary_bundle_for_canonical_basis(
            version("m4.phase4.bundle"),
            &first_bundle,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected ready bundle basis"),
        },
        right_ready,
    ) {
        worth_foundational::CanonicalComparisonOutcome::Equivalent(equivalent) => {
            assert_eq!(
                equivalent.domain(),
                worth_foundational::CanonicalBasisDomain::BoundaryArtifact
            );
        }
        outcome => panic!("same bundle semantics should compare equivalent, got {outcome:?}"),
    }

    let entries = foundational_boundary_canonical_basis_entries(&left_ready);
    assert_eq!(
        named_entry(entries, "shape").value(),
        &worth_foundational::CanonicalBasisValue::ExactText("coordinated-bundle".into())
    );
    assert_eq!(
        named_entry(entries, "bundle.source").value(),
        &worth_foundational::CanonicalBasisValue::ExactText("compatibility-lowered".into())
    );
    assert_eq!(
        named_entry(entries, "member.summary.present").value(),
        &worth_foundational::CanonicalBasisValue::Bool(true)
    );
    assert_eq!(
        named_entry(entries, "member.report.present").value(),
        &worth_foundational::CanonicalBasisValue::Bool(true)
    );
    assert_eq!(
        named_entry(entries, "member.receipt.present").value(),
        &worth_foundational::CanonicalBasisValue::Bool(true)
    );
}

#[test]
fn bundle_basis_keeps_optional_member_absence_and_attachment_visibility_explicit() {
    let profile = materialized_profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::CertificationReady,
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Durable,
        worth_foundational::CertificationPostureProfile::EvidenceBacked,
    );
    let bundle = plan_artifact_boundary_bundle(
        plan_descriptive_boundary_materialization(
            claim_derived_projection_boundary_surface(
                worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![1_u8, 2, 3], 2),
            ),
            worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("primary plan"),
    )
    .with_summary(
        plan_descriptive_boundary_materialization(
            claim_derived_projection_boundary_surface(
                worth_foundational::FoundationalBoundarySummarySurface::new("summary only", 1)
                    .expect("summary"),
            ),
            worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("summary plan"),
    )
    .expect("summary legality")
    .materialize()
    .expect("bundle materialized");

    let ready = match prepare_materialized_boundary_bundle_for_canonical_basis(
        version("m4.phase4.bundle.absence"),
        &bundle,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready bundle basis"),
    };

    let entries = foundational_boundary_canonical_basis_entries(&ready);
    assert_eq!(
        named_entry(entries, "member.report.present").value(),
        &worth_foundational::CanonicalBasisValue::Bool(false)
    );
    assert_eq!(
        named_entry(entries, "member.receipt.present").value(),
        &worth_foundational::CanonicalBasisValue::Bool(false)
    );
    assert_eq!(
        named_entry(entries, "member.primary.attachment.profile-meaning").value(),
        &worth_foundational::CanonicalBasisValue::Bool(true)
    );
    assert_eq!(
        named_entry(entries, "member.primary.attachment.canonical-basis").value(),
        &worth_foundational::CanonicalBasisValue::Bool(false)
    );
}
