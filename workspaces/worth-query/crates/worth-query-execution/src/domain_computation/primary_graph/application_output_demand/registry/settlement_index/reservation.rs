//! Reserve the exact invisible settlement address before owner effects.
use super::*;
use std::collections::btree_map::Entry;
pub(super) fn reserve(
    state: &mut DemandRegistryState,
    identity: &Arc<RecordedSettlementIdentity>,
) -> Result<(Posting, Box<PendingVacancyCleanup>), WorthQueryOutputDemandDenial> {
    state.drain_cancelled_settlement_vacancies();
    let source = state
        .settlement_keys
        .sources
        .entry(identity.source().clone())
        .or_default();
    let Entry::Vacant(vacant) = source.entry(identity.address()) else {
        return Err(WorthQueryOutputDemandDenial::new(
            Kind::SchedulingDeferred,
            "another attempt owns the exact settlement address",
        ));
    };
    let posting = Arc::new(Mutex::new(None));
    let cleanup = Box::new(PendingVacancyCleanup {
        identity: Arc::clone(identity),
        next: None,
    });
    vacant.insert(Arc::clone(&posting));
    Ok((posting, cleanup))
}
