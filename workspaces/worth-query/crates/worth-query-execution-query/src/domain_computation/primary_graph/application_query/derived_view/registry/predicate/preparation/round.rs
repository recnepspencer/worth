//! One interpretation collects only its own output-sized, originally funded notices.
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::{
    WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexStop as Stop,
    WorthQueryManagedIndexWorkDenial as WorkDenial,
};
use std::collections::BTreeMap;
use worth_execution::ExecutionRequestArrayBuilder;

struct Collected {
    slots: BTreeMap<WatchSlot, PredicateNoticeDisposition>,
    allocation: Arc<SourceFactsAllocation>,
}
pub struct Round {
    pub identity: NativeSourceInterpretationIdentity,
    pub descriptor: Arc<Descriptor>,
    collected: Mutex<Option<Collected>>,
}
pub use worth_query_execution_attempt::facade::ports::PreparedPredicateNotices;
impl Round {
    pub fn new(
        execution: &ExecutionRequest<'_, '_>,
        identity: NativeSourceInterpretationIdentity,
        descriptor: Arc<Descriptor>,
    ) -> Result<Self, Denial> {
        let allocation = SourceFactsAllocation::open(execution).map_err(storage)?;
        Ok(Self {
            identity,
            descriptor,
            collected: Mutex::new(Some(Collected {
                slots: BTreeMap::new(),
                allocation,
            })),
        })
    }
    pub fn record(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
        slot: WatchSlot,
        disposition: PredicateNoticeDisposition,
    ) -> Result<(), WorkDenial> {
        let mut collected = self
            .collected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let collected = collected.as_mut().ok_or(Stop::MembershipUnavailable)?;
        let count = collected
            .slots
            .len()
            .checked_add(1)
            .ok_or(Stop::CapacityOverflow)?;
        let levels = u64::from(usize::BITS - count.leading_zeros());
        control.checkpoint(levels.checked_mul(11 * 3).ok_or(Stop::CapacityOverflow)?)?;
        if let Some(existing) = collected.slots.get_mut(&slot) {
            if disposition == PredicateNoticeDisposition::Unavailable {
                *existing = disposition;
            }
        } else {
            let bytes = size_of::<(WatchSlot, PredicateNoticeDisposition)>()
                .checked_mul(11)
                .and_then(|n| n.checked_add(14 * size_of::<usize>()))
                .and_then(|n| n.checked_mul(2))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(Stop::CapacityOverflow)?;
            collected
                .allocation
                .claim(execution, bytes)
                .map_err(Stop::MaterialStorage)?;
            control.poll()?;
            collected.slots.insert(slot, disposition);
        }
        control.poll()?;
        Ok(())
    }
    pub fn freeze(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<PreparedPredicateNotices>, Denial> {
        let collected = self
            .collected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .ok_or(Denial::ForeignSource)?;
        let mut slots = ExecutionRequestArrayBuilder::allocate(*execution, collected.slots.len())
            .map_err(|cause| array_denial(execution, cause))?;
        for (&slot, &disposition) in &collected.slots {
            SourceFactsAllocation::work(execution, 1).map_err(storage)?;
            slots
                .push((slot, disposition))
                .map_err(|cause| array_denial(execution, cause))?;
        }
        let slots = slots
            .seal()
            .map_err(|cause| array_denial(execution, cause))?;
        let allocation = SourceFactsAllocation::open(execution).map_err(storage)?;
        allocation
            .claim_arc::<PreparedPredicateNotices>(execution)
            .map_err(storage)?;
        Ok(Arc::new(PreparedPredicateNotices::new(slots, allocation)))
    }
}
fn array_denial(
    execution: &ExecutionRequest<'_, '_>,
    cause: worth_execution::ExecutionRequestArrayDenial,
) -> Denial {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => storage(cause.into()),
        cause => crate::domain_computation::primary_graph::application_query::managed_index::source_preparation_denial(
            execution, WorkDenial::Controlled(Stop::Storage(cause)),
        ),
    }
}
