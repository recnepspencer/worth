use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use super::runtime_binding::WorthQueryWorkflowRuntimeBindingSemantics;
use crate::workflow::{
    WorkflowAuthorityTargetFamily, WorkflowBudgetClass, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowFreshnessPolicy,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowRuntimeSemantics {
    binding: WorthQueryWorkflowRuntimeBindingSemantics,
    declaration_family: WorkflowDeclarationFamily,
    authority_target_family: WorkflowAuthorityTargetFamily,
    cost_class: WorkflowCostClass,
    budget_class: WorkflowBudgetClass,
    freshness_policy: WorkflowFreshnessPolicy,
}

impl WorthQueryWorkflowRuntimeSemantics {
    pub fn new(
        binding: WorthQueryWorkflowRuntimeBindingSemantics,
        declaration_family: WorkflowDeclarationFamily,
        authority_target_family: WorkflowAuthorityTargetFamily,
        cost_class: WorkflowCostClass,
        budget_class: WorkflowBudgetClass,
        freshness_policy: WorkflowFreshnessPolicy,
    ) -> Self {
        Self {
            binding,
            declaration_family,
            authority_target_family,
            cost_class,
            budget_class,
            freshness_policy,
        }
    }

    pub fn binding(&self) -> &WorthQueryWorkflowRuntimeBindingSemantics {
        &self.binding
    }

    pub fn declaration_family(&self) -> &WorkflowDeclarationFamily {
        &self.declaration_family
    }

    pub fn authority_target_family(&self) -> &WorkflowAuthorityTargetFamily {
        &self.authority_target_family
    }

    pub fn cost_class(&self) -> &WorkflowCostClass {
        &self.cost_class
    }

    pub fn budget_class(&self) -> &WorkflowBudgetClass {
        &self.budget_class
    }

    pub fn freshness_policy(&self) -> &WorkflowFreshnessPolicy {
        &self.freshness_policy
    }

    pub(crate) fn semantics_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(
            domain_capability_scope_encoder("worth_query_workflow_runtime_semantics_v1")
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("binding"),
                    &self.binding.semantics_identity(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("declaration_family"),
                    self.declaration_family.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("authority_target_family"),
                    self.authority_target_family.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("cost_class"),
                    self.cost_class.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("budget_class"),
                    self.budget_class.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("freshness_policy"),
                    self.freshness_policy.as_str(),
                )
                .seal()?,
        )
    }
}
