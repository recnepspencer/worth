use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::ordinary::mutation::WorthQueryMutationDeclaration;
use crate::ordinary::workflow;
use crate::ordinary::WorthQueryOrdinaryInspectionPolicy;
use crate::session_label::WorthQuerySessionLabel;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryReadOnlyPreviewDeclaration {
    pub(crate) identity: WorthQueryEvidenceIdentity,
    pub(crate) label: WorthQuerySessionLabel,
    pub(crate) inspection_policy: WorthQueryOrdinaryInspectionPolicy,
}

impl WorthQueryReadOnlyPreviewDeclaration {
    pub fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        &self.label
    }

    pub fn inspection_policy(&self) -> WorthQueryOrdinaryInspectionPolicy {
        self.inspection_policy
    }

    pub fn with_rich_inspection(mut self) -> Self {
        self.inspection_policy = WorthQueryOrdinaryInspectionPolicy::Rich;
        self
    }

    pub fn with_mutation(
        self,
        mutation: WorthQueryMutationDeclaration,
    ) -> WorthQueryPromotionEligiblePreviewDeclaration {
        let workflow = workflow::declare(self.label, mutation);
        WorthQueryPromotionEligiblePreviewDeclaration {
            workflow: if self.inspection_policy == WorthQueryOrdinaryInspectionPolicy::Rich {
                workflow.with_rich_inspection()
            } else {
                workflow
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryPromotionEligiblePreviewDeclaration {
    pub(crate) workflow: workflow::WorthQueryWorkflowDeclaration,
}

impl WorthQueryPromotionEligiblePreviewDeclaration {
    pub fn identity(&self) -> &workflow::WorthQueryWorkflowDeclarationIdentity {
        self.workflow.identity()
    }

    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        self.workflow.session_label()
    }
}

pub fn declare(label: WorthQuerySessionLabel) -> WorthQueryReadOnlyPreviewDeclaration {
    let identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::PreviewExecutionEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "ordinary-read-only-preview-declaration",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("session_label"),
                label.identity_digest(),
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens");
    WorthQueryReadOnlyPreviewDeclaration {
        identity,
        label,
        inspection_policy: WorthQueryOrdinaryInspectionPolicy::OperationalOnly,
    }
}
