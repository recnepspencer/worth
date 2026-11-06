use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::ordinary::WorthQueryOrdinaryInspectionPolicy;

use super::{WorthQueryWritebackContext, WorthQueryWritebackRequest};
use crate::ordinary::workflow::WorthQueryWorkflowFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWritebackTrigger {
    ProjectedStateDiff,
}

impl WorthQueryWritebackTrigger {
    pub fn family(&self) -> WorthQueryWorkflowFamily {
        match self {
            Self::ProjectedStateDiff => WorthQueryWorkflowFamily::ProjectedStateWriteback,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWritebackDeclarationIdentity {
    identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryWritebackDeclarationIdentity {
    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub fn as_str(&self) -> &str {
        self.identity.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWritebackDeclaration {
    identity: WorthQueryWritebackDeclarationIdentity,
    trigger: WorthQueryWritebackTrigger,
    inspection_policy: WorthQueryOrdinaryInspectionPolicy,
}

impl WorthQueryWritebackDeclaration {
    pub fn identity(&self) -> &WorthQueryWritebackDeclarationIdentity {
        &self.identity
    }

    pub fn trigger(&self) -> WorthQueryWritebackTrigger {
        self.trigger
    }

    pub fn family(&self) -> WorthQueryWorkflowFamily {
        self.trigger.family()
    }

    pub fn with_rich_inspection(mut self) -> Self {
        self.inspection_policy = WorthQueryOrdinaryInspectionPolicy::Rich;
        self
    }

    pub fn using(self, context: WorthQueryWritebackContext) -> WorthQueryWritebackRequest {
        WorthQueryWritebackRequest {
            declaration: self,
            context,
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        WorthQueryWritebackDeclarationIdentity,
        WorthQueryWritebackTrigger,
        WorthQueryOrdinaryInspectionPolicy,
    ) {
        (self.identity, self.trigger, self.inspection_policy)
    }
}

pub fn projected_state_diff() -> WorthQueryWritebackTrigger {
    WorthQueryWritebackTrigger::ProjectedStateDiff
}

pub fn declare_writeback(
    trigger: WorthQueryWritebackTrigger,
) -> Result<WorthQueryWritebackDeclaration, IdentityDerivationDenial> {
    let identity = WorthQueryWritebackDeclarationIdentity {
        identity: WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::WorkflowMutationLowering,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "ordinary-writeback-declaration",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("family"),
            trigger.family().as_str(),
        )
        .seal()?,
    };
    Ok(WorthQueryWritebackDeclaration {
        identity,
        trigger,
        inspection_policy: WorthQueryOrdinaryInspectionPolicy::OperationalOnly,
    })
}
