pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
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

pub(crate) fn artifact_locator(id: u64) -> BoundaryArtifactLocator {
    worth_foundational::BoundaryArtifactLocator::new(
        worth_foundational::BoundaryArtifactId::new(id),
        worth_foundational::BoundaryArtifactField::Basis,
    )
}

pub(crate) fn transition_locator(id: u64) -> FoundationalTransitionLocator {
    worth_foundational::FoundationalTransitionLocator::CommitParentage(
        worth_foundational::FoundationalCommitParentageLocator::new(
            worth_foundational::FoundationalCommitId::new(worth_foundational::BoundaryHandle::new(
                id,
            )),
            worth_foundational::FoundationalCommitParentBasis::new(
                worth_foundational::EquivalenceBasisId::new(id + 100),
            ),
        ),
    )
}

pub(crate) fn provenance() -> worth_foundational::FoundationalBoundaryEvidenceProvenanceArtifact {
    boundary_evidence()
        .provenance()
        .historical(
            worth_foundational::FoundationalBoundaryEvidenceSourceBasis::boundary_artifact(
                artifact_locator(1),
            ),
        )
        .with_freshness(
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        )
        .expect_success("historical provenance")
}

pub(crate) fn receipt() -> worth_foundational::FoundationalBoundaryEvidenceExecutedReceiptArtifact {
    boundary_evidence()
        .receipt()
        .execution(
            worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary::transition(
                transition_locator(2),
            ),
        )
        .with_provenance(provenance())
}

pub(crate) fn attested() -> worth_foundational::FoundationalBoundaryEvidenceAttestedLineageArtifact
{
    boundary_evidence()
        .lineage()
        .continuity(
            worth_foundational::FoundationalBoundaryEvidenceLineageSubject::new(
                worth_foundational::BoundaryHandle::new(3),
            ),
        )
        .attested_by(receipt())
}

pub(crate) fn support() -> worth_foundational::FoundationalBoundaryEvidencePublishedSupportArtifact
{
    boundary_evidence()
        .support()
        .published_evidence()
        .with_basis_disclosure(
            worth_foundational::FoundationalBoundaryEvidenceSupportBasisDisclosure::ReducedBasis,
        )
        .attested_by(
            boundary_evidence()
                .receipt()
                .support_publication(
                    worth_foundational::FoundationalBoundaryEvidenceReceiptBoundary::transition(
                        transition_locator(4),
                    ),
                )
                .with_provenance(provenance()),
        )
        .expect_success("support publication")
}

pub(crate) fn materialized_mixed_bundle_from_common_path(
    locator_id: u64,
) -> worth_foundational::FoundationalMaterializedBoundaryEvidenceAttachmentBundle {
    worth_foundational::boundary_evidence_api::common_path::attachment()
        .for_boundary_artifact(artifact_locator(locator_id))
        .with_receipt_attachment(receipt().completed_receipt().clone())
        .with_provenance_attachment(provenance())
        .with_explanation_bundle_locator(worth_foundational::FoundationalDiagnosticLocator::Transition(
            transition_locator(locator_id + 1),
        ))
        .with_published_support(support())
        .with_attested_continuity(attested())
        .materialize_under(
            worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        )
}

pub(crate) fn materialized_mixed_bundle_from_direct_path(
    locator_id: u64,
) -> worth_foundational::FoundationalMaterializedBoundaryEvidenceAttachmentBundle {
    boundary_evidence()
        .attachment()
        .for_boundary_artifact(artifact_locator(locator_id))
        .with_attested_continuity(attested())
        .with_published_support(support())
        .with_explanation_bundle_locator(worth_foundational::FoundationalDiagnosticLocator::Transition(
            transition_locator(locator_id + 1),
        ))
        .with_provenance_attachment(provenance())
        .with_receipt_attachment(receipt().completed_receipt().clone())
        .materialize_under(
            worth_foundational::FoundationalBoundaryEvidenceMaterializationProfile::FullDescriptiveRichness,
        )
}

pub(crate) trait ExpectTransitionSuccess<T> {
    fn expect_success(self, label: &str) -> T;
}

impl<T, E> ExpectTransitionSuccess<T> for TransitionOutcome<T, E> {
    fn expect_success(self, label: &str) -> T {
        match self {
            worth_proof::TransitionOutcome::Success(value) => value,
            _ => panic!("expected {label} success"),
        }
    }
}
