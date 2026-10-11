use super::{
    DemandRegistryState, DemandState, WorthQueryOutputDemandInterest, WorthQueryOutputDemandKey,
    WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn finish_superseded(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        subject: &str,
    ) -> WorthQueryOutputDemandDenial {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("superseded demand retains its owner record");
        match &record.state {
            DemandState::Failed(existing)
                if existing.kind() == WorthQueryOutputDemandDenialKind::Closed =>
            {
                return existing.clone();
            }
            DemandState::Output(output) => {
                if let super::WorthQueryOutputAdvancement::Stopped { denial, .. } =
                    &output.advancement
                {
                    return denial.clone();
                }
            }
            _ => {}
        }
        let denial = superseded_denial(subject);
        let unpublished = record.unpublished_new_key();
        match &mut record.state {
            DemandState::Output(output) => output.stop(denial.clone()),
            _ => record.state = DemandState::Failed(denial.clone()),
        }
        record.performed_source = None;
        let released = record.release_obligations();
        record.wake.notify();
        if unpublished {
            // The row ends without an output of its own: the newest Ready it
            // replaced answers for the occurrence again.
            state.revive_replaced_ready(&interest.key);
        }
        let released_prerequisites = state.release_record_prerequisites(&interest.key);
        state.obligation_reserved_bytes = state.obligation_reserved_bytes.saturating_sub(released);
        state.remove_required_member_if_released(&interest.key);
        drop(state);
        drop(released_prerequisites);
        denial
    }
}

pub(super) fn supersede_predecessors(
    state: &mut DemandRegistryState,
    successor: &WorthQueryOutputDemandKey,
    admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    // Admit the occurrence walk before changing any predecessor. Keyed mutation
    // and removal then use only this closed set, never unrelated registry rows.
    let mut keys = Vec::new();
    for (key, _) in
        super::refreshed_rejoin::occurrence_rows_admitted(&state.records, successor, admission)?
    {
        admission
            .charge_external_work(6)
            .map_err(|_| super::refreshed_rejoin::work_denial())?;
        if key != successor && key.replacement_order(successor) == Some(std::cmp::Ordering::Greater)
        {
            return Err(superseded_denial(&successor.producer));
        }
        let awaited = super::refreshed_rejoin::awaited_by_stale_owner_admitted(
            &state.records,
            key,
            admission,
        )?;
        keys.push((key.clone(), awaited));
    }
    // Every selected row may relinquish its own edges; quote their work
    // before stopping any predecessor, while refusal still has no effect.
    for (key, _) in &keys {
        if key != successor {
            state.admit_record_prerequisite_release(key, admission)?;
        }
    }
    // Fund the awaited facts before any predecessor changes state.
    for (key, _) in &keys {
        if key != successor && key.replacement_order(successor) == Some(std::cmp::Ordering::Less) {
            let record = state.records.get_mut(key).expect("occurrence key exists");
            let denial = superseded_denial(&key.producer);
            match &mut record.state {
                DemandState::Output(output) => output.stop(denial),
                _ => record.state = DemandState::Failed(denial),
            }
            record.performed_source = None;
            record.wake.notify();
            state.release_record_prerequisites(key);
        }
    }
    for (key, awaited) in keys {
        let record = state.records.get(&key).expect("occurrence key exists");
        let unfinished = match record.idle_disposition(awaited) {
            super::IdleRowDisposition::InFlight
            | super::IdleRowDisposition::KeptAnswer
            | super::IdleRowDisposition::HeldForStaleOwner => true,
            super::IdleRowDisposition::Retired(reason) => !reason.releases_finished_work(),
        };
        let keep = &key == successor
            || record.interests != 0
            || record.framework_required_count != 0
            || record.prepared_prerequisite_claims != 0
            || record.pending_cleanup_queued
            || unfinished;
        if !keep {
            // An incomparable abandoned publication was not stopped in the
            // predecessor pass, but owes the same release of its own claims.
            state.release_record_prerequisites(&key);
            let record = state.records.remove(&key).expect("selected row exists");
            state.obligation_reserved_bytes = state
                .obligation_reserved_bytes
                .saturating_sub(record.obligation_reserved_bytes());
            if let Some(member) = state.required_keys.take(&key) {
                state.required_reserved_bytes = state.required_reserved_bytes.saturating_sub(
                    super::required_members::member_bytes(member.as_ref())
                        .expect("admitted key charge fits"),
                );
            }
        }
    }
    Ok(())
}

pub(super) fn reject_older_successor(
    state: &DemandRegistryState,
    successor: &WorthQueryOutputDemandKey,
) -> Result<(), WorthQueryOutputDemandDenial> {
    // Another producer may still compute from this source until currentness
    // rejects its work; only a newer revision of this producer blocks it.
    if super::refreshed_rejoin::occurrence_rows(&state.records, successor).any(|(key, _)| {
        key != successor && key.replacement_order(successor) == Some(std::cmp::Ordering::Greater)
    }) {
        return Err(superseded_denial(&successor.producer));
    }
    Ok(())
}

fn superseded_denial(subject: &str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        subject.to_owned(),
    )
}
