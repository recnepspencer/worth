use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::workflow::{
    lower_merge_workflow_declaration, lower_mutation_intent_declaration,
    lower_query_writeback_declaration, LoweredMergeWorkflowDeclaration,
    LoweredMutationIntentDeclaration, QueryWritebackDeclaration, WorkflowBasisFamily,
    WorkflowLoweringCounters, WorkflowLoweringError, WorkflowLoweringFailureClass,
    WorkflowStalenessClass,
};
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::counters::EffectLifecycleCounters;
use super::eligibility::AdmittedEffectIntent;
use super::normalized::EffectOperationInput;
use super::planning::{
    scope_admitted_effect_plan, AuthorityScopedEffectPlan, EffectArtifactPolicy,
    EffectAuthorityOwner, EffectConflictFootprint, EffectInvariantScope,
    EffectPermittedLoweringFamily, EffectPolicyPosture, EffectPreviewPosture,
    EffectStrategyIdentityTarget,
};
use super::taxonomy::{EffectAuthorityLane, EffectFamily};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectLoweringDenialKind {
    InvalidWorkflowDeclarationFamily,
    UnsupportedMergeFamily,
    UnsupportedRelationalStrategyTarget,
    UnsupportedWritebackFamily,
    InvalidMergeBranchPairing,
    UnsupportedWritebackCausality,
    StaleWorkflowDenied,
    ExplicitRebindRequired,
    LoweringSerializationFailed,
}

impl EffectLoweringDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidWorkflowDeclarationFamily => "invalid_workflow_declaration_family",
            Self::UnsupportedMergeFamily => "unsupported_merge_family",
            Self::UnsupportedRelationalStrategyTarget => "unsupported_relational_strategy_target",
            Self::UnsupportedWritebackFamily => "unsupported_writeback_family",
            Self::InvalidMergeBranchPairing => "invalid_merge_branch_pairing",
            Self::UnsupportedWritebackCausality => "unsupported_writeback_causality",
            Self::StaleWorkflowDenied => "stale_workflow_denied",
            Self::ExplicitRebindRequired => "explicit_rebind_required",
            Self::LoweringSerializationFailed => "lowering_serialization_failed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLoweringDenial {
    denial_kind: EffectLoweringDenialKind,
    message: &'static str,
    staleness_class: WorkflowStalenessClass,
    authority_scoped_plan_identity: WorthQueryEvidenceIdentity,
    denial_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl EffectLoweringDenial {
    fn from_workflow_error(plan: &AuthorityScopedEffectPlan, error: WorkflowLoweringError) -> Self {
        Self::from_workflow_error_for_batch(
            plan.plan_identity(),
            plan.counters().effect_support_row_count(),
            error,
        )
    }

    pub(crate) fn from_workflow_error_for_batch(
        execution_subject_identity: &WorthQueryEvidenceIdentity,
        effect_support_row_count: usize,
        error: WorkflowLoweringError,
    ) -> Result<Self, IdentityDerivationDenial> {
        let denial_kind = lowering_denial_kind(error.failure_class());
        let denial_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_lowering_denial_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("plan"),
                    execution_subject_identity,
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), denial_kind.as_str())
                .field_shape(
                    WorthQueryEvidenceTag::new("staleness"),
                    error.staleness_class().as_str(),
                )
                .field_shape(WorthQueryEvidenceTag::new("message"), error.message())
                .seal()?;
        Ok(Self {
            denial_kind,
            message: error.message(),
            staleness_class: error.staleness_class().clone(),
            authority_scoped_plan_identity: execution_subject_identity.clone(),
            denial_identity,
            counters: EffectLifecycleCounters::lowering_denied(
                effect_support_row_count,
                error.counters().workflow_lowering_width(),
            ),
        })
    }

    pub fn denial_kind(&self) -> EffectLoweringDenialKind {
        self.denial_kind
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    pub fn staleness_class(&self) -> &WorkflowStalenessClass {
        &self.staleness_class
    }

    pub fn authority_scoped_plan_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.authority_scoped_plan_identity
    }

    pub fn authority_scoped_plan_for_reporting(&self) -> &str {
        self.authority_scoped_plan_identity.as_str()
    }

    pub fn denial_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.denial_identity
    }

    pub fn denial_for_reporting(&self) -> &str {
        self.denial_identity.as_str()
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoweredEffectExecutionArtifact {
    Mutation(LoweredMutationIntentDeclaration),
    Merge(LoweredMergeWorkflowDeclaration),
    Writeback(QueryWritebackDeclaration),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoweredEffectExecutionPlan {
    authority_scoped_plan: AuthorityScopedEffectPlan,
    artifact: LoweredEffectExecutionArtifact,
    lowered_effect_execution_plan_identity: WorthQueryEvidenceIdentity,
    counters: EffectLifecycleCounters,
}

impl LoweredEffectExecutionPlan {
    fn new(
        authority_scoped_plan: AuthorityScopedEffectPlan,
        artifact: LoweredEffectExecutionArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        let lowered_effect_execution_plan_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "lowered_effect_execution_plan_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("plan"),
                    authority_scoped_plan.plan_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("artifact"),
                    artifact_identity(&artifact),
                )
                .seal()?;
        let counters = EffectLifecycleCounters::lowered(
            authority_scoped_plan.counters().effect_support_row_count(),
            artifact_counters(&artifact).workflow_lowering_width(),
            artifact_counters(&artifact).workflow_executor_rediscovery_count(),
        );
        Ok(Self {
            authority_scoped_plan,
            artifact,
            lowered_effect_execution_plan_identity,
            counters,
        })
    }

    pub fn authority_scoped_plan(&self) -> &AuthorityScopedEffectPlan {
        &self.authority_scoped_plan
    }

    pub fn family(&self) -> EffectFamily {
        self.authority_scoped_plan.family()
    }

    pub fn authority_lane(&self) -> EffectAuthorityLane {
        self.authority_scoped_plan.authority_lane()
    }

    pub fn authority_owner(&self) -> EffectAuthorityOwner {
        self.authority_scoped_plan.authority_owner()
    }

    pub fn basis_lane(&self) -> &WorkflowBasisFamily {
        self.authority_scoped_plan.basis_lane()
    }

    pub fn invariant_scope(&self) -> EffectInvariantScope {
        self.authority_scoped_plan.invariant_scope()
    }

    pub fn preview_posture(&self) -> EffectPreviewPosture {
        self.authority_scoped_plan.preview_posture()
    }

    pub fn policy_posture(&self) -> EffectPolicyPosture {
        self.authority_scoped_plan.policy_posture()
    }

    pub fn permitted_lowering_family(&self) -> EffectPermittedLoweringFamily {
        self.authority_scoped_plan.permitted_lowering_family()
    }

    pub fn artifact_policy(&self) -> EffectArtifactPolicy {
        self.authority_scoped_plan.artifact_policy()
    }

    pub fn conflict_footprint(&self) -> EffectConflictFootprint {
        self.authority_scoped_plan.conflict_footprint()
    }

    pub fn strategy_identity_target(&self) -> EffectStrategyIdentityTarget {
        self.authority_scoped_plan.strategy_identity_target()
    }

    pub fn staleness_class(&self) -> &WorkflowStalenessClass {
        match &self.artifact {
            LoweredEffectExecutionArtifact::Mutation(declaration) => declaration.staleness_class(),
            LoweredEffectExecutionArtifact::Merge(declaration) => declaration.staleness_class(),
            LoweredEffectExecutionArtifact::Writeback(declaration) => declaration.staleness_class(),
        }
    }

    pub fn artifact(&self) -> &LoweredEffectExecutionArtifact {
        &self.artifact
    }

    pub fn as_mutation(&self) -> Option<&LoweredMutationIntentDeclaration> {
        match &self.artifact {
            LoweredEffectExecutionArtifact::Mutation(declaration) => Some(declaration),
            _ => None,
        }
    }

    pub fn as_merge(&self) -> Option<&LoweredMergeWorkflowDeclaration> {
        match &self.artifact {
            LoweredEffectExecutionArtifact::Merge(declaration) => Some(declaration),
            _ => None,
        }
    }

    pub fn as_writeback(&self) -> Option<&QueryWritebackDeclaration> {
        match &self.artifact {
            LoweredEffectExecutionArtifact::Writeback(declaration) => Some(declaration),
            _ => None,
        }
    }

    pub fn lowered_effect_execution_plan_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lowered_effect_execution_plan_identity
    }

    pub fn lowered_effect_execution_plan_for_reporting(&self) -> &str {
        self.lowered_effect_execution_plan_identity.as_str()
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

pub fn lower_authority_scoped_effect_plan(
    authority_scoped_plan: AuthorityScopedEffectPlan,
) -> Result<LoweredEffectExecutionPlan, EffectLoweringDenial> {
    let artifact = match authority_scoped_plan
        .admitted()
        .normalized()
        .operation_input()
    {
        EffectOperationInput::Mutation(input) => lower_mutation_intent_declaration(
            authority_scoped_plan.admitted().workflow_declaration(),
            authority_scoped_plan
                .admitted()
                .normalized()
                .expected_lower_runtime_binding_identity()
                .expect("admitted mutation effects must preserve a lower-runtime binding identity"),
            input.clone(),
        )
        .map(LoweredEffectExecutionArtifact::Mutation),
        EffectOperationInput::Merge(input) => lower_merge_workflow_declaration(
            authority_scoped_plan.admitted().workflow_declaration(),
            input.clone(),
        )
        .map(LoweredEffectExecutionArtifact::Merge),
        EffectOperationInput::Writeback(input) => lower_query_writeback_declaration(
            authority_scoped_plan.admitted().workflow_declaration(),
            input.clone(),
        )
        .map(LoweredEffectExecutionArtifact::Writeback),
    }
    .map_err(|error| EffectLoweringDenial::from_workflow_error(&authority_scoped_plan, error))?;

    Ok(LoweredEffectExecutionPlan::new(
        authority_scoped_plan,
        artifact,
    ))
}

pub(crate) fn assemble_lowered_batch_mutation_component(
    admitted: AdmittedEffectIntent,
    declaration: LoweredMutationIntentDeclaration,
) -> LoweredEffectExecutionPlan {
    LoweredEffectExecutionPlan::new(
        scope_admitted_effect_plan(admitted),
        LoweredEffectExecutionArtifact::Mutation(declaration),
    )
}

fn lowering_denial_kind(failure_class: &WorkflowLoweringFailureClass) -> EffectLoweringDenialKind {
    match failure_class {
        WorkflowLoweringFailureClass::InvalidWorkflowDeclarationFamily => {
            EffectLoweringDenialKind::InvalidWorkflowDeclarationFamily
        }
        WorkflowLoweringFailureClass::UnsupportedMergeFamily => {
            EffectLoweringDenialKind::UnsupportedMergeFamily
        }
        WorkflowLoweringFailureClass::UnsupportedRelationalStrategyTarget => {
            EffectLoweringDenialKind::UnsupportedRelationalStrategyTarget
        }
        WorkflowLoweringFailureClass::UnsupportedWritebackFamily => {
            EffectLoweringDenialKind::UnsupportedWritebackFamily
        }
        WorkflowLoweringFailureClass::InvalidMergeBranchPairing => {
            EffectLoweringDenialKind::InvalidMergeBranchPairing
        }
        WorkflowLoweringFailureClass::UnsupportedWritebackCausality => {
            EffectLoweringDenialKind::UnsupportedWritebackCausality
        }
        WorkflowLoweringFailureClass::StaleWorkflowDenied => {
            EffectLoweringDenialKind::StaleWorkflowDenied
        }
        WorkflowLoweringFailureClass::ExplicitRebindRequired => {
            EffectLoweringDenialKind::ExplicitRebindRequired
        }
        WorkflowLoweringFailureClass::LoweringSerializationFailed => {
            EffectLoweringDenialKind::LoweringSerializationFailed
        }
    }
}

fn artifact_identity(artifact: &LoweredEffectExecutionArtifact) -> &WorthQueryEvidenceIdentity {
    match artifact {
        LoweredEffectExecutionArtifact::Mutation(declaration) => declaration.lowering_identity(),
        LoweredEffectExecutionArtifact::Merge(declaration) => declaration.lowering_identity(),
        LoweredEffectExecutionArtifact::Writeback(declaration) => declaration.lowering_identity(),
    }
}

fn artifact_counters(artifact: &LoweredEffectExecutionArtifact) -> &WorkflowLoweringCounters {
    match artifact {
        LoweredEffectExecutionArtifact::Mutation(declaration) => declaration.counters(),
        LoweredEffectExecutionArtifact::Merge(declaration) => declaration.counters(),
        LoweredEffectExecutionArtifact::Writeback(declaration) => declaration.counters(),
    }
}
