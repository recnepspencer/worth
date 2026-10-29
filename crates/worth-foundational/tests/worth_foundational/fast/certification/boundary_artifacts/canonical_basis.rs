use worth_foundational::{
    claim_derived_projection_boundary_surface, claim_receipt_evidence_boundary_surface,
    claim_support_only_boundary_surface, foundational_boundary_canonical_basis_entries,
    plan_artifact_boundary_bundle, plan_descriptive_boundary_materialization,
    prepare_materialized_boundary_artifact_for_canonical_basis,
    prepare_materialized_boundary_bundle_for_canonical_basis, AdmissionReadinessProfile,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisValue, CanonicalComparisonOutcome, CertificationPostureProfile,
    CompatibilityPostureProfile, DiagnosticRichnessProfile, FoundationalBoundaryArtifactSurface,
    FoundationalBoundaryMaterializationSeam, FoundationalBoundaryMaterializationSource,
    FoundationalBoundaryReceiptSurface, FoundationalBoundaryReportSurface,
    FoundationalBoundarySummarySurface, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use super::support::{
    exact_compare, materialize_projection_artifact, materialized_profile, version,
};

fn named_entry<'a>(entries: &'a [CanonicalBasisEntry], name: &str) -> &'a CanonicalBasisEntry {
    entries
        .iter()
        .find(|entry| entry.locus() == &CanonicalBasisLocus::Named(name.to_string().into()))
        .unwrap_or_else(|| panic!("missing basis entry {name}"))
}

#[test]
fn materialized_boundary_artifacts_lower_shared_semantics_not_payload_layout() {
    let profile = materialized_profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::EvidenceBacked,
    );
    let left = materialize_projection_artifact(vec![1_u8, 2, 3], 2, profile);
    let right = materialize_projection_artifact(vec![9_u8, 8, 7], 2, profile);

    let left_ready = match prepare_materialized_boundary_artifact_for_canonical_basis(
        version("m4.phase4.single"),
        &left,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready boundary basis"),
    };
    let right_ready = match prepare_materialized_boundary_artifact_for_canonical_basis(
        version("m4.phase4.single"),
        &right,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready boundary basis"),
    };

    assert_eq!(
        left_ready.payload().domain(),
        CanonicalBasisDomain::BoundaryArtifact
    );
    match exact_compare(
        match prepare_materialized_boundary_artifact_for_canonical_basis(
            version("m4.phase4.single"),
            &left,
        ) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected ready boundary basis"),
        },
        right_ready,
    ) {
        CanonicalComparisonOutcome::Equivalent(equivalent) => {
            assert_eq!(equivalent.domain(), CanonicalBasisDomain::BoundaryArtifact);
        }
        outcome => panic!("same boundary semantics should compare equivalent, got {outcome:?}"),
    }

    let entries = foundational_boundary_canonical_basis_entries(&left_ready);
    assert_eq!(
        named_entry(entries, "shape").value(),
        &CanonicalBasisValue::ExactText("single-surface".into())
    );
    assert_eq!(
        named_entry(entries, "surface.category").value(),
        &CanonicalBasisValue::ExactText("artifact".into())
    );
    assert_eq!(
        named_entry(entries, "surface.role").value(),
        &CanonicalBasisValue::ExactText("derived-projection".into())
    );
    assert_eq!(
        named_entry(entries, "surface.source").value(),
        &CanonicalBasisValue::ExactText("compatibility-lowered".into())
    );
    assert_eq!(
        named_entry(entries, "surface.seam").value(),
        &CanonicalBasisValue::ExactText("boundary-exchange".into())
    );
    assert_eq!(
        named_entry(entries, "surface.attachment.diagnostics-attachment").kind(),
        CanonicalBasisEntryKind::BoundaryAttachment
    );
    assert!(!entries
        .iter()
        .any(|entry| entry.kind() == CanonicalBasisEntryKind::Cost));
}

#[test]
fn coordinated_boundary_bundles_reuse_one_boundary_artifact_basis_domain() {
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
            FoundationalBoundarySummarySurface::new("summary alpha", 2).expect("summary"),
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
            claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
                vec![0_u8, 1, 2],
                2,
            )),
            FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("primary plan"),
    )
    .with_summary(
        plan_descriptive_boundary_materialization(
            claim_derived_projection_boundary_surface(
                FoundationalBoundarySummarySurface::new("summary beta", 2).expect("summary"),
            ),
            FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("summary plan"),
    )
    .expect("summary legality")
    .with_report(
        plan_descriptive_boundary_materialization(
            claim_support_only_boundary_surface(
                FoundationalBoundaryReportSurface::new(vec!["row-x", "row-y"], 2).expect("report"),
            ),
            FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("report plan"),
    )
    .expect("report legality")
    .with_receipt(
        plan_descriptive_boundary_materialization(
            claim_receipt_evidence_boundary_surface(
                FoundationalBoundaryReceiptSurface::new("exchange stored", 1).expect("receipt"),
            ),
            FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
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
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready bundle basis"),
    };
    let right_ready = match prepare_materialized_boundary_bundle_for_canonical_basis(
        version("m4.phase4.bundle"),
        &second_bundle,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready bundle basis"),
    };

    assert_eq!(
        left_ready.payload().domain(),
        CanonicalBasisDomain::BoundaryArtifact
    );
    match exact_compare(
        match prepare_materialized_boundary_bundle_for_canonical_basis(
            version("m4.phase4.bundle"),
            &first_bundle,
        ) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected ready bundle basis"),
        },
        right_ready,
    ) {
        CanonicalComparisonOutcome::Equivalent(equivalent) => {
            assert_eq!(equivalent.domain(), CanonicalBasisDomain::BoundaryArtifact);
        }
        outcome => panic!("same bundle semantics should compare equivalent, got {outcome:?}"),
    }

    let entries = foundational_boundary_canonical_basis_entries(&left_ready);
    assert_eq!(
        named_entry(entries, "shape").value(),
        &CanonicalBasisValue::ExactText("coordinated-bundle".into())
    );
    assert_eq!(
        named_entry(entries, "bundle.source").value(),
        &CanonicalBasisValue::ExactText("compatibility-lowered".into())
    );
    assert_eq!(
        named_entry(entries, "member.summary.present").value(),
        &CanonicalBasisValue::Bool(true)
    );
    assert_eq!(
        named_entry(entries, "member.report.present").value(),
        &CanonicalBasisValue::Bool(true)
    );
    assert_eq!(
        named_entry(entries, "member.receipt.present").value(),
        &CanonicalBasisValue::Bool(true)
    );
}

#[test]
fn bundle_basis_keeps_optional_member_absence_and_attachment_visibility_explicit() {
    let profile = materialized_profile(
        DiagnosticRichnessProfile::Standard,
        SupportPostureProfile::CertificationReady,
        CompatibilityPostureProfile::CompatibilityLowered,
        AdmissionReadinessProfile::Admitted,
        RetentionDeliveryProfile::Durable,
        CertificationPostureProfile::EvidenceBacked,
    );
    let bundle = plan_artifact_boundary_bundle(
        plan_descriptive_boundary_materialization(
            claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
                vec![1_u8, 2, 3],
                2,
            )),
            FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("primary plan"),
    )
    .with_summary(
        plan_descriptive_boundary_materialization(
            claim_derived_projection_boundary_surface(
                FoundationalBoundarySummarySurface::new("summary only", 1).expect("summary"),
            ),
            FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
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
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected ready bundle basis"),
    };

    let entries = foundational_boundary_canonical_basis_entries(&ready);
    assert_eq!(
        named_entry(entries, "member.report.present").value(),
        &CanonicalBasisValue::Bool(false)
    );
    assert_eq!(
        named_entry(entries, "member.receipt.present").value(),
        &CanonicalBasisValue::Bool(false)
    );
    assert_eq!(
        named_entry(entries, "member.primary.attachment.profile-meaning").value(),
        &CanonicalBasisValue::Bool(true)
    );
    assert_eq!(
        named_entry(entries, "member.primary.attachment.canonical-basis").value(),
        &CanonicalBasisValue::Bool(false)
    );
}
