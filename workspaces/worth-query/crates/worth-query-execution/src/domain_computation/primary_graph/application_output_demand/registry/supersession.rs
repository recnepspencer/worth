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
        record.wake.notify();
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
            let denial = superseded_denial(&key.producer);
            match &mut record.state {
                DemandState::Output(output) => output.stop(denial),
                _ => record.state = DemandState::Failed(denial),
            }
            record.performed_source = None;
            record.wake.notify();
        }
    }
    state.records.retain(|key, record| {
        key == successor
            || !key.same_occurrence(successor)
            || record.interests != 0
            || !matches!(record.state, DemandState::Failed(_))
                && !matches!(&record.state, DemandState::Output(output)
                    if matches!(output.advancement, super::WorthQueryOutputAdvancement::Stopped { .. }))
    });
    Ok(())
}

fn superseded_denial(subject: &str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::Superseded, subject)
}
