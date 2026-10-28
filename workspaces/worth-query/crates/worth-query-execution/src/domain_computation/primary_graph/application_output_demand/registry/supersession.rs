use super::DemandRegistryState;
use super::DemandState;
use super::WorthQueryOutputDemandKey;

use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

pub(super) fn supersede_predecessors(
    state: &mut DemandRegistryState,
    successor: &WorthQueryOutputDemandKey,
) -> Result<(), WorthQueryOutputDemandDenial> {
    // Admit the occurrence walk before changing any predecessor. Keyed mutation
    // and removal then use only this closed set, never unrelated registry rows.
    let mut keys = Vec::new();

    for (key, _) in super::refreshed_rejoin::occurrence_rows(&state.records, successor) {
        if key != successor && key.replacement_order(successor) == Some(std::cmp::Ordering::Greater)
        {
            return Err(superseded_denial(&successor.producer));
        }
        keys.push(key.clone());
    }
    let priced = state;
    {
        let priced: &mut DemandRegistryState = priced;
        {
            for key in &keys {
                let state = &mut *priced;
                if key != successor
                    && key.replacement_order(successor) == Some(std::cmp::Ordering::Less)
                {
                    let mut record = state.records.get_mut(key).expect("occurrence key exists");
                    crate::domain_computation::primary_graph::fact_verification_observation::registry_custody(
                    crate::domain_computation::primary_graph::WorthQueryRegistryCustodyTransitionForTest::AdmissionSupersession,
                    record.settlements.len(), record.prerequisites.len(),
                );
                    let denial = superseded_denial(&key.producer);
                    match &mut record.state {
                        DemandState::Output(output) => output.stop(denial),
                        _ => record.state = DemandState::Failed(denial),
                    }
                    record.performed_source = None;
                    record.wake.notify();
                    record.finish();
                    priced.release_record_prerequisites(key);
                }
            }
            for key in keys {
                let unfinished = match priced.disposition(&key).expect("occurrence key exists") {
                    super::IdleRowDisposition::InFlight
                    | super::IdleRowDisposition::KeptAnswer
                    | super::IdleRowDisposition::HeldForStaleOwner => true,
                    super::IdleRowDisposition::Retired(reason) => !reason.releases_finished_work(),
                };
                let state = &*priced;
                let record = state.records.get(&key).expect("occurrence key exists");
                let keep = &key == successor
                    || record.interests != 0
                    || record.framework_required_count != 0
                    || record.prepared_prerequisite_claims != 0
                    || record.pending_cleanup_queued
                    || unfinished;
                if !keep {
                    priced.release_record_prerequisites(&key);
                    let state = &mut *priced;
                    let removed = state.remove_record(&key).expect("selected row exists");
                    state.required_keys.take(&key);
                    drop(removed);
                }
            }
        }
    };
    Ok(())
}

pub(super) fn reject_older_successor(
    state: &DemandRegistryState,
    successor: &WorthQueryOutputDemandKey,
) -> Result<(), WorthQueryOutputDemandDenial> {
    // Another producer may still compute from this source until currentness
    // rejects its work; only a newer revision of this producer blocks it.

    for (key, _) in super::refreshed_rejoin::occurrence_rows(&state.records, successor) {
        let other_row = key != successor;
        if other_row && key.replacement_order(successor) == Some(std::cmp::Ordering::Greater) {
            return Err(superseded_denial(&successor.producer));
        }
    }
    Ok(())
}

pub(super) fn superseded_denial(subject: &str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        subject.to_owned(),
    )
}
