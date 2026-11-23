use serde::{Deserialize, Serialize};

use crate::merge::data::policy::{
    AspectComparisonState, AspectMergePolicyKind, AuthorizedAspectValueSurface,
    LoweredAspectAction, LoweredAspectDenialIntent, LoweredAspectExecutionIntent,
    LoweredMergeBlockedReason, LoweredMergeRejectedReason, MergeExecutionReadiness,
    MergePolicyDecisionBoundary, MergePolicyOwnershipClass,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergePolicyAspectDigestRow {
    pub aspect_key: worth_foundational::facade::AspectKey,
    pub comparison: AspectComparisonState,
    pub applied_policy: Option<AspectMergePolicyKind>,
    pub policy_ownership: Option<MergePolicyOwnershipClass>,
    pub decision_boundary: MergePolicyDecisionBoundary,
    pub resolved_value_strategy:
        Option<crate::merge::data::resolved_value_strategy::MergeResolvedAspectValueStrategy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeLoweredAspectDigestRow {
    pub aspect_key: worth_foundational::facade::AspectKey,
    pub readiness: MergeExecutionReadiness,
    pub lowered_action: Option<LoweredAspectAction>,
    pub authorized_values: Option<AuthorizedAspectValueSurface>,
    pub execution_intent: Option<LoweredAspectExecutionIntent>,
    pub resolved_value_strategy:
        Option<crate::merge::data::resolved_value_strategy::MergeResolvedAspectValueStrategy>,
    pub denial_intent: Option<LoweredAspectDenialIntent>,
    pub blocked_reason: Option<LoweredMergeBlockedReason>,
    pub rejected_reason: Option<LoweredMergeRejectedReason>,
}
