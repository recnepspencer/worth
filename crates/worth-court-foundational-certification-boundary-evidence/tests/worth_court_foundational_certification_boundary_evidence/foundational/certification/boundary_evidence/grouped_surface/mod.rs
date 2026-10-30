#[allow(unused_imports)]
use crate::certification::ordinary_context::boundary_evidence::grouped_surface::*;
use {worth_foundational::attachment, worth_foundational::boundary_evidence};

#[test]
fn grouped_boundary_evidence_surface_exposes_common_lower_and_stronger_lanes() {
    let _common_root: worth_foundational::BoundaryEvidenceFrontDoor =
        worth_foundational::boundary_evidence_api::common_path::boundary_evidence();
    let _common_attachment: worth_foundational::FoundationalBoundaryEvidenceAttachmentFrontDoor =
        worth_foundational::boundary_evidence_api::common_path::attachment();
    let _direct_attachment: worth_foundational::FoundationalBoundaryEvidenceAttachmentFrontDoor =
        attachment();
    assert_eq!(
        worth_foundational::boundary_evidence_api::lower_lane::primitives::foundational_boundary_evidence_category_definitions().len(),
        4
    );
    assert_eq!(
        worth_foundational::boundary_evidence_api::lower_lane::provenance::foundational_boundary_evidence_provenance_layer_definitions().len(),
        7
    );
    assert_eq!(
        worth_foundational::boundary_evidence_api::lower_lane::receipts::foundational_boundary_evidence_receipt_kind_definitions().len(),
        8
    );
    assert!(
        !worth_foundational::boundary_evidence_api::lower_lane::lineage::foundational_boundary_evidence_lineage_outcome_kind_definitions()
            .is_empty()
    );
    assert_eq!(
        worth_foundational::boundary_evidence_api::lower_lane::support::foundational_boundary_evidence_support_truth_kind_definitions().len(),
        7
    );
    assert_eq!(
        worth_foundational::boundary_evidence_api::lower_lane::attachments::foundational_boundary_evidence_attachment_target_kind_definitions(
        )
        .len(),
        3
    );

    let materialized = boundary_evidence()
        .attachment()
        .for_boundary_artifact(artifact_locator(20))
        .with_attested_continuity(
            boundary_evidence()
                .lineage()
                .continuity(worth_foundational::FoundationalBoundaryEvidenceLineageSubject::new(
                    worth_foundational::BoundaryHandle::new(20),
                ))
                .attested_by(receipt()),
        )
        .with_provenance_attachment(provenance())
        .with_published_support(support())
        .materialize_under(
            worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        );
    let current_admitted =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::admit_current_basis_boundary_evidence_attachment_bundle(
            materialized.clone(),
            worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(),
        );
    let current_bridged =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::bridge_current_basis_boundary_evidence_attachment_bundle_trust_boundary(
            current_admitted,
        );
    let current_readmitted =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::readmit_current_basis_boundary_evidence_attachment_bundle_after_boundary(
            current_bridged,
            worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(),
        );

    let support_admitted =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::admit_support_basis_boundary_evidence_attachment_bundle(
            materialized,
            worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
        )
        .expect("support basis admission");
    let support_bridged =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::bridge_support_basis_boundary_evidence_attachment_bundle_trust_boundary(
            support_admitted,
        );
    let support_readmitted =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::readmit_support_basis_boundary_evidence_attachment_bundle_after_boundary(
            support_bridged,
            worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
        );

    let report =
        worth_foundational::boundary_evidence_api::stronger_lane::readiness::foundational_boundary_evidence_milestone7_readiness_report();
    let certified =
        worth_foundational::boundary_evidence_api::stronger_lane::readiness::certify_foundational_boundary_evidence_milestone7_production_test_readiness();

    assert_eq!(
        current_readmitted.payload().continuity_scope(),
        Some(
            worth_foundational::FoundationalBoundaryEvidenceContinuityAttachmentScope::ObjectLevel
        )
    );
    assert!(support_readmitted.payload().support().is_some());
    assert!(report.passes_readiness_checklist());
    assert!(std::ptr::eq(
        worth_foundational::boundary_evidence_api::stronger_lane::readiness::require_foundational_boundary_evidence_milestone7_production_test_readiness(
            &certified
        ),
        certified.payload()
    ));
}
