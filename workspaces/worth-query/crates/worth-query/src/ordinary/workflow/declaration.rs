use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::ordinary::mutation::WorthQueryMutationDeclaration;
use crate::ordinary::WorthQueryOrdinaryInspectionPolicy;
use crate::session_label::WorthQuerySessionLabel;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowFamily {
    PreviewPromotion,
    ProjectedStateWriteback,
    BranchMerge,
}

impl WorthQueryWorkflowFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreviewPromotion => "preview-promotion",
            Self::ProjectedStateWriteback => "projected-state-writeback",
            Self::BranchMerge => "branch-merge",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowDeclarationIdentity {
    identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryWorkflowDeclarationIdentity {
    pub fn as_str(&self) -> &str {
        self.identity.as_str()
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryWorkflowDeclaration {
    identity: WorthQueryWorkflowDeclarationIdentity,
    family: WorthQueryWorkflowFamily,
    label: WorthQuerySessionLabel,
    mutation: WorthQueryMutationDeclaration,
    inspection_policy: WorthQueryOrdinaryInspectionPolicy,
}

impl WorthQueryWorkflowDeclaration {
    pub fn identity(&self) -> &WorthQueryWorkflowDeclarationIdentity {
        &self.identity
    }

    pub fn family(&self) -> WorthQueryWorkflowFamily {
        self.family
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

    #[cfg(test)]
    pub(crate) fn mutation(&self) -> &WorthQueryMutationDeclaration {
        &self.mutation
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        WorthQueryWorkflowDeclarationIdentity,
        WorthQueryWorkflowFamily,
        WorthQuerySessionLabel,
        WorthQueryMutationDeclaration,
        WorthQueryOrdinaryInspectionPolicy,
    ) {
        (
            self.identity,
            self.family,
            self.label,
            self.mutation,
            self.inspection_policy,
        )
    }
}

pub fn declare(
    label: WorthQuerySessionLabel,
    mutation: WorthQueryMutationDeclaration,
) -> WorthQueryWorkflowDeclaration {
    let family = WorthQueryWorkflowFamily::PreviewPromotion;
    let inspection_policy = mutation.inspection_policy();
    WorthQueryWorkflowDeclaration {
        identity: workflow_identity(family, &label, &mutation),
        family,
        label,
        mutation,
        inspection_policy,
    }
}

fn workflow_identity(
    family: WorthQueryWorkflowFamily,
    label: &WorthQuerySessionLabel,
    mutation: &WorthQueryMutationDeclaration,
) -> Result<WorthQueryWorkflowDeclarationIdentity, IdentityDerivationDenial> {
    Ok(WorthQueryWorkflowDeclarationIdentity {
        identity: WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::WorkflowMutationLowering,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "ordinary-workflow-declaration",
        )
        .field_shape(WorthQueryEvidenceTag::new("family"), family.as_str())
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("session_label"),
            label.identity_digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("mutation"),
            mutation.identity().evidence_identity(),
        )
        .seal()?,
    })
}
