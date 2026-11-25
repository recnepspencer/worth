use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeExecutionDecisionSurface {
    LoweredRecordDecisionOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeExecutionConsumptionRule {
    ConsumeCanonicalLoweredArtifactOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeExecutionAuthorizationRule {
    MustNotWidenBeyondAuthorizedAspectValueSurface,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeExecutionAuthorityContract {
    pub decision_surface: MergeExecutionDecisionSurface,
    pub identity_authority: MergeExecutionConsumptionRule,
    pub conflict_authority: MergeExecutionConsumptionRule,
    pub policy_authority: MergeExecutionConsumptionRule,
    pub value_authorization: MergeExecutionAuthorizationRule,
}
