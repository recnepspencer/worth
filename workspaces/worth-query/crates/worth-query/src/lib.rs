//! `worth-query` is the internal Query engine crate. Its `facade` module is the
//! Workspace engine surface (`WorthQueryWorkspace`), and the crate owns typed
//! query authoring and proof-carrying canonical query artifacts.
//!
//! `worth-ui-query-binding`, `worth-query-replay`, and Query certification use
//! this surface. It is not an application API: application declarations use
//! `worth-query-decl`, application hosts use `worth-query-host`, and
//! certification code uses `worth-query-replay`.

#![forbid(unsafe_code)]

#[cfg(test)]
extern crate self as worth_query;

mod application;
mod aspect_field_authoring;
mod authoring;
mod authorized_projection;
mod basis;
mod basis_lifecycle;
mod binding;
mod binding_pipeline;
mod canonical_field_path_overlap_index;
mod canonicalization;
mod collection;
mod composition;
mod consumer_kit;
mod continuation_pipeline;
mod contribution_composed_orchestration;
mod correspondence;
mod correspondence_history;
mod correspondence_history_parity;
mod declarative_live;
mod diagnostics;
mod domain_capabilities;
mod domain_installation;
mod effect_lifecycle;
mod evidence_identity;
mod execution;
pub mod facade;
mod family_helpers;
mod frontier_planning;
mod frontier_signal_adapter;
mod grouped_authoring;
mod historical;
mod identity;
mod identity_authority;
mod identity_evolution;
mod integration_harness;
mod intent_admission;
mod live;
mod live_performance;
mod lower_runtime_routing;
mod memory_workspace;
mod orchestration_inventory;
mod ordinary;
mod ordinary_outcome;
mod planning;
mod policy_basis;
mod policy_certification;
mod policy_delivery;
mod policy_execution_seam;
mod policy_live;
mod policy_narrowing;
mod policy_plan;
mod preview;
mod program;
mod projection_consumption;
mod query_context;
mod recovery_boundary;
mod runtime;
mod saved_query;
#[macro_use]
mod schema_macro;
mod relationship_proof;
mod schema_view;
mod session_label;
mod signal_compatibility_orchestration;
mod subscription;
mod target_binding;
mod tenant_basis;
mod typed;
mod validation;
mod view_shape;
mod view_shape_live;
mod workflow;

#[cfg(test)]
mod future_signal_test_support;
pub(crate) use contribution_composed_orchestration::WorthQueryContributionComposedClassification;
pub(crate) use evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
#[cfg(test)]
pub(crate) use evidence_identity::{
    WorthQueryEvidenceIdentityComparisonError, WorthQueryEvidenceIdentityScheme,
};
#[cfg(test)]
pub(crate) use session_label::WorthQuerySessionLabel;

#[cfg(test)]
mod harness;
