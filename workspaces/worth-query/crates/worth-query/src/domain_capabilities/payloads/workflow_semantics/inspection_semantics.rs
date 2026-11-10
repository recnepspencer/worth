use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};
use worth_relational::facade::merge::RelationalMergeInspectionArtifact;

use crate::workflow::{LoweredMergeWorkflowDeclaration, QueryWritebackDeclaration};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowInspectionSemantics {
    MergeConflict {
        lowered_merge: LoweredMergeWorkflowDeclaration,
        relational_inspection: RelationalMergeInspectionArtifact,
    },
    PostMergeFromMerge {
        lowered_merge: LoweredMergeWorkflowDeclaration,
    },
    PostMergeFromWriteback {
        lowered_writeback: QueryWritebackDeclaration,
    },
}

impl WorthQueryWorkflowInspectionSemantics {
    pub fn merge_conflict(
        lowered_merge: LoweredMergeWorkflowDeclaration,
        relational_inspection: RelationalMergeInspectionArtifact,
    ) -> Self {
        Self::MergeConflict {
            lowered_merge,
            relational_inspection,
        }
    }

    pub fn post_merge_from_merge(lowered_merge: LoweredMergeWorkflowDeclaration) -> Self {
        Self::PostMergeFromMerge { lowered_merge }
    }

    pub fn post_merge_from_writeback(lowered_writeback: QueryWritebackDeclaration) -> Self {
        Self::PostMergeFromWriteback { lowered_writeback }
    }

    pub fn lowered_merge_conflict(
        &self,
    ) -> Option<(
        &LoweredMergeWorkflowDeclaration,
        &RelationalMergeInspectionArtifact,
    )> {
        match self {
            Self::MergeConflict {
                lowered_merge,
                relational_inspection,
            } => Some((lowered_merge, relational_inspection)),
            _ => None,
        }
    }

    pub fn post_merge_from_merge_input(&self) -> Option<&LoweredMergeWorkflowDeclaration> {
        match self {
            Self::PostMergeFromMerge { lowered_merge } => Some(lowered_merge),
            _ => None,
        }
    }

    pub fn post_merge_from_writeback_input(&self) -> Option<&QueryWritebackDeclaration> {
        match self {
            Self::PostMergeFromWriteback { lowered_writeback } => Some(lowered_writeback),
            _ => None,
        }
    }

    pub(crate) fn semantics_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(match self {
            Self::MergeConflict {
                lowered_merge,
                relational_inspection,
            } => domain_capability_scope_encoder("worth_query_workflow_inspection_v1")
                .field_shape(WorthQueryEvidenceTag::new("kind"), "merge_conflict")
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("lowered_merge"),
                    lowered_merge.lowering_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("relational_inspection"),
                    &workflow_external_relational_inspection_reference_identity(
                        relational_inspection.artifact_digest(),
                    )?,
                )
                .seal()?,
            Self::PostMergeFromMerge { lowered_merge } => {
                domain_capability_scope_encoder("worth_query_workflow_inspection_v1")
                    .field_shape(WorthQueryEvidenceTag::new("kind"), "post_merge_from_merge")
                    .field_evidence_identity(
                        WorthQueryEvidenceTag::new("lowered_merge"),
                        lowered_merge.lowering_identity(),
                    )
                    .seal()?
            }
            Self::PostMergeFromWriteback { lowered_writeback } => {
                domain_capability_scope_encoder("worth_query_workflow_inspection_v1")
                    .field_shape(
                        WorthQueryEvidenceTag::new("kind"),
                        "post_merge_from_writeback",
                    )
                    .field_evidence_identity(
                        WorthQueryEvidenceTag::new("lowered_writeback"),
                        lowered_writeback.lowering_identity(),
                    )
                    .seal()?
            }
        })
    }
}

fn workflow_external_relational_inspection_reference_identity(
    external_artifact_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    domain_capability_scope_encoder(
        "worth_query_workflow_external_relational_inspection_reference_v1",
    )
    .field_shape(
        WorthQueryEvidenceTag::new("external_artifact_digest"),
        external_artifact_digest,
    )
    .seal()
}
