use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use crate::declarative_live::{
    declare_runtime_live_query_session_with_grouped_baseline, DeclarativeLiveQueryError,
    DeclarativeLiveQueryRequest, DeclarativeLiveViewShape,
};
use crate::memory_workspace::{
    WorthQueryEntity, WorthQueryMutationKind, WorthQueryMutationReceipt, WorthQueryWorkspaceError,
};
use crate::program::{
    validate_inputs, WorthQueryAuthorityRequirement, WorthQueryDerivedView,
    WorthQueryOperationInput, WorthQueryOperationOutput, WorthQueryProgram,
    WorthQueryProgramEffect, WorthQueryProgramError, WorthQueryProgramTrace,
};
#[cfg(not(test))]
use crate::schema_view::QuerySchemaView;
#[cfg(test)]
pub(crate) use crate::schema_view::{
    QuerySchemaView, ScalarAspectType, SchemaFieldView, SchemaRelationView,
};
use crate::session_label::WorthQuerySessionLabel;
use crate::subscription::{
    admit_active_subscription_lane, admit_query_subscription, attach_subscription_consumer,
    close_subscription_lifecycle, declare_query_subscription, lower_query_subscription_to_bridge,
    open_active_subscription_lane, prepare_subscription_activation,
    select_query_subscription_family, ActiveAllocationScopeWidth, ActiveFanoutWidth,
    ActiveRegistryLookupWidth, ActiveSubscriptionAllocationPosture, ActiveSubscriptionRuntime,
    ActiveSubscriptionWorkBudget, ConsumerDeliveryPacingWidth, DeliveryBackpressurePolicy,
    QuerySubscriptionAdmissionBudget, QuerySubscriptionAdmissionDimensions,
    QuerySubscriptionBridgeLoweringBudget, QuerySubscriptionSliceBudget,
    QuerySubscriptionWorkBudget, SubscriptionConsumerAttachmentBudget,
    SubscriptionConsumerAttachmentRequest, SubscriptionLifecycleCloseRequest,
};
use crate::view_shape_live::LiveViewShapeFamily;
use worth_relational::facade::branch::{RelationalForkDenial, RelationalForkOutcome};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{
    BridgeContinuityMutationBundle, BridgeNamingMutationBundle,
    BridgeSymbolicTargetReferenceBundle, RuntimeBridge,
};

/// Compose the two public owner fork operations for Query-owned fixtures.
/// The helper deliberately keeps the exact fork basis local to this call; no
/// raw branch selector is passed into Relational's fork transition.
pub(crate) fn fork_branch_from_exact_source(
    runtime: &mut RelationalRuntime,
    target_branch: BranchId,
    source_branch: &BranchId,
) -> Result<RelationalForkOutcome, RelationalForkDenial> {
    let (_, basis) = runtime.observe_fork_source(source_branch)?;
    runtime.fork_branch(target_branch, basis)
}

mod aspect_api_closeout;
mod async_result_identity;
mod async_result_projection;
mod async_result_state;
mod async_source_binding;
mod async_source_transition;
mod async_source_transition_plan;
mod authoritative_mutation_evidence_bridge_alignment;
mod authoritative_mutation_evidence_closeout;
mod authoritative_mutation_evidence_support;
mod authoritative_mutation_evidence_support_bridge;
mod authority;
mod backend;
mod branch;
mod bridge_mutation_lowering;
mod builder;
mod computed;
mod concurrent_hostile_matrix;
mod conditional_execution;
mod conditional_resources;
pub use conditional_resources::{
    WorthQueryConditionalEvaluationCacheBudget, WorthQueryConditionalEvaluationCacheBudgetDenial,
    WorthQueryConditionalEvaluationResourceObservation, WorthQueryConditionalExecutionResources,
};
pub use worth_query_execution::facade::integration::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetDenial, RuntimeWorldBudgetInstallation,
    RuntimeWorldBudgets, RuntimeWorldCustodyBudgetInstallation,
    RuntimeWorldHistoryBudgetInstallation, RuntimeWorldObservationBudgetInstallation,
    RuntimeWorldPublicationBudgetInstallation, RuntimeWorldRecoveryBudgetInstallation,
    RuntimeWorldRetentionBudgetInstallation, WorthQueryProductWorldClock,
    WorthQueryProductWorldResources,
};
mod facade_contract;
mod installed_live_routing;
mod installed_product;
pub(crate) use installed_product::WorthQueryExecutedConditional;
mod live_subscription_target_index;
mod primary_graph_source;
pub(crate) mod product_branch;
mod product_selection;
mod runtime_root_state;
mod settlement_repair;
mod shared_projection_owners;
pub use primary_graph_source::{
    WorthQueryPrimaryGraphSourceAdapter, WorthQueryPrimaryGraphSourceProjection,
};
pub(crate) use shared_projection_owners::{
    WorthQuerySharedPrimaryOwnerRefreshStop, WorthQuerySharedProjectionLeaseToken,
};
mod delivery;
mod domain_installation_api;
mod downstream_delivery_contract;
mod downstream_delivery_resume;
mod effect;
mod error;
mod evidence_identities;
#[cfg(test)]
pub(crate) use evidence_identities::{
    runtime_state_snapshot_basis_label_identity, runtime_state_snapshot_result_shape_label_identity,
};
#[cfg(test)]
mod fallback_seam_counters;
mod granular_source_read_basis;
mod graph_read_access;
#[doc(hidden)]
pub use granular_source_read_basis::WorthQueryGranularSourceReadBasis;
pub(crate) use graph_read_access::WorthQueryGraphReadOperationLookup;
mod handle_contract;
mod inspection;
mod installed_domain_substrate_provenance;
mod intent;
mod journal_position;
mod journal_replay;
mod live_subscription;
mod live_subscription_accessors;
mod live_subscription_delivery_routing;
mod managed_live_resource;
mod materialized_fact_posture;
mod mixed_cause_delivery;
#[cfg(test)]
mod mixed_cause_emission;
mod mutation;
mod mutation_surface;
pub(crate) mod native_aspect_contracts;
mod ordinary_inspection_execution;
mod ordinary_runtime_posture;
mod ordinary_workflow_authority;
mod ordinary_workflow_branch_name;
mod ordinary_workflow_execution;
mod preview;
mod primary_graph;
pub(crate) use installed_domain_substrate_provenance::WorthQueryInstalledDomainSubstrateProvenance;
mod public_api;
mod published_artifacts;
mod read_composition;
mod read_composition_builder_shared;
mod read_composition_builder_walks;
mod read_composition_current_execution;
mod read_composition_frontier;
mod read_composition_frontier_search;
mod read_composition_installed_operation;
mod read_composition_lowering;
mod read_composition_materialization;
mod read_composition_operator_builders;
mod read_composition_phase_gate;
mod read_composition_phase_one_closeout;
mod read_composition_relationship_proof;
mod read_composition_row_selection;
mod read_composition_runtime;
mod read_composition_shared;
mod read_composition_successor;
mod read_composition_support_report;
mod read_composition_walks;
mod remask_posture;
mod runtime_api_contract;
mod runtime_authoritative_mutation_routing;
use runtime_authoritative_mutation_routing::{
    WorthQueryAuthoritativeMutationExecutionEvidence, WorthQueryAuthoritativeMutationRoutingInput,
    WorthQueryPreparedAuthoritativeMutationRouting,
};
mod bridge_async_live_view_declaration;
mod owned_async_lifecycle;
mod owned_async_source;
mod owned_async_supersession;
mod runtime_batch_write_entrypoints;
mod runtime_batch_write_intents;
mod runtime_batch_writes;
mod runtime_batching;
mod runtime_declarations;
mod runtime_helpers;
mod runtime_inspection;
mod runtime_inspection_materialization_identity;
mod runtime_inspection_materialization_intents;
mod runtime_intent_phase_four_execution;
mod runtime_intent_phase_three_resolution;
mod runtime_intents;
mod runtime_journal_replay;
mod runtime_live_read_intents;
mod runtime_probe_routing_intents;
mod runtime_provenance;
mod runtime_read_access_plan;
mod runtime_read_execution_receipts;
mod runtime_read_intents;
mod runtime_reads_programs;
mod runtime_session_lowering;

pub(crate) use read_composition_row_selection::{
    canonical_ordering_key, row_matches_predicates, WorthQueryCanonicalOrderingKey,
};
mod runtime_sessions;
mod runtime_unified_inspection_intents;
mod runtime_write_intents;
mod runtime_writes;
mod shared_read;
mod shared_read_pins;
mod state;
mod state_basis;
mod state_snapshot;
mod support;
mod support_matrix;
mod surface;
#[cfg(test)]
mod time_only_delivery;
mod workspace;
mod workspace_contracts;
mod workspace_declaration;
mod workspace_domain_installation;
mod workspace_graph;
mod workspace_inspection;
mod workspace_live_queries;
mod workspace_live_view_close;
mod workspace_mutations;
mod workspace_queries;
mod workspace_read_composition_support;
mod workspace_shared_read;
mod workspace_submission;

const RUNTIME_SUBSCRIPTION_FAMILY_BUDGET_POLICY: &str =
    "runtime-live-subscription-family:scratch_buffer_only:canonical=64:relationship=64:policy=64:projection=512:tenant=1";
const RUNTIME_SUBSCRIPTION_SLICE_BUDGET_POLICY: &str =
    "runtime-live-subscription-slice:scratch_buffer_only:all-widths=64";
const RUNTIME_SUBSCRIPTION_BRIDGE_BUDGET_POLICY: &str =
    "runtime-live-subscription-bridge:admitted:bridge=1:slice=64:policy=64:basis=64:signal=64";
const RUNTIME_SUBSCRIPTION_ADMISSION_BUDGET_POLICY: &str =
    "runtime-live-subscription-admission:admitted:all-widths=64";
const RUNTIME_ACTIVE_LIFECYCLE_BUDGET_POLICY: &str =
    "runtime-live-active-lifecycle:registry=1:fanout=1:allocation=1:lifecycle_arena";
const RUNTIME_CONSUMER_ATTACHMENT_BUDGET_POLICY: &str =
    "runtime-live-consumer-attachment:fanout=1:pacing=1:allocation=1:retain_within_window";

pub(crate) use backend::{build_bridge_authority_bundle, WorthQueryBridgeMutationTarget};
pub(crate) use branch::WorthQueryRuntimeBranchComparisonBasis;
use bridge_mutation_lowering::{bridge_continuity_mutation_bundle, bridge_naming_mutation_bundle};
use computed::{
    admit_derived_view_declaration, insert_derived_runtime,
    retained_live_view_names_for_candidates, route_derived_view_patches,
    WorthQueryComputedDependencyIndex, WorthQueryDerivedViewRuntime,
};
pub(crate) use delivery::WorthQueryLiveMutationRoutingWork;
use delivery::{
    WorthQueryRuntimeLiveSubscriptionActivation, WorthQueryRuntimeLiveSubscriptionState,
};
use downstream_delivery_contract::project_downstream_delivery;
use downstream_delivery_resume::{aggregate_support_posture, support_gate_resume_kind};
use effect::{
    admit_effect_declaration, insert_effect_runtime, route_effect_deliveries,
    WorthQueryEffectIndex, WorthQueryEffectRuntime, WorthQueryEffectTarget,
};
#[cfg(test)]
pub(crate) use error::{
    WorthQueryRuntimeDeclarationFailureKind, WorthQueryRuntimeLookupFailureKind,
    WorthQueryRuntimeMissingArtifactKind,
};
#[cfg(test)]
pub(crate) use fallback_seam_counters::{
    forbidden_fallback_seam_invocation_count, record_forbidden_fallback_seam_invocation,
    reset_forbidden_fallback_seam_invocations, WorthQueryForbiddenFallbackSeam,
};
pub(crate) use graph_read_access::provision_ephemeral_graph_indexes_for_read_execution;
pub(crate) use graph_read_access::streaming_receipt_for_admitted_read_result;
#[cfg(test)]
pub(crate) use graph_read_access::{
    admit_graph_read_access_authority_from_policy_tenant_request,
    admit_graph_read_access_for_family, explain_boolean_selectivity_shape_for_family,
    explain_graph_read_access_requirement_outcome_for_family_with_operation_lookup,
    explain_graph_read_access_requirements_for_family,
    explain_graph_read_access_requirements_for_family_with_operation_lookup,
    explain_graph_read_access_shape_for_family,
    explain_graph_read_access_shape_for_family_with_operation_lookup,
    plan_admitted_graph_read_access_for_family,
    resolve_graph_read_operations_for_family_with_operation_lookup,
};
pub(crate) use graph_read_access::{
    admit_graph_read_access_for_family_in_authority_with_inventory_and_lookup,
    explain_boolean_selectivity_shape_for_family_in_authority_with_lookup,
    explain_graph_read_access_requirements_for_family_in_authority_with_lookup,
    explain_graph_read_access_shape_for_family_in_authority_with_lookup,
};
pub(crate) use graph_read_access::{
    WorthQueryGraphReadOperationRegistration, WorthQueryGraphReadOperationRegistry,
    WorthQueryGraphReadRegistryAdmissionError,
};
pub(crate) use inspection::request_causal_inspection;
#[cfg(test)]
pub(crate) use inspection::{admit_causal_inspection, resolve_indexed_causal_evidence_references};
pub(crate) use intent::{
    admit_authoritative_intent_declaration, admit_authoritative_intent_execution,
    admit_effect_triggered_intent_declaration, WorthQueryIntentAdmissionDenial,
};
#[cfg(test)]
pub(crate) use journal_replay::journal_replay_truth_reconstruction_identity;
pub(crate) use live_subscription::{
    live_subscription_source_identity, live_subscription_view_shape_source_identity,
};
use live_subscription_delivery_routing::route_live_subscription_delivery;
pub(crate) use managed_live_resource::{
    WorthQueryManagedLiveResourceCloseCause, WorthQueryManagedLiveRuntimeDelivery,
    WorthQueryManagedLiveWorkspaceCapability,
};
#[cfg(test)]
pub(crate) use mutation::WorthQueryDesiredAspectValue;
use mutation::{admit_continuity_intent, admit_naming_intent};
pub(crate) use mutation::{
    command_declared_aspect_value_digest, command_declared_aspect_value_identity,
};
#[cfg(test)]
pub(crate) use native_aspect_contracts::WorthQueryMutationContractDenialKind;
pub(crate) use ordinary_workflow_authority::{
    WorthQueryLowerRuntimeMutationExecution, WorthQueryMergeAuthorityValidationError,
    WorthQueryOrdinaryAuthorityAdmission, WorthQueryOrdinaryAuthorityDrift,
    WorthQueryOrdinaryAuthorityFamily, WorthQueryValidatedMergeAuthority,
};
pub(crate) use ordinary_workflow_execution::{
    WorthQueryLowerRuntimeMergeExecution, WorthQueryLowerRuntimePreviewExecution,
    WorthQueryLowerRuntimeWritebackExecution, WorthQueryOrdinaryMergeExecutionError,
    WorthQueryOrdinaryMergeFailureStage, WorthQueryOrdinaryWritebackExecutionError,
    WorthQueryOrdinaryWritebackFailureStage,
};
#[cfg(test)]
use runtime_helpers::runtime_subscription_budget_digest;
use runtime_helpers::{
    admit_authority_requirements, attach_continuity_mutation_to_receipt,
    attach_naming_mutation_to_receipt, attach_symbolic_target_reference_to_receipt,
    classify_receipt_mutation_summary, combined_batch_mutation_receipt, live_subscription_error,
    record_same_batch_symbolic_target, resolve_same_batch_symbolic_target,
    resolve_symbolic_aspect_references, runtime_active_lifecycle_budget,
    runtime_active_lifecycle_budget_policy, runtime_bridge_lowering_budget,
    runtime_consumer_attachment_budget, runtime_consumer_attachment_budget_policy,
    runtime_family_budget, runtime_slice_budget, runtime_subscription_admission_budget,
    runtime_subscription_budget_policy, subscription_dimensions_for_request,
    synthetic_existing_assertion_receipt, WorthQuerySameBatchSymbolicTarget,
    WorthQuerySameBatchSymbolicTargetKey,
};
#[cfg(test)]
pub(crate) use shared_read::WorthQuerySharedReadBasisInspection;
pub(in crate::runtime) use shared_read_pins::{
    worth_query_shared_read_stale_basis_error, WorthQuerySharedReadGenerationLease,
};
pub(in crate::runtime) use surface::WorthQueryReadExecutionProduct;
pub(crate) use worth_query_execution::facade::runtime::WorthQueryRuntimeAuthorityIdentity;

use runtime_root_state::WorthQueryRoutedMutationSummary;

// The established flat runtime API remains available as runtime::Name.
pub use facade_contract::*;

#[cfg(test)]
pub(crate) mod tests;
