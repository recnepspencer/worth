#[allow(unused_imports)]
use crate::certification::ordinary_context::boundary_evidence::attachments::*;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::attachment, worth_foundational::boundary_evidence,
    worth_foundational::boundary_evidence_api::common_path,
    worth_foundational::boundary_evidence_api::lower_lane,
    worth_foundational::boundary_evidence_api::stronger_lane,
    worth_foundational::foundational_boundary_evidence_attachment_target_kind_definitions,
    worth_foundational::foundational_boundary_evidence_continuity_attachment_scope_definitions,
    worth_foundational::foundational_boundary_evidence_materialization_profile_definitions,
    worth_foundational::BoundaryArtifactField, worth_foundational::BoundaryArtifactId,
    worth_foundational::BoundaryArtifactLocator, worth_foundational::BoundaryHandle,
    worth_foundational::CanonicalBasisDomain, worth_foundational::CanonicalDigestAlgorithmId,
    worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture,
    worth_foundational::FoundationalBoundaryEvidenceLineageSubject,
    worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile,
    worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary,
    worth_foundational::FoundationalBoundaryEvidenceSourceBasis,
    worth_foundational::FoundationalCommitId, worth_foundational::FoundationalCommitParentBasis,
    worth_foundational::FoundationalCommitParentageLocator,
    worth_foundational::FoundationalDiagnosticLocator,
    worth_foundational::FoundationalTransitionLocator,
};

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
    let object_bundle = worth_foundational::boundary_evidence_api::common_path::attachment()
        .for_boundary_artifact(artifact_locator(10))
        .with_attested_continuity(attested())
        .with_provenance_attachment(provenance());
    let locator_bundle = boundary_evidence()
        .attachment()
        .for_transition(transition_locator(11))
        .with_locator_continuity(
            worth_foundational::FoundationalBoundaryEvidenceLineageSubject::new(
                worth_foundational::BoundaryHandle::new(11),
            ),
            worth_foundational::FoundationalDiagnosticLocator::BoundaryArtifact(artifact_locator(
                11,
            )),
            worth_foundational::FoundationalDiagnosticLocator::Transition(transition_locator(12)),
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
        .for_diagnostic_bundle(worth_foundational::FoundationalDiagnosticLocator::BoundaryArtifact(
            artifact_locator(14),
        ))
        .with_attested_continuity(attested())
        .with_provenance_attachment(provenance())
        .materialize_under(
            worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
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
        .with_support_report_locator(
            worth_foundational::FoundationalDiagnosticLocator::BoundaryArtifact(artifact_locator(
                20,
            )),
        );

    let materialized = bundle.materialize_under(
        worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile::ElideSupportAndDiagnostics,
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
            worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
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
        worth_foundational::CanonicalBasisDomain::BoundaryArtifact
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
            worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        );

    let admitted = worth_foundational::boundary_evidence_api::stronger_lane::readmission::admit_current_basis_boundary_evidence_attachment_bundle(
        materialized,
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(),
    );
    let bridged =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::bridge_current_basis_boundary_evidence_attachment_bundle_trust_boundary(
            admitted,
        );
    let readmitted =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::readmit_current_basis_boundary_evidence_attachment_bundle_after_boundary(
            bridged,
            worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(),
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
            worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        );

    let denial = match worth_foundational::boundary_evidence_api::stronger_lane::readmission::admit_support_basis_boundary_evidence_attachment_bundle(
        without_support,
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
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
            worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        );

    let admitted = worth_foundational::boundary_evidence_api::stronger_lane::readmission::admit_support_basis_boundary_evidence_attachment_bundle(
        with_support,
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
    )
    .expect("support basis admission");
    let bridged =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::bridge_support_basis_boundary_evidence_attachment_bundle_trust_boundary(
            admitted,
        );
    let readmitted =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::readmit_support_basis_boundary_evidence_attachment_bundle_after_boundary(
            bridged,
            worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority(),
        );

    assert!(readmitted.payload().support().is_some());
}

#[test]
fn common_lower_and_stronger_lanes_expose_the_same_surface() {
    assert_eq!(
        boundary_evidence().attachment_target_kind_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::attachments::foundational_boundary_evidence_attachment_target_kind_definitions(
        )
    );
    let _front_door: worth_foundational::FoundationalBoundaryEvidenceAttachmentFrontDoor =
        attachment();
    let _common_front_door: worth_foundational::FoundationalBoundaryEvidenceAttachmentFrontDoor =
        worth_foundational::boundary_evidence_api::common_path::attachment();
    let _authority =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_attachment_readmission_authority(
        );
    let _support_authority =
        worth_foundational::boundary_evidence_api::stronger_lane::readmission::foundational_boundary_evidence_support_readmission_authority();
}
#[path = "digest_participation/mod.rs"]
pub(crate) mod digest_participation;
