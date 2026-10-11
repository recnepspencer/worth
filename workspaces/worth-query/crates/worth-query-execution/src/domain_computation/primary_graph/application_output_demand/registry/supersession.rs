use super::{DemandRegistryState, DemandState, WorthQueryOutputDemandKey};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

pub(super) fn supersede_predecessors(
    state: &mut DemandRegistryState,
    successor: &WorthQueryOutputDemandKey,
    admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    // Admit the occurrence walk before changing any predecessor. Keyed mutation
    // and removal then use only this closed set, never unrelated registry rows.
    let mut keys = Vec::new();
    let mut charge = super::refreshed_rejoin::WalkAdmission::new(
        admission,
        super::refreshed_rejoin::WalkKind::Supersession,
    );
    for (key, _) in
        super::refreshed_rejoin::occurrence_rows(&state.records, successor, &mut charge)?
    {
        charge
            .admission_mut()
            .charge_external_work(6)
            .map_err(|_| super::refreshed_rejoin::WalkKind::Supersession.denial())?;
        if key != successor && key.replacement_order(successor) == Some(std::cmp::Ordering::Greater)
        {
            return Err(superseded_denial(&successor.producer));
        }
        keys.push(key.clone());
    }
    let walks: Vec<_> = keys
        .iter()
        .filter(|key| *key != successor)
        .flat_map(|key| {
            state.records[key]
                .prerequisites
                .iter()
                .map(|upstream| (upstream.as_ref().clone(), 3))
        })
        .collect();
    super::refreshed_rejoin::price_mutation(state, admission, &walks, super::refreshed_rejoin::WalkKind::Supersession)?.complete(|priced| {
    for key in &keys {
        let state = priced.state_mut();
        if key != successor && key.replacement_order(successor) == Some(std::cmp::Ordering::Less) {
            let record = state.records.get_mut(key).expect("occurrence key exists");
            let denial = superseded_denial(&key.producer);
            match &mut record.state {
                DemandState::Output(output) => output.stop(denial),
                _ => record.state = DemandState::Failed(denial),
            }
            record.performed_source = None;
            record.wake.notify();
            priced.release_prerequisites(key);
        }
    }
    let state = priced.state_mut();
    for key in keys {
        let record = state.records.get(&key).expect("occurrence key exists");
        let keep = &key == successor
            || record.interests != 0
            || record.framework_required_count != 0
            || record.prepared_prerequisite_claims != 0
            || record.pending_cleanup_queued
            || !matches!(record.state, DemandState::Failed(_))
                && !matches!(&record.state, DemandState::Output(output)
                    if matches!(output.advancement, super::WorthQueryOutputAdvancement::Stopped { .. }));
        if !keep {
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
    });
    Ok(())
}

pub(super) fn reject_older_successor(
    state: &DemandRegistryState,
    successor: &WorthQueryOutputDemandKey,
    admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    use super::refreshed_rejoin::OccurrenceSearchCharge;
    // Another producer may still compute from this source until currentness
    // rejects its work; only a newer revision of this producer blocks it.
    let mut charge = super::refreshed_rejoin::WalkAdmission::new(
        admission,
        super::refreshed_rejoin::WalkKind::Supersession,
    );
    for (key, _) in
        super::refreshed_rejoin::occurrence_rows(&state.records, successor, &mut charge)?
    {
        charge.inspect_row()?;
        let other_row = {
            #[cfg(test)]
            super::demand_key::observe_row_comparison();
            key != successor
        };
        if other_row && key.replacement_order(successor) == Some(std::cmp::Ordering::Greater) {
            return Err(superseded_denial(&successor.producer));
        }
    }
    Ok(())
}

pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn superseded_denial(
    subject: &str,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        subject.to_owned(),
    )
}
