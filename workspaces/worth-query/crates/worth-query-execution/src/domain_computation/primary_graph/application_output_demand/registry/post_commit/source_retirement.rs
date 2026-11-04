//! Synchronous source-wide closure precedes admission-free custody readers.

use super::super::source_custody::denial;

use super::*;
use crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind;
use crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch;
use std::cmp::Ordering;
pub fn retire_stale_records(
    state: &mut DemandRegistryState,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    successor: &WorthQueryObservedSourceEpoch,
) {
    // Custody retirement is source-wide: a changed query source invalidates
    // every producer record that was computed from its older epoch.

    // Keys come from this exclusively borrowed registry's own source index.
    // The first pass changes row state and releases claims, queuing cleanup;
    // physical removal occurs only in the second pass, once for each unique key.
    let keys = super::super::refreshed_rejoin::source_rows(&state.records, successor)
        .filter(|(key, record)| {
            super::retirement_observation::record_sweep_contact();
            record.product_occurrence == occurrence
                && record.source_scope == Some(scope)
                && key.source.replacement_order(successor) == Some(Ordering::Less)
        })
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    for key in &keys {
        let mut record = state
            .records
            .get_mut(key)
            .expect("source occurrence key exists");
        crate::domain_computation::primary_graph::fact_verification_observation::registry_custody(
            crate::domain_computation::primary_graph::WorthQueryRegistryCustodyTransitionForTest::SourceRetirement,
            record.settlements.len(), record.prerequisites.len(),
        );
        let cause = denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            key.producer.clone(),
        );
        match &mut record.state {
            DemandState::Output(output) => output.stop(cause),
            _ => record.state = DemandState::Failed(cause),
        }
        record.performed_source = None;
        record.wake.notify();
        record.finish();
        state.release_record_prerequisites(key);
    }
    // Remove only after every stale row has released its prerequisites: an
    // upstream row that sorts before its consumer is held until then.
    for key in keys {
        let record = state
            .records
            .get(&key)
            .expect("source occurrence key exists");
        if record.interests == 0
            && record.framework_required_count == 0
            && record.prepared_prerequisite_claims == 0
            && !record.pending_cleanup_queued
        {
            state.remove_record(&key);
            state.required_keys.take(&key);
        }
    }
}
