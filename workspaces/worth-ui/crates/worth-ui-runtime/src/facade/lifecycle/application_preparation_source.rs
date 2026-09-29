use super::WorthUiApplicationPreparationDenial;
use crate::capability::{CapabilitySnapshot, CapabilitySnapshotDigest};
use crate::facade::prepared_application_authority::{
    WorthUiPreparedApplicationArtifact, WorthUiPreparedDeclarationSourceIdentity,
};
use crate::runtime::expression::WorthUiAuthoredExpressionMaterial;
use crate::runtime::{WorthUiPreparedDeclarationMaterial, WorthUiSemanticHandoffEvidence};

pub(crate) struct WorthUiApplicationPreparationSource {
    canonical_artifact: WorthUiPreparedApplicationArtifact,
    authored_source_basis: crate::runtime::WorthUiAuthoredSourceBasis,
    declaration_material: WorthUiPreparedDeclarationMaterial,
    semantic_handoff: WorthUiSemanticHandoffEvidence,
    expression_material: WorthUiAuthoredExpressionMaterial,
}

impl WorthUiApplicationPreparationSource {
    pub(crate) fn runtime_service_support(&self) -> crate::capability::UiRuntimeServiceSupport {
        self.semantic_handoff.runtime_service_support()
    }

    pub(crate) fn authored_service_policy_defaults(
        &self,
    ) -> crate::declaration::UiServicePolicyDefaults {
        self.semantic_handoff.authored_service_policy_defaults()
    }

    pub(crate) fn rust_authored(
        input: &worth_ui_dsl::WorthUiRustAuthoredArtifactInput,
        snapshot: &CapabilitySnapshot,
    ) -> Result<Self, WorthUiApplicationPreparationDenial> {
        let handoff =
            crate::runtime::prepare_rust_authored_handoff(input, snapshot).map_err(|denial| {
                match denial {
                crate::runtime::WorthUiAuthoredCompositionPreparationDenial::DslCompilation(
                    report,
                ) => WorthUiApplicationPreparationDenial::DslCompilation(report),
                crate::runtime::WorthUiAuthoredCompositionPreparationDenial::RuntimePreparation(
                    denial,
                ) => WorthUiApplicationPreparationDenial::RuntimePreparation(denial),
                crate::runtime::WorthUiAuthoredCompositionPreparationDenial::Candidate(
                    denial,
                ) => WorthUiApplicationPreparationDenial::Candidate(denial),
            }
            })?;
        let authored_source_basis = crate::runtime::WorthUiAuthoredSourceBasis::rust_authored(
            input.source_revision_digest(),
            handoff.composition_basis().clone(),
        );
        let (canonical_artifact, declaration_material, semantic_handoff, expression_material) =
            handoff.into_parts();
        Ok(Self {
            canonical_artifact,
            authored_source_basis,
            declaration_material,
            semantic_handoff,
            expression_material,
        })
    }

    pub(crate) fn watched_submission(
        submission: crate::runtime::WorthUiWatchedCandidateSubmission,
        snapshot_digest: CapabilitySnapshotDigest,
    ) -> Result<Self, WorthUiApplicationPreparationDenial> {
        let candidate_snapshot_digest = submission.candidate_snapshot_digest();
        let authored_source_basis = submission.authored_source_basis();
        let handoff = submission.into_preparation_handoff();
        let (canonical_artifact, declaration_material, semantic_handoff, expression_material) =
            handoff.into_parts();
        if candidate_snapshot_digest != snapshot_digest.as_u64() {
            return Err(
                WorthUiApplicationPreparationDenial::CandidateSnapshotMismatch {
                    candidate_snapshot_digest,
                    prepared_snapshot_digest: snapshot_digest.as_u64(),
                },
            );
        }
        Ok(Self {
            canonical_artifact,
            authored_source_basis,
            declaration_material,
            semantic_handoff,
            expression_material,
        })
    }

    pub(crate) fn into_prepared_parts(
        self,
    ) -> (
        WorthUiPreparedApplicationArtifact,
        crate::runtime::WorthUiAuthoredSourceBasis,
        WorthUiPreparedDeclarationSourceIdentity,
        WorthUiSemanticHandoffEvidence,
        WorthUiAuthoredExpressionMaterial,
        Vec<crate::declaration::UiDeclarationArtifact>,
    ) {
        let Self {
            canonical_artifact,
            authored_source_basis,
            declaration_material,
            semantic_handoff,
            expression_material,
        } = self;
        let (artifacts, declaration_source_identity) = declaration_material.into_parts();
        (
            canonical_artifact,
            authored_source_basis,
            declaration_source_identity,
            semantic_handoff,
            expression_material,
            artifacts,
        )
    }
}
