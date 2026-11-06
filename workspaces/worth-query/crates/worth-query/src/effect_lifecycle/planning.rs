use crate::basis_lifecycle::BasisFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::workflow::{
    QueryWorkflowDeclaration, WorkflowAuthorityTargetFamily, WorkflowBasisFamily,
    WorkflowBudgetClass, WorkflowCostClass, WorkflowFreshnessPolicy,
    WorkflowPreviewEvaluationClass,
};

use super::counters::EffectLifecycleCounters;
use super::eligibility::AdmittedEffectIntent;
use super::lowering::{
    lower_authority_scoped_effect_plan, EffectLoweringDenial, LoweredEffectExecutionPlan,
};
use super::taxonomy::{EffectAuthorityLane, EffectFamily};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectInvariantScope {
    EntityScopedMutation,
    BranchScopedMerge,
    BridgeWritebackProjection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectPreviewPosture {
    NotPreviewBound,
    ReadOnlyPreview,
    PromotionEligiblePreview,
    PreviewComparisonBound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectPolicyPosture {
    Unmasked,
    TenantScoped,
    PolicyScoped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectPermittedLoweringFamily {
    MutationIntentDeclaration,
    MergeWorkflowDeclaration,
    QueryWritebackDeclaration,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectAuthorityOwner {
    WorthRelational,
    WorthRuntimeBridge,
    WorthQuery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectArtifactPolicy {
    ReceiptFirst,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectConflictFootprint {
    EntityMutation,
    BranchMerge,
    BridgeWriteback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectStrategyIdentityTarget {
    NativeStrategyCommitRequest,
    MergeExecutionRequest,
    BridgeWritebackDeclaration,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuthorityScopedEffectPlan {
    admitted: AdmittedEffectIntent,
    invariant_scope: EffectInvariantScope,
    preview_posture: EffectPreviewPosture,
    policy_posture: EffectPolicyPosture,
    permitted_lowering_family: EffectPermittedLoweringFamily,
    artifact_policy: EffectArtifactPolicy,
    conflict_footprint: EffectConflictFootprint,
    plan_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl AuthorityScopedEffectPlan {
    pub(crate) fn new(admitted: AdmittedEffectIntent) -> Result<Self, IdentityDerivationDenial> {
        let invariant_scope = invariant_scope_for(admitted.normalized().family());
        let preview_posture = preview_posture_for(admitted.workflow_declaration());
        let policy_posture = policy_posture_for(admitted.normalized().basis_family());
        let permitted_lowering_family =
            permitted_lowering_family_for(admitted.normalized().family());
        let artifact_policy = EffectArtifactPolicy::ReceiptFirst;
        let conflict_footprint = conflict_footprint_for(admitted.normalized().family());
        let counters = EffectLifecycleCounters::authority_scoped_plan(
            admitted.normalized().counters().effect_support_row_count(),
        );
        let plan_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "authority_scoped_effect_plan_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("admitted"),
                    admitted.admitted_identity(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("scope"),
                    invariant_scope.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("preview"),
                    preview_posture.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("policy"),
                    policy_posture.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("lowering"),
                    permitted_lowering_family.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("artifact"),
                    artifact_policy.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("footprint"),
                    conflict_footprint.as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("counters"),
                    &counters.evidence_identity(),
                )
                .seal()?;
        Ok(Self {
            admitted,
            invariant_scope,
            preview_posture,
            policy_posture,
            permitted_lowering_family,
            artifact_policy,
            conflict_footprint,
            plan_identity,
            counters,
        })
    }

    pub fn admitted(&self) -> &AdmittedEffectIntent {
        &self.admitted
    }

    pub fn family(&self) -> EffectFamily {
        self.admitted.normalized().family()
    }

    pub fn authority_lane(&self) -> EffectAuthorityLane {
        self.admitted.normalized().authority_lane()
    }

    pub fn authority_owner(&self) -> EffectAuthorityOwner {
        match self
            .admitted
            .workflow_declaration()
            .report()
            .authority_target_family()
        {
            WorkflowAuthorityTargetFamily::RelationalMutation
            | WorkflowAuthorityTargetFamily::RelationalMerge => {
                EffectAuthorityOwner::WorthRelational
            }
            WorkflowAuthorityTargetFamily::BridgeWriteback => {
                EffectAuthorityOwner::WorthRuntimeBridge
            }
            WorkflowAuthorityTargetFamily::QueryInspection => EffectAuthorityOwner::WorthQuery,
        }
    }

    pub fn freshness_policy(&self) -> &WorkflowFreshnessPolicy {
        self.admitted
            .workflow_declaration()
            .request()
            .freshness_policy()
    }

    pub fn execution_cost_class(&self) -> &WorkflowCostClass {
        self.admitted.workflow_declaration().request().cost_class()
    }

    pub fn budget_class(&self) -> &WorkflowBudgetClass {
        self.admitted
            .workflow_declaration()
            .request()
            .budget_class()
    }

    pub fn basis_lane(&self) -> &WorkflowBasisFamily {
        self.admitted
            .workflow_declaration()
            .binding()
            .basis_family()
    }

    pub fn invariant_scope(&self) -> EffectInvariantScope {
        self.invariant_scope
    }

    pub fn preview_posture(&self) -> EffectPreviewPosture {
        self.preview_posture
    }

    pub fn policy_posture(&self) -> EffectPolicyPosture {
        self.policy_posture
    }

    pub fn permitted_lowering_family(&self) -> EffectPermittedLoweringFamily {
        self.permitted_lowering_family
    }

    pub fn artifact_policy(&self) -> EffectArtifactPolicy {
        self.artifact_policy
    }

    pub fn conflict_footprint(&self) -> EffectConflictFootprint {
        self.conflict_footprint
    }

    pub fn strategy_identity_target(&self) -> EffectStrategyIdentityTarget {
        match self.permitted_lowering_family {
            EffectPermittedLoweringFamily::MutationIntentDeclaration => {
                EffectStrategyIdentityTarget::NativeStrategyCommitRequest
            }
            EffectPermittedLoweringFamily::MergeWorkflowDeclaration => {
                EffectStrategyIdentityTarget::MergeExecutionRequest
            }
            EffectPermittedLoweringFamily::QueryWritebackDeclaration => {
                EffectStrategyIdentityTarget::BridgeWritebackDeclaration
            }
        }
    }

    pub fn plan_for_reporting(&self) -> &str {
        self.plan_identity.as_str()
    }

    pub fn plan_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.plan_identity
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }

    pub fn lower(self) -> Result<LoweredEffectExecutionPlan, EffectLoweringDenial> {
        lower_authority_scoped_effect_plan(self)
    }
}

pub fn scope_admitted_effect_plan(admitted: AdmittedEffectIntent) -> AuthorityScopedEffectPlan {
    AuthorityScopedEffectPlan::new(admitted)
}

fn invariant_scope_for(family: EffectFamily) -> EffectInvariantScope {
    match family {
        EffectFamily::Mutation => EffectInvariantScope::EntityScopedMutation,
        EffectFamily::Merge => EffectInvariantScope::BranchScopedMerge,
        EffectFamily::Writeback => EffectInvariantScope::BridgeWritebackProjection,
    }
}

fn preview_posture_for(declaration: &QueryWorkflowDeclaration) -> EffectPreviewPosture {
    match declaration.binding().preview_evaluation_class() {
        Some(WorkflowPreviewEvaluationClass::ReadOnly) => EffectPreviewPosture::ReadOnlyPreview,
        Some(WorkflowPreviewEvaluationClass::PromotionEligible) => {
            if declaration.binding().basis_family()
                == &WorkflowBasisFamily::PreviewPromotionComparison
            {
                EffectPreviewPosture::PreviewComparisonBound
            } else {
                EffectPreviewPosture::PromotionEligiblePreview
            }
        }
        None => EffectPreviewPosture::NotPreviewBound,
    }
}

fn policy_posture_for(basis_family: BasisFamily) -> EffectPolicyPosture {
    match basis_family {
        BasisFamily::TenantScoped => EffectPolicyPosture::TenantScoped,
        BasisFamily::PolicyScoped => EffectPolicyPosture::PolicyScoped,
        _ => EffectPolicyPosture::Unmasked,
    }
}

fn permitted_lowering_family_for(family: EffectFamily) -> EffectPermittedLoweringFamily {
    match family {
        EffectFamily::Mutation => EffectPermittedLoweringFamily::MutationIntentDeclaration,
        EffectFamily::Merge => EffectPermittedLoweringFamily::MergeWorkflowDeclaration,
        EffectFamily::Writeback => EffectPermittedLoweringFamily::QueryWritebackDeclaration,
    }
}

fn conflict_footprint_for(family: EffectFamily) -> EffectConflictFootprint {
    match family {
        EffectFamily::Mutation => EffectConflictFootprint::EntityMutation,
        EffectFamily::Merge => EffectConflictFootprint::BranchMerge,
        EffectFamily::Writeback => EffectConflictFootprint::BridgeWriteback,
    }
}

macro_rules! planning_enum_as_str {
    ($name:ident { $($variant:ident => $value:literal),* $(,)? }) => {
        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $value),*
                }
            }
        }
    };
}

planning_enum_as_str!(EffectInvariantScope {
    EntityScopedMutation => "entity_scoped_mutation",
    BranchScopedMerge => "branch_scoped_merge",
    BridgeWritebackProjection => "bridge_writeback_projection",
});
planning_enum_as_str!(EffectPreviewPosture {
    NotPreviewBound => "not_preview_bound",
    ReadOnlyPreview => "read_only_preview",
    PromotionEligiblePreview => "promotion_eligible_preview",
    PreviewComparisonBound => "preview_comparison_bound",
});
planning_enum_as_str!(EffectPolicyPosture {
    Unmasked => "unmasked",
    TenantScoped => "tenant_scoped",
    PolicyScoped => "policy_scoped",
});
planning_enum_as_str!(EffectPermittedLoweringFamily {
    MutationIntentDeclaration => "mutation_intent_declaration",
    MergeWorkflowDeclaration => "merge_workflow_declaration",
    QueryWritebackDeclaration => "query_writeback_declaration",
});
planning_enum_as_str!(EffectAuthorityOwner {
    WorthRelational => "worth-relational",
    WorthRuntimeBridge => "worth-runtime-bridge",
    WorthQuery => "worth-query",
});
planning_enum_as_str!(EffectArtifactPolicy {
    ReceiptFirst => "receipt_first",
});
planning_enum_as_str!(EffectConflictFootprint {
    EntityMutation => "entity_mutation",
    BranchMerge => "branch_merge",
    BridgeWriteback => "bridge_writeback",
});
planning_enum_as_str!(EffectStrategyIdentityTarget {
    NativeStrategyCommitRequest => "native_strategy_commit_request",
    MergeExecutionRequest => "merge_execution_request",
    BridgeWritebackDeclaration => "bridge_writeback_declaration",
});
