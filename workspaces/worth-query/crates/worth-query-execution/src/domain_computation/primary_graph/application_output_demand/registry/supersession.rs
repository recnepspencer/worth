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
        let denial = superseded_denial(subject);
        if !matches!(record.state, DemandState::Failed(ref existing) if existing.kind() == WorthQueryOutputDemandDenialKind::Closed)
        {
            record.state = DemandState::Failed(denial.clone());
            record.performed_source = None;
            record.wake.notify();
        }
        denial
    }
}

pub(super) fn supersede_predecessors(
    state: &mut DemandRegistryState,
    successor: &WorthQueryOutputDemandKey,
) -> Result<(), WorthQueryOutputDemandDenial> {
    if state.records.keys().any(|key| {
        key != successor && key.same_occurrence(successor) && key.revision() > successor.revision()
    }) {
        return Err(superseded_denial(&successor.producer));
    }
    for (key, record) in &mut state.records {
        if key != successor
            && key.same_occurrence(successor)
            && key.revision() < successor.revision()
        {
            record.state = DemandState::Failed(superseded_denial(&key.producer));
            record.performed_source = None;
            record.wake.notify();
        }
    }
    state.records.retain(|key, record| {
        key == successor
            || !key.same_occurrence(successor)
            || record.interests != 0
            || !matches!(record.state, DemandState::Failed(_))
    });
    Ok(())
}

fn superseded_denial(subject: &str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::Superseded, subject)
}
