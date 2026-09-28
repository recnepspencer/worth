//! `worth-runtime-bridge` owns the runtime bridge facade and its internal
//! subdomain boundaries.
//!
//! Milestone 1 currently includes canonical truth-envelope and snapshot input
//! contracts, build-time-frozen bridge mapping registration, and deterministic
//! routing plan / invalidation artifact lowering.
//!
//! Milestone 5 bulk planning begins by composing canonical single-route plans
//! into a replay-safe batch workload identity and summary without reintroducing
//! scalar orchestration at the public boundary.
//!
//! For ordinary usage, start with [`facade`].
//!
//! ```no_run
//! use worth_runtime_bridge::facade::{
//!     BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode, MappingSelector,
//!     RuntimeBridge, SignalInvalidationScope, SnapshotReadContract, TruthPatchScope,
//!     TruthCommitIdentity,
//! };
//! use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
//!
//! fn standard_path_example<
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
//!             BridgeMappingId::from_stable_name("pricing:catalog"),
//!             TruthPatchScope::for_entity_field(
//!                 MappingSelector::exact("item:widget"),
//!                 AspectKey::new("cost").expect("valid aspect key"),
//!                 FieldKey::new("usd".to_owned()).expect("valid field key"),
//!             ),
//!             SnapshotReadContract::scalar(
//!                 AspectKey::new("cost").expect("valid aspect key"),
//!                 ScalarAspectType::String,
//!             ),
//!             SignalInvalidationScope::from_stable_name("order:total"),
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

#![forbid(unsafe_code)]

mod adapter;
mod authorization;
mod builder;
mod canonical_basis;
#[cfg(feature = "certification-construction")]
pub mod certification;
mod certification_constructor;
mod clone_budget;
mod conditional_execution;
mod continuity;
mod correspondence;
mod delivery;
mod diagnostics;
mod error;
mod execution_basis;
pub mod facade;
mod historical;
mod identity;
mod identity_authority;
mod input;
mod mapping;
mod merge;
mod policy;
mod relational_grouped_truth;
mod relational_source;
mod routing;
mod snapshot;
mod source;
mod speculation;
mod stream;
mod structural;
mod subscription;
mod temporal;
mod writeback;

#[cfg(test)]
mod harness;
#[cfg(test)]
mod truth_identity_fixtures;

#[cfg(test)]
mod tests {}
