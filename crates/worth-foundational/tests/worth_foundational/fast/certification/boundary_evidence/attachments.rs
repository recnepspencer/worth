use worth_foundational::{
    attachment, boundary_evidence,
    boundary_evidence_api::{common_path, lower_lane, stronger_lane},
    foundational_boundary_evidence_attachment_target_kind_definitions,
    foundational_boundary_evidence_continuity_attachment_scope_definitions,
    foundational_boundary_evidence_materialization_profile_definitions, BoundaryArtifactField,
    BoundaryArtifactId, BoundaryArtifactLocator, BoundaryHandle, CanonicalBasisDomain,
    CanonicalDigestAlgorithmId, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLineageSubject, FoundationalBoundaryEvidenceMaterializationProfile,
    FoundationalBoundaryEvidenceReceiptBoundary, FoundationalBoundaryEvidenceSourceBasis,
    FoundationalCommitId, FoundationalCommitParentBasis, FoundationalCommitParentageLocator,
    FoundationalDiagnosticLocator, FoundationalTransitionLocator,
};
use worth_proof::TransitionOutcome;
#[path = "attachments/digest_participation.rs"]
pub(crate) mod digest_participation;

fn artifact_locator(id: u64) -> BoundaryArtifactLocator {
    BoundaryArtifactLocator::new(BoundaryArtifactId::new(id), BoundaryArtifactField::Basis)
}

fn transition_locator(id: u64) -> FoundationalTransitionLocator {
    FoundationalTransitionLocator::CommitParentage(FoundationalCommitParentageLocator::new(
        FoundationalCommitId::new(BoundaryHandle::new(id)),
        FoundationalCommitParentBasis::new(worth_foundational::EquivalenceBasisId::new(id + 100)),
    ))
}

fn provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    boundary_evidence()
        .provenance()
        .historical(FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
            artifact_locator(1),
        ))
        .with_freshness(FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained)
        .expect_success("historical provenance")
}

fn receipt() -> worth_foundational::FoundationalBoundaryEvidenceExecutedReceiptArtifact {
    boundary_evidence()
        .receipt()
        .execution(FoundationalBoundaryEvidenceReceiptBoundary::transition(
            transition_locator(2),
        ))
        .with_provenance(provenance())
}

fn attested() -> worth_foundational::FoundationalBoundaryEvidenceAttestedLineageArtifact {
    boundary_evidence()
        .lineage()
        .continuity(FoundationalBoundaryEvidenceLineageSubject::new(
            BoundaryHandle::new(3),
        ))
        .attested_by(receipt())
}

fn support() -> worth_foundational::FoundationalBoundaryEvidencePublishedSupportArtifact {
    boundary_evidence()
        .support()
        .published_evidence()
        .with_basis_disclosure(
            worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedBasis,
        )
        .attested_by(
            boundary_evidence()
                .receipt()
                .support_publication(FoundationalBoundaryEvidenceReceiptBoundary::transition(
                    transition_locator(4),
                ))
                .with_provenance(provenance()),
        )
        .expect_success("support publication")
}

fn materialized_mixed_bundle_from_common_path(
    locator_id: u64,
) -> worth_foundational::FoundationalMaterializedBoundaryEvidenceAttachmentBundle {
    common_path::attachment()
        .for_boundary_artifact(artifact_locator(locator_id))
        .with_receipt_attachment(receipt().completed_receipt().clone())
        .with_provenance_attachment(provenance())
        .with_explanation_bundle_locator(FoundationalDiagnosticLocator::Transition(
            transition_locator(locator_id + 1),
        ))
        .with_published_support(support())
        .with_attested_continuity(attested())
        .materialize_under(
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        )
}

fn materialized_mixed_bundle_from_direct_path(
    locator_id: u64,
) -> worth_foundational::FoundationalMaterializedBoundaryEvidenceAttachmentBundle {
    boundary_evidence()
        .attachment()
        .for_boundary_artifact(artifact_locator(locator_id))
        .with_attested_continuity(attested())
        .with_published_support(support())
        .with_explanation_bundle_locator(FoundationalDiagnosticLocator::Transition(
            transition_locator(locator_id + 1),
        ))
        .with_provenance_attachment(provenance())
        .with_receipt_attachment(receipt().completed_receipt().clone())
        .materialize_under(
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        )
}

#[test]
fn attachment_definitions_are_blind_consumer_interpretable() {
    assert_eq!(
        foundational_boundary_evidence_attachment_target_kind_definitions().len(),
        3
    );
    assert_eq!(
        foundational_boundary_evidence_continuity_attachment_scope_definitions().len(),
        2
    );
    assert_eq!(
        foundational_boundary_evidence_materialization_profile_definitions().len(),
        3
    );
}

#[test]
fn object_and_locator_continuity_attachments_stay_distinct() {
    let object_bundle = common_path::attachment()
        .for_boundary_artifact(artifact_locator(10))
        .with_attested_continuity(attested())
        .with_provenance_attachment(provenance());
    let locator_bundle = boundary_evidence()
        .attachment()
        .for_transition(transition_locator(11))
        .with_locator_continuity(
            FoundationalBoundaryEvidenceLineageSubject::new(BoundaryHandle::new(11)),
            FoundationalDiagnosticLocator::BoundaryArtifact(artifact_locator(11)),
            FoundationalDiagnosticLocator::Transition(transition_locator(12)),
        )
        .with_provenance_attachment(provenance());

    assert_eq!(
        object_bundle.continuity_scope(),
        Some(
            worth_foundational::FoundationalBoundaryEvidenceContinuityAttachmentScope::ObjectLevel
        )
    );
    assert_eq!(
        locator_bundle.continuity_scope(),
        Some(
            worth_foundational::FoundationalBoundaryEvidenceContinuityAttachmentScope::LocatorLevel
        )
    );
}

#[test]
fn diagnostic_bundle_targets_stay_descriptive_and_context_only() {
    let materialized = boundary_evidence()
        .attachment()
        .for_diagnostic_bundle(FoundationalDiagnosticLocator::BoundaryArtifact(
            artifact_locator(14),
        ))
        .with_attested_continuity(attested())
        .with_provenance_attachment(provenance())
        .materialize_under(
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        );

    assert_eq!(
        materialized.target().target_kind(),
        worth_foundational::FoundationalBoundaryEvidenceAttachmentTargetKind::DiagnosticBundle
    );
    assert_eq!(
        materialized.continuity_scope(),
        Some(
            worth_foundational::FoundationalBoundaryEvidenceContinuityAttachmentScope::ObjectLevel
        )
    );
}

#[test]
fn materialization_profiles_elide_optional_surfaces_without_changing_target_or_continuity() {
    let bundle = boundary_evidence()
        .attachment()
        .for_boundary_artifact(artifact_locator(20))
        .with_attested_continuity(attested())
        .with_provenance_attachment(provenance())
        .with_receipt_attachment(receipt().completed_receipt().clone())
        .with_published_support(support())
        .with_support_report_locator(FoundationalDiagnosticLocator::BoundaryArtifact(
            artifact_locator(20),
        ));

    let materialized = bundle.materialize_under(
        FoundationalBoundaryEvidenceMaterializationProfile::ElideSupportAndDiagnostics,
    );

    assert_eq!(materialized.target().target_kind(), bundle.target_kind());
    assert_eq!(materialized.continuity_scope(), bundle.continuity_scope());
    assert!(materialized.support().is_none());
    assert!(materialized.diagnostic().is_none());
}

#[test]
fn canonical_basis_preparation_for_attachment_bundle_is_boundary_honest() {
    let materialized = boundary_evidence()
        .attachment()
        .for_boundary_artifact(artifact_locator(30))
        .with_attested_continuity(attested())
        .with_provenance_attachment(provenance())
        .materialize_under(
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        );

    let basis =
        worth_foundational::prepare_boundary_evidence_attachment_bundle_for_canonical_basis(
            worth_foundational::CanonicalizationRuleVersion::new("m7.phase6.attachment")
                .expect("version"),
            &materialized,
        )
        .expect_success("attachment basis");

    assert_eq!(
        basis.payload().domain(),
        CanonicalBasisDomain::BoundaryArtifact
    );
    assert!(basis.payload().entries().iter().any(
        |entry| entry.kind() == worth_foundational::CanonicalBasisEntryKind::BoundaryAttachment
    ));
}

#[test]
fn stronger_lane_readmission_for_attached_bundle_requires_boundary_bridge() {
    let materialized = boundary_evidence()
        .attachment()
        .for_boundary_artifact(artifact_locator(40))
        .with_attested_continuity(attested())
        .with_provenance_attachment(provenance())
        .materialize_under(
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        );

    let admitted = stronger_lane::readmission::admit_current_basis_boundary_evidence_attachment_bundle(
        materialized,
        stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(),
    );
    let bridged =
        stronger_lane::readmission::bridge_current_basis_boundary_evidence_attachment_bundle_trust_boundary(
            admitted,
        );
    let readmitted =
        stronger_lane::readmission::readmit_current_basis_boundary_evidence_attachment_bundle_after_boundary(
            bridged,
            stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(),
        );

    assert_eq!(
        readmitted.payload().continuity_scope(),
        Some(
            worth_foundational::FoundationalBoundaryEvidenceContinuityAttachmentScope::ObjectLevel
        )
    );
}

#[test]
fn stronger_lane_support_readmission_requires_support_attachment_and_boundary_bridge() {
    let without_support = boundary_evidence()
        .attachment()
        .for_boundary_artifact(artifact_locator(41))
        .with_attested_continuity(attested())
        .with_provenance_attachment(provenance())
        .materialize_under(
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        );

    let denial = match stronger_lane::readmission::admit_support_basis_boundary_evidence_attachment_bundle(
        without_support,
        stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
    ) {
        Ok(_) => panic!("expected support basis denial"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial,
        worth_foundational::FoundationalBoundaryEvidenceSupportReadmissionDenial::SupportAttachmentRequired
    );

    let with_support = boundary_evidence()
        .attachment()
        .for_boundary_artifact(artifact_locator(42))
        .with_attested_continuity(attested())
        .with_provenance_attachment(provenance())
        .with_published_support(support())
        .materialize_under(
            FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        );

    let admitted = stronger_lane::readmission::admit_support_basis_boundary_evidence_attachment_bundle(
        with_support,
        stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
    )
    .expect("support basis admission");
    let bridged =
        stronger_lane::readmission::bridge_support_basis_boundary_evidence_attachment_bundle_trust_boundary(
            admitted,
        );
    let readmitted =
        stronger_lane::readmission::readmit_support_basis_boundary_evidence_attachment_bundle_after_boundary(
            bridged,
            stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
        );

    assert!(readmitted.payload().support().is_some());
}

#[test]
fn common_lower_and_stronger_lanes_expose_the_same_phase6_surface() {
    assert_eq!(
        boundary_evidence().attachment_target_kind_definitions(),
        lower_lane::attachments::foundational_boundary_evidence_attachment_target_kind_definitions(
        )
    );
    let _front_door: worth_foundational::FoundationalBoundaryEvidenceAttachmentFrontDoor =
        attachment();
    let _common_front_door: worth_foundational::FoundationalBoundaryEvidenceAttachmentFrontDoor =
        common_path::attachment();
    let _authority =
        stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(
        );
    let _support_authority =
        stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority();
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
