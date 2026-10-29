use worth_foundational::{
    attachment, boundary_evidence,
    boundary_evidence_api::{common_path, lower_lane, stronger_lane},
    BoundaryArtifactField, BoundaryArtifactId, BoundaryArtifactLocator, BoundaryHandle,
    FoundationalBoundaryEvidenceFreshnessPosture, FoundationalBoundaryEvidenceLineageSubject,
    FoundationalBoundaryEvidenceMaterializationProfile,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalCommitId, FoundationalCommitParentBasis, FoundationalCommitParentageLocator,
    FoundationalTransitionLocator,
};
use worth_proof::TransitionOutcome;

fn artifact_locator(id: u64) -> BoundaryArtifactLocator {
    BoundaryArtifactLocator::new(BoundaryArtifactId::new(id), BoundaryArtifactField::Basis)
}

fn transition_locator(id: u64) -> FoundationalTransitionLocator {
    FoundationalTransitionLocator::CommitParentage(FoundationalCommitParentageLocator::new(
        FoundationalCommitId::new(BoundaryHandle::new(id)),
        FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(id + 10)),
    ))
}

fn provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    common_path::provenance()
        .historical(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
            artifact_locator(1),
        ))
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
        .expect_success("historical provenance")
}

fn receipt() -> worth_foundational::FoundationalBoundaryEvidenceExecutedReceiptArtifact {
    common_path::receipt()
        .execution(FoundationalBoundaryEvidenceReceiptBoundary::transition(
            transition_locator(2),
        ))
        .with_provenance(provenance())
}

fn support() -> worth_foundational::FoundationalBoundaryEvidencePublishedSupportArtifact {
    common_path::support()
        .published_evidence()
        .with_basis_disclosure(
            worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedBasis,
        )
        .attested_by(
            common_path::receipt()
                .support_publication(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                    transition_locator(3),
                ))
                .with_provenance(provenance()),
        )
        .expect_success("support publication")
}

#[test]
fn grouped_boundary_evidence_surface_exposes_common_lower_and_stronger_lanes() {
    let _common_root: worth_foundational::BoundaryEvidenceFrontDoor =
        common_path::boundary_evidence();
    let _common_attachment: worth_foundational::FoundationalBoundaryEvidenceAttachmentFrontDoor =
        common_path::attachment();
    let _direct_attachment: worth_foundational::FoundationalBoundaryEvidenceAttachmentFrontDoor =
        attachment();
    assert_eq!(
        lower_lane::primitives::foundational_boundary_evidence_category_definitions().len(),
        4
    );
    assert_eq!(
        lower_lane::provenance::foundational_boundary_evidence_provenance_layer_definitions().len(),
        7
    );
    assert_eq!(
        lower_lane::receipts::foundational_boundary_evidence_receipt_kind_definitions().len(),
        8
    );
    assert!(
        !lower_lane::lineage::foundational_boundary_evidence_lineage_outcome_kind_definitions()
            .is_empty()
    );
    assert_eq!(
        lower_lane::support::foundational_boundary_evidence_support_truth_kind_definitions().len(),
        7
    );
    assert_eq!(
        lower_lane::attachments::foundational_boundary_evidence_attachment_target_kind_definitions(
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
                .continuity(FoundationalBoundaryEvidenceLineageSubject::new(
                    BoundaryHandle::new(20),
                ))
                .attested_by(receipt()),
        )
        .with_provenance_attachment(provenance())
        .with_published_support(support())
        .materialize_under(
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        );
    let current_admitted =
        stronger_lane::readmission::admit_current_basis_boundary_evidence_attachment_bundle(
            materialized.clone(),
            stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(),
        );
    let current_bridged =
        stronger_lane::readmission::bridge_current_basis_boundary_evidence_attachment_bundle_trust_boundary(
            current_admitted,
        );
    let current_readmitted =
        stronger_lane::readmission::readmit_current_basis_boundary_evidence_attachment_bundle_after_boundary(
            current_bridged,
            stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(),
        );

    let support_admitted =
        stronger_lane::readmission::admit_support_basis_boundary_evidence_attachment_bundle(
            materialized,
            stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
        )
        .expect("support basis admission");
    let support_bridged =
        stronger_lane::readmission::bridge_support_basis_boundary_evidence_attachment_bundle_trust_boundary(
            support_admitted,
        );
    let support_readmitted =
        stronger_lane::readmission::readmit_support_basis_boundary_evidence_attachment_bundle_after_boundary(
            support_bridged,
            stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
        );

    let report =
        stronger_lane::readiness::foundational_boundary_evidence_milestone7_readiness_report();
    let certified =
        stronger_lane::readiness::certify_foundational_boundary_evidence_milestone7_production_test_readiness();

    assert_eq!(
        current_readmitted.payload().continuity_scope(),
        Some(
            worth_foundational::FoundationalBoundaryEvidenceContinuityAttachmentScope::ObjectLevel
        )
    );
    assert!(support_readmitted.payload().support().is_some());
    assert!(report.passes_readiness_checklist());
    assert!(std::ptr::eq(
        stronger_lane::readiness::require_foundational_boundary_evidence_milestone7_production_test_readiness(
            &certified
        ),
        certified.payload()
    ));
}

trait ExpectTransitionSuccess<T> {
    fn expect_success(self, label: &str) -> T;
}

impl<T, E> ExpectTransitionSuccess<T> for TransitionOutcome<T, E> {
    fn expect_success(self, label: &str) -> T {
        match self {
            TransitionOutcome::Success(value) => value,
            _ => panic!("expected {label} success"),
        }
    }
}
