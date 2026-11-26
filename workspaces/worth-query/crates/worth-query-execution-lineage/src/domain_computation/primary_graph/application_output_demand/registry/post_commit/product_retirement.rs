use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

impl WorthQueryOutputDemandRegistry {
    pub fn release_product_occurrence(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // World already retired this incarnation; this entry carries no completion receipt.

        state.note_custody_changes(|_, custody| {
            super::retirement_observation::record_sweep_contact();
            if custody.occurrence == occurrence {
                custody.retire(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Closed,
                    "product occurrence retired before required-output custody was consumed",
                ));
                return true;
            }
            false
        });
        if let Some(preparation) = state.source_preparations.get_mut(&occurrence) {
            preparation.retired = true;
        }
        let retired_cursors = state.take_discontinuity_cursors(occurrence);
        // Mark retained rows terminal before draining their descendants. A
        // descendant can release the final framework pin during this walk.
        for mut record in state.records.values_mut() {
            super::retirement_observation::record_sweep_contact();
            if record.product_occurrence != occurrence
                || (record.interests == 0
                    && record.framework_required_count == 0
                    && record.prepared_prerequisite_claims == 0)
            {
                continue;
            }
            let denial = WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Closed,
                "product occurrence retired",
            );
            match &mut record.state {
                DemandState::Output(output) => output.stop(denial),
                _ => record.state = DemandState::Failed(denial),
            }
            record.performed_source = None;
            record.readmission_source.take();
            record.wake.notify();
        }
        state.release_matching_prerequisites(|_, record| record.product_occurrence == occurrence);
        let drained = drain_occurrence_cleanup(&mut state, occurrence);
        let DemandRegistryState {
            records,
            required_keys,
            ..
        } = &mut *state;
        records.retain(|key, record| {
            super::retirement_observation::record_sweep_contact();
            if record.product_occurrence != occurrence {
                return true;
            }
            if unheld_retired_row(record) {
                required_keys.take(key);
                return false;
            }
            record.release_obligations();
            if !record.is_required() {
                required_keys.take(key);
            }
            true
        });
        drop(state);
        drop(drained);
        drop(retired_cursors);
    }
}

/// The single release pass frees one claim hop. A row whose last claimant
/// retired in that pass is queued with its own claims and output custody,
/// and a later admitted drain may never come for a retired occurrence. Drain
/// this occurrence's queued rows now, so its whole claim chain releases the
/// World history it observes (public #358). A refused step stays queued.
fn drain_occurrence_cleanup(
    state: &mut DemandRegistryState,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
) -> Vec<crate::domain_computation::primary_graph::application_output_demand::registry::cleanup::RetiredTerminalCleanup> {
    let mut drained = Vec::new();
    for _ in 0..state.records.len() {
        let queued_here = state.pending_cleanup_head.as_ref().is_some_and(|head| {
            state
                .records
                .get(head.as_ref())
                .is_some_and(|row| row.product_occurrence == occurrence)
        });
        if !queued_here || drained.try_reserve(1).is_err() {
            break;
        }
        match state.take_terminal_cleanup_step() {
            Ok(Some(step)) => drained.push(step),
            Ok(None) | Err(_) => break,
        }
    }
    drained
}

fn unheld_retired_row(record: &DemandRecord) -> bool {
    record.interests == 0
        && record.framework_required_count == 0
        && record.prepared_prerequisite_claims == 0
        && !record.pending_cleanup_queued
}

mod court_doors_for_test;
