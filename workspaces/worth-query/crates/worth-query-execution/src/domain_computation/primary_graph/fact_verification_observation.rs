//! Actual retained-fact and computation handoff work, apart from request billing.
use std::cell::Cell;
pub(super) mod input_cutoff;
mod registry_custody;
pub use input_cutoff::{
    WorthQueryInputCutoffCandidateForTest, WorthQueryInputCutoffForTest,
    WorthQueryInputCutoffGateForTest, WorthQueryInputCutoffNativeBasisForTest,
    WorthQueryInputCutoffNoRowReasonForTest, WorthQueryInputCutoffOutcomeForTest,
    WorthQueryInputCutoffSettlementForTest, WorthQueryInputCutoffSettlementPostureForTest,
};
pub(super) use registry_custody::registry_custody;
pub use registry_custody::{
    WorthQueryRegistryCustodyForTest, WorthQueryRegistryCustodyTransitionForTest,
};

use crate::domain_computation::execution_observation;

/// Each field names a separate examination boundary; summing them would
/// double-count facts examined in several phases. These are not admission units.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryFactVerificationForTest {
    /// Current partition entries read to construct the native dispatch plan.
    pub computation_dispatch_plan_visits: u64,
    /// Actual settled native result cells consumed, including carried cells.
    pub computation_result_receiver_visits: u64,
    /// Partition routing entries inspected by the real preparation owner.
    pub computation_prepared_partition_visits: u64,
    /// Identities materialized for an explicit full tree shape/rebuild plan.
    pub computation_tree_plan_visits: u64,
    /// Item/call entries visited by the retained state's remaining flat sizing.
    pub computation_retained_size_visits: u64,
    /// Actual warm native retained-run maintenance, outside Query preparation.
    pub computation_native_range_visits: u64,
    pub computation_native_copied_nodes: u64,
    pub computation_native_carried_slot_visits: u64,
    /// Actual cold measured-call construction and warm changed-path/range work.
    pub computation_call_build_visits: u64,
    pub computation_call_node_copies: u64,
    pub computation_call_range_visits: u64,
    pub computation_call_range_carries: u64,
    pub computation_call_reach_visits: u64,
    pub computation_call_scope_lookups: u64,
    /// Actual registry transitions, including those before cutoff preparation.
    pub registry_custody: [Option<WorthQueryRegistryCustodyForTest>; 32],
    /// Further transitions are explicitly unobserved.
    pub registry_custody_omitted: u64,
    /// Up to 32 actual cutoff invocations, in completion order. No fact data.
    pub input_cutoffs: [Option<WorthQueryInputCutoffForTest>; 32],
    /// Further invocations are counted, never silently treated as observed.
    pub input_cutoffs_omitted: u64,
    /// Owned facts moved into pre-admitted native publication chunks.
    pub native_payload_fact_moves: u64,
    /// Actual canonical leaf and pair derivations, including cold construction.
    pub content_leaf_derivations: u64,
    pub content_pair_derivations: u64,
    pub content_field_index_visits: u64,
    /// Native chunks sealed for patched completion or postcommit publication.
    pub native_payload_chunks: u64,
    /// Pre-admitted two-child joins sealed for those chunks.
    pub native_payload_joins: u64,
    /// Facts examined to cache deep clone bounds at cold/changed native leaves.
    pub native_payload_size_visits: u64,
    /// Original/decoded flat payload entries visited for cached resident sizing.
    pub original_payload_size_visits: u64,
    /// Actual nodes visited during sparse patch preparation/application.
    pub native_patch_path_visits: u64,
    /// Original native Fact bodies deep-cloned after their admission.
    pub native_patch_fact_clones: u64,
    /// Current sealed computation facts visited while constructing ordinal/call metadata.
    pub sealed_computation_index_visits: u64,
    /// Cold/changed completion metadata entries visited for admission, payload
    /// construction and cached retained-bound maintenance. Counts repeated walks.
    pub completion_metadata_visits: u64,
    pub completed_fact_moves: u64,
    pub completed_fact_chunks: u64,
    pub completed_fact_joins: u64,
    pub carried_call_records: u64,
    pub fresh_read_key_records: u64,
    pub handler_read_key_records: u64,
    pub carried_key_expansion_visits: u64,
    pub fallback_carried_metadata_visits: u64,
    pub completion_call_roster_visits: u64,
    pub completion_fresh_key_visits: u64,
    /// Last refusal of an existing native completion candidate; no fact data.
    pub native_completion_last_decline: Option<&'static str>,
    /// Admitted source facts walked before interruption/validation filtering.
    pub source_admission_visits: u64,
    /// Postcommit capacity and endpoint preparation walks over source facts.
    /// A fact walked in both passes counts twice.
    pub rebase_preparation_visits: u64,
    pub provider_application_fact_visits: u64,
    pub effect_index_fact_visits: u64,
    pub posting_fact_visits: u64,
    pub posting_registration_visits: u64,

    pub rebase_touch_key_visits: u64,
    /// Postcommit resolution walk, before skipping facts after a failure.
    pub rebase_resolution_visits: u64,
    /// Unanswered source-fact classification, including short-circuit prefixes.
    pub rebase_unanswered_visits: u64,
    /// Entries into native source-currentness comparison, including refusal
    /// before a probe. The fact is counted before eligibility filtering.
    pub source_currentness_visits: u64,
    /// Entries into ordinary value/revision equality comparison.
    pub value_comparison_visits: u64,
    /// Retained fact keys walked to establish the observation work bound.
    pub retained_bound_visits: u64,
    /// Prior computation facts walked before deciding which owner calls moved.
    pub retained_computation_visits: u64,
    /// Retained keys actually passed to native fact observation, distinct from
    /// the prior-fact and admission-bound walks.
    pub retained_native_visits: u64,
    /// Prior facts walked during sealing before filtering for carried calls.
    pub carried_seal_visits: u64,
}

thread_local! {
    static OBSERVED: Cell<WorthQueryFactVerificationForTest> = const { Cell::new(WorthQueryFactVerificationForTest {
        computation_dispatch_plan_visits: 0, computation_result_receiver_visits: 0,
        computation_prepared_partition_visits: 0, computation_tree_plan_visits: 0, computation_retained_size_visits: 0,
        computation_native_range_visits: 0, computation_native_copied_nodes: 0, computation_native_carried_slot_visits: 0,
        computation_call_build_visits: 0, computation_call_node_copies: 0, computation_call_range_visits: 0, computation_call_range_carries: 0, computation_call_reach_visits: 0, computation_call_scope_lookups: 0,
        registry_custody: [None; 32], registry_custody_omitted: 0,
        input_cutoffs: [None; 32], input_cutoffs_omitted: 0,
        content_field_index_visits: 0, content_leaf_derivations: 0, content_pair_derivations: 0,
        native_payload_fact_moves: 0, native_payload_chunks: 0, native_payload_joins: 0,
        native_payload_size_visits: 0, original_payload_size_visits: 0, native_patch_path_visits: 0, native_patch_fact_clones: 0,
        sealed_computation_index_visits: 0, completion_metadata_visits: 0,
        completed_fact_moves: 0, completed_fact_chunks: 0, completed_fact_joins: 0,
        carried_call_records: 0, fresh_read_key_records: 0, handler_read_key_records: 0, carried_key_expansion_visits: 0, fallback_carried_metadata_visits: 0,
        completion_call_roster_visits: 0, completion_fresh_key_visits: 0, native_completion_last_decline: None,
        source_admission_visits: 0, source_currentness_visits: 0,
        provider_application_fact_visits: 0, effect_index_fact_visits: 0, posting_fact_visits: 0, posting_registration_visits: 0, rebase_touch_key_visits: 0,
        rebase_preparation_visits: 0, rebase_resolution_visits: 0, rebase_unanswered_visits: 0,
        value_comparison_visits: 0, retained_bound_visits: 0,
        retained_computation_visits: 0, retained_native_visits: 0, carried_seal_visits: 0,
    }) };
    static CLEANUP_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

/// Take this thread's examined-fact observations once. The existing owned
/// Query execution observation clears leftovers when its final guard drops.
#[doc(hidden)]
pub fn fact_verification_on_this_thread_for_test() -> WorthQueryFactVerificationForTest {
    OBSERVED.take()
}

pub(super) fn source_admission() {
    record(|observed| add(&mut observed.source_admission_visits, 1));
}

pub(super) fn computation_prepared_partition() {
    record(|observed| add(&mut observed.computation_prepared_partition_visits, 1));
}
pub(super) fn computation_call_reach() {
    record(|o| add(&mut o.computation_call_reach_visits, 1));
}
pub(super) fn computation_call_scope() {
    record(|o| add(&mut o.computation_call_scope_lookups, 1));
}
pub(super) fn computation_call_build() {
    record(|o| add(&mut o.computation_call_build_visits, 1));
}
pub(super) fn computation_call_copy() {
    record(|o| add(&mut o.computation_call_node_copies, 1));
}
pub(super) fn computation_call_range() {
    record(|o| add(&mut o.computation_call_range_visits, 1));
}
pub(super) fn computation_call_carry() {
    record(|o| add(&mut o.computation_call_range_carries, 1));
}
pub(super) fn computation_tree_plan() {
    record(|observed| add(&mut observed.computation_tree_plan_visits, 1));
}
pub(super) fn computation_retained_size() {
    record(|observed| add(&mut observed.computation_retained_size_visits, 1));
}

pub(super) fn computation_native_run(metrics: worth_execution::RetainedRunMetrics) {
    record(|observed| {
        add(
            &mut observed.computation_native_range_visits,
            metrics.range_visits,
        );
        add(
            &mut observed.computation_native_copied_nodes,
            metrics.copied_nodes,
        );
        add(
            &mut observed.computation_native_carried_slot_visits,
            metrics.carried_slot_visits,
        );
    });
}

pub(super) fn computation_dispatch_plan() {
    record(|observed| add(&mut observed.computation_dispatch_plan_visits, 1));
}
pub(super) fn computation_result_receiver() {
    record(|observed| add(&mut observed.computation_result_receiver_visits, 1));
}

pub(super) fn completion_metadata() {
    record(|observed| add(&mut observed.completion_metadata_visits, 1));
}

pub(super) fn completed_fact_move() {
    record(|observed| add(&mut observed.completed_fact_moves, 1));
}
pub(super) fn completed_fact_chunk() {
    record(|observed| add(&mut observed.completed_fact_chunks, 1));
}
pub(super) fn completed_fact_join() {
    record(|observed| add(&mut observed.completed_fact_joins, 1));
}
pub(super) fn carried_call_record() {
    record(|observed| add(&mut observed.carried_call_records, 1));
}
pub(super) fn fresh_read_key_record() {
    record(|observed| add(&mut observed.fresh_read_key_records, 1));
}
pub(super) fn handler_read_key_record() {
    record(|observed| add(&mut observed.handler_read_key_records, 1));
}
pub(super) fn carried_key_expansion() {
    record(|observed| add(&mut observed.carried_key_expansion_visits, 1));
}
pub(super) fn fallback_carried_metadata() {
    record(|observed| add(&mut observed.fallback_carried_metadata_visits, 1));
}
pub(super) fn completion_call_roster() {
    record(|observed| add(&mut observed.completion_call_roster_visits, 1));
}
pub(super) fn completion_fresh_key() {
    record(|observed| add(&mut observed.completion_fresh_key_visits, 1));
}
pub(super) fn native_completion_decline(reason: &'static str) {
    record(|observed| observed.native_completion_last_decline = Some(reason));
}

pub(super) fn original_payload_size() {
    record(|observed| add(&mut observed.original_payload_size_visits, 1));
}

pub(super) fn native_payload_size() {
    record(|observed| add(&mut observed.native_payload_size_visits, 1));
}
pub(super) fn native_patch_path() {
    record(|observed| add(&mut observed.native_patch_path_visits, 1));
}
pub(super) fn native_patch_clone() {
    record(|observed| add(&mut observed.native_patch_fact_clones, 1));
}

pub(super) fn native_payload_fact_move() {
    record(|observed| add(&mut observed.native_payload_fact_moves, 1));
}
pub(super) fn native_payload_chunk() {
    record(|observed| add(&mut observed.native_payload_chunks, 1));
}
pub(super) fn native_payload_join() {
    record(|observed| add(&mut observed.native_payload_joins, 1));
}

pub(super) fn rebase_preparation() {
    record(|observed| add(&mut observed.rebase_preparation_visits, 1));
}

pub(super) fn rebase_resolution() {
    record(|observed| add(&mut observed.rebase_resolution_visits, 1));
}

pub(super) fn rebase_unanswered() {
    record(|observed| add(&mut observed.rebase_unanswered_visits, 1));
}

pub(super) fn source_currentness() {
    record(|observed| add(&mut observed.source_currentness_visits, 1));
}

pub(super) fn value_comparison() {
    record(|observed| add(&mut observed.value_comparison_visits, 1));
}

pub(super) fn retained_computation() {
    record(|observed| add(&mut observed.retained_computation_visits, 1));
}

pub(super) fn retained_native() {
    record(|observed| add(&mut observed.retained_native_visits, 1));
}

pub(super) fn carried_seal() {
    record(|observed| add(&mut observed.carried_seal_visits, 1));
}

fn record(update: impl FnOnce(&mut WorthQueryFactVerificationForTest)) {
    if !execution_observation::is_installed() {
        return;
    }
    if !CLEANUP_REGISTERED.replace(true) {
        execution_observation::register_cleanup(clear);
    }
    let mut observed = OBSERVED.get();
    update(&mut observed);
    OBSERVED.set(observed);
}

fn add(total: &mut u64, value: u64) {
    *total = total
        .checked_add(value)
        .expect("fact examination counter overflow");
}

fn clear() {
    CLEANUP_REGISTERED.set(false);
    OBSERVED.take();
}

pub(super) fn sealed_computation_index() {
    record(|observed| add(&mut observed.sealed_computation_index_visits, 1));
}

pub(super) fn content_leaf() {
    record(|observed| add(&mut observed.content_leaf_derivations, 1));
}
pub(super) fn content_pair() {
    record(|observed| add(&mut observed.content_pair_derivations, 1));
}

pub(super) fn content_field_index() {
    record(|observed| add(&mut observed.content_field_index_visits, 1));
}

pub(super) fn provider_application_fact() {
    record(|observed| add(&mut observed.provider_application_fact_visits, 1));
}
pub(super) fn effect_index_fact() {
    record(|observed| add(&mut observed.effect_index_fact_visits, 1));
}
pub(super) fn posting_fact() {
    record(|observed| add(&mut observed.posting_fact_visits, 1));
}

pub(super) fn rebase_touch_key() {
    record(|observed| add(&mut observed.rebase_touch_key_visits, 1));
}

pub(super) fn posting_registration() {
    record(|observed| add(&mut observed.posting_registration_visits, 1));
}
