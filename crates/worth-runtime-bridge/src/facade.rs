//! Public API boundary for `worth-runtime-bridge`.
//! External crates should import through this module rather than reaching into
//! internal crate structure directly.
//!
//! This file is the authoritative bridge API surface. The standard path, the
//! advanced controls, and the specialist proof surfaces are all exposed here so
//! callers can learn one import path and stay there.
//!
//! ```no_run
//! use worth_runtime_bridge::facade::{
//!     BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode, MappingSelector,
//!     RuntimeBridge, SignalInvalidationScope, SnapshotReadContract, TruthPatchScope,
//!     TruthCommitIdentity,
//! };
//! use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
//!
//! fn facade_example<
//!     TruthSource,
//!     BranchHeads,
//!     ComputeSink,
//! >(
//!     truth_source: TruthSource,
//!     branch_heads: BranchHeads,
//!     compute_sink: ComputeSink,
//!     commit_identity: TruthCommitIdentity,
//! ) -> Result<(), Box<dyn std::error::Error>>
//! where
//!     TruthSource: worth_runtime_bridge::facade::RelationalBridgeSource + Clone + 'static,
//!     BranchHeads: worth_runtime_bridge::facade::TruthBranchHeadSource + Clone + 'static,
//!     ComputeSink: worth_runtime_bridge::facade::SignalBridgeSink + Clone + 'static,
//! {
//!     let bridge = RuntimeBridge::builder()
//!         .with_truth_source(truth_source)
//!         .with_truth_branch_head_source(branch_heads)
//!         .with_compute_sink(compute_sink)
//!         .register_mapping(BridgeMappingRegistration::new(
//!             BridgeMappingId::from_stable_name("pricing:steel"),
//!             TruthPatchScope::for_entity_field(
//!                 MappingSelector::exact("component:steel"),
//!                 AspectKey::new("cost").expect("valid aspect key"),
//!                 FieldKey::new("usd".to_owned()).expect("valid field key"),
//!             ),
//!             SnapshotReadContract::scalar(
//!                 AspectKey::new("cost").expect("valid aspect key"),
//!                 ScalarAspectType::String,
//!             ),
//!             SignalInvalidationScope::from_stable_name("price:bicycle"),
//!             CoarseRoutingMode::Direct,
//!         ))
//!         .build()?;
//!
//!     let route = bridge.route(commit_identity)?;
//!     let evaluation = bridge.evaluate_current(route.target())?;
//!     let diagnostics = bridge.diagnostics().explain_last();
//!
//!     let _ = evaluation;
//!     let _ = diagnostics;
//!     Ok(())
//! }
//! ```

use std::sync::Arc;

use crate::diagnostics::DiagnosticSink;
use crate::mapping::{FrozenAspectMappingRegistry, FrozenMappingRegistry};
use crate::subscription::FrozenSubscriptionFamilyRegistry;
use preview_session_liveness::BridgePreviewSessionLivenessOwner;

mod exports_core;
mod exports_subscription;
mod preview_session_liveness;
mod request;
mod runtime;
pub mod runtime_world;
mod standard_path;

pub use crate::authorization::{
    BridgeAuthorizationBindingIdentity, BridgeAuthorizationClauseContract,
    BridgeAuthorizationClauseObservation, BridgeAuthorizationCorrespondenceIdentity,
    BridgeAuthorizationDecisionEvidence, BridgeAuthorizationDenial, BridgeAuthorizationDenialKind,
    BridgeAuthorizationDependencyCardinality, BridgeAuthorizationInstallationBatch,
    BridgeAuthorizationInstallationRequest, BridgeAuthorizationObservation,
    BridgeAuthorizationRequirementContract, BridgeAuthorizationRequirementObservation,
    BridgeAuthorizationRuleContract, BridgeAuthorizationRuleDecisionEvidence,
    BridgeAuthorizationRuleEffect, BridgeAuthorizationRuleObservation, BridgeAuthorizationRuntime,
};
pub use crate::correspondence::{
    AdmittedRuntimeWorldCorrespondenceBasis, BridgeCorrespondenceAdmissionIdentity,
    RuntimeWorldCorrespondenceAdmissionDenial,
};
pub use crate::identity_authority::*;
pub use exports_core::*;
pub use exports_subscription::*;
pub use preview_session_liveness::{
    BridgePreviewSessionLivenessGuard, BridgePreviewSessionLivenessObserver,
};
pub use request::BridgeRouteRequest;
pub use runtime::RuntimeBridge;
pub use runtime_world::{
    RuntimeWorldCorrespondenceInspectionCounters, RuntimeWorldCorrespondencePort,
};
pub use standard_path::{
    BridgeDiagnostics, BridgeEvaluationTarget, BridgeRoute, BridgeSpeculativeComparison,
    BridgeSpeculativeDiscardOutcome, BridgeSpeculativePromotionOutcome,
    BridgeSpeculativeSessionHandle, BridgeSpeculativeSessionRequest,
    BridgeStandardDiagnosticsExplanation, BridgeStandardRouteError,
    BridgeStandardSessionExplanation, BridgeTruthViewEvaluation, BridgeTruthViewEvaluationRequest,
};

#[doc(hidden)]
pub mod everyday {
    pub use super::*;
}

#[doc(hidden)]
pub mod advanced {
    pub use super::*;
}

#[doc(hidden)]
pub mod specialist {
    pub use super::*;
}

#[cfg(test)]
pub(crate) mod tests;
