//! Passive exact capability-use correspondence, never an authorization decision.
use super::*;
use crate::domain_computation::{
    authorization::WorthQueryRetainedCapabilityAuthorization, primary_graph::SourceFactsAllocation,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(in crate::domain_computation) struct WorthQueryCapabilityUseIdentity {
    request: WorthQueryRetainedCapabilityRequest,
    comparison_work: u64,
    // The shared immutable payload and actor provenance retire before this payer.
    _allocation: Arc<SourceFactsAllocation>,
}
impl WorthQueryCapabilityUseIdentity {
    pub(in crate::domain_computation) fn retain(
        authorization: &WorthQueryRetainedCapabilityAuthorization,
        execution: &ExecutionRequest<'_, '_>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Self, Cause> {
        Self::retain_request(authorization.request(), execution, allocation)
    }
    pub(super) fn retain_request(
        request: &WorthQueryRetainedCapabilityRequest,
        execution: &ExecutionRequest<'_, '_>,
        allocation: &Arc<SourceFactsAllocation>,
    ) -> Result<Self, Cause> {
        SourceFactsAllocation::work(execution, 32)?;
        let mut bytes = arc_bytes::<RetainedCapabilityRequestData>()?;
        for text in [
            &*request.resource_entity,
            &*request.context_name,
            &*request.context_type,
        ] {
            // Arc<str> has exact byte length, a control header and alignment padding.
            add(
                &mut bytes,
                text.len()
                    .checked_add(3 * size_of::<usize>())
                    .ok_or(Cause::ChargedBytesOverflow)?,
            )?;
        }
        for value in [
            Some(&request.action),
            Some(&request.purpose),
            request.field.as_ref(),
            request.magnitude.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            add(&mut bytes, value.owned_allocation_capacity_bytes())?;
        }
        if let Some(relation) = &request.related_relation {
            add(&mut bytes, relation.owned_allocation_capacity_bytes())?;
        }
        bytes = bytes
            .checked_add(
                request
                    .context
                    .retained_storage_bytes()
                    .ok_or(Cause::ChargedBytesOverflow)?,
            )
            .ok_or(Cause::ChargedBytesOverflow)?;
        for (key, _) in request.context.iter() {
            SourceFactsAllocation::work(execution, 6)?;
            add(
                &mut bytes,
                key.owned_allocation_capacity_bytes()
                    .ok_or(Cause::ChargedBytesOverflow)?,
            )?;
        }
        if let Some(actor) = &request.recorded_actor {
            bytes = bytes
                .checked_add(
                    actor
                        .retained_use_bytes()
                        .ok_or(Cause::ChargedBytesOverflow)?,
                )
                .ok_or(Cause::ChargedBytesOverflow)?;
        }
        allocation.claim(execution, bytes)?;
        Ok(Self {
            request: request.clone(),
            comparison_work: bytes.checked_add(32).ok_or(Cause::WorkCounterOverflow)?,
            _allocation: allocation.clone(),
        })
    }
    pub(in crate::domain_computation) fn descriptor<E>(
        &self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
        overflow: &dyn Fn() -> E,
    ) -> Result<[u8; 32], E> {
        super::material::descriptor(&self.request, admit, overflow)
    }
    pub(in crate::domain_computation) fn same_use<E>(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
    ) -> Result<bool, E> {
        admit(self.comparison_work)?;
        admit(other.comparison_work)?;
        let equal = self.request.matches_use(&other.request);
        admit(0)?;
        Ok(equal)
    }
    pub(in crate::domain_computation) fn matches(
        &self,
        authorization: &WorthQueryRetainedCapabilityAuthorization,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, Cause> {
        SourceFactsAllocation::work(execution, self.comparison_work)?;
        let candidate = authorization.request();
        Ok(self.request.matches_use(candidate))
    }
}
fn arc_bytes<T>() -> Result<u64, Cause> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .ok()
        .map(|(layout, _)| layout.pad_to_align().size())
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(Cause::ChargedBytesOverflow)
}
fn add(bytes: &mut u64, more: usize) -> Result<(), Cause> {
    *bytes = bytes
        .checked_add(u64::try_from(more).map_err(|_| Cause::ChargedBytesOverflow)?)
        .ok_or(Cause::ChargedBytesOverflow)?;
    Ok(())
}
