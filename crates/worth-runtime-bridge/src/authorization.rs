mod contract;
mod denial;
mod evidence;
mod installation;
mod runtime;

pub use contract::{
    BridgeAuthorizationBindingIdentity, BridgeAuthorizationClauseContract,
    BridgeAuthorizationClauseObservation, BridgeAuthorizationCorrespondenceIdentity,
    BridgeAuthorizationDependencyCardinality, BridgeAuthorizationInstallationRequest,
    BridgeAuthorizationObservation, BridgeAuthorizationRequirementContract,
    BridgeAuthorizationRequirementObservation, BridgeAuthorizationRuleContract,
    BridgeAuthorizationRuleEffect, BridgeAuthorizationRuleObservation,
};
pub use denial::{BridgeAuthorizationDenial, BridgeAuthorizationDenialKind};
pub use evidence::{BridgeAuthorizationDecisionEvidence, BridgeAuthorizationRuleDecisionEvidence};
pub use installation::BridgeAuthorizationInstallationBatch;
pub use runtime::{BridgeAuthorizationRetentionStop, BridgeAuthorizationRuntime};

#[cfg(test)]
mod tests;
