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
        match &mut record.state {
            DemandState::Output(output) => output.stop(denial.clone()),
            _ => record.state = DemandState::Failed(denial.clone()),
        }
        record.performed_source = None;
        let released = record.release_obligations();
        record.wake.notify();
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
) -> Result<(), WorthQueryOutputDemandDenial> {
    reject_older_successor(state, successor)?;
    for (key, record) in &mut state.records {
        if key != successor && key.replacement_order(successor) == Some(std::cmp::Ordering::Less) {
            let denial = superseded_denial(&key.producer);
            match &mut record.state {
                DemandState::Output(output) => output.stop(denial),
                _ => record.state = DemandState::Failed(denial),
            }
            record.performed_source = None;
            record.wake.notify();
        }
    }
    state.release_matching_prerequisites(|key, _| {
        key != successor && key.replacement_order(successor) == Some(std::cmp::Ordering::Less)
    });
    let mut released_bytes = 0;
    let required_keys = &mut state.required_keys;
    let required_reserved_bytes = &mut state.required_reserved_bytes;
    state.records.retain(|key, record| {
        let keep = key == successor
            || !key.same_occurrence(successor)
            || record.interests != 0
            || record.framework_required_count != 0
            || record.prepared_prerequisite_claims != 0
            || record.pending_cleanup_queued
            || !matches!(record.state, DemandState::Failed(_))
                && !matches!(&record.state, DemandState::Output(output)
                    if matches!(output.advancement, super::WorthQueryOutputAdvancement::Stopped { .. }));
        if !keep {
            released_bytes += record.obligation_reserved_bytes();
            if let Some(member) = required_keys.take(key) {
                *required_reserved_bytes = required_reserved_bytes.saturating_sub(
                    super::required_members::member_bytes(member.as_ref())
                        .expect("admitted key charge fits"),
                );
            }
        }
        keep
    });
    state.obligation_reserved_bytes = state
        .obligation_reserved_bytes
        .saturating_sub(released_bytes);
    Ok(())
}

pub(super) fn reject_older_successor(
    state: &DemandRegistryState,
    successor: &WorthQueryOutputDemandKey,
) -> Result<(), WorthQueryOutputDemandDenial> {
    // Another producer may still compute from this source until currentness
    // rejects its work; only a newer revision of this producer blocks it.
    if state.records.keys().any(|key| {
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
