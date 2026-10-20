use super::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_execution::{ExecutionAllocationPolicy, ExecutionMemoryReservation};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph::application_attempt) struct CandidateBackingReservation
{
    memory: ExecutionMemoryReservation,
    status: worth_execution::ExecutionLeaseStatus,
}

impl CandidateBackingReservation {
    fn check_live(&self) -> Result<(), Denial> {
        if self.status.is_cancelled() {
            return Err(Denial::new(
                Kind::Cancelled,
                "original candidate backing request cancelled",
            ));
        }
        if self.status.deadline_elapsed() {
            return Err(Denial::new(
                Kind::DeadlineExceeded,
                "original candidate backing deadline elapsed",
            ));
        }
        Ok(())
    }

    fn resize(&mut self, bytes: u64) -> Result<(), Denial> {
        self.check_live()?;
        self.memory
            .resize(bytes)
            .map_err(Denial::candidate_memory_denied)?;
        self.check_live()
    }
}

/// Allocates only the actual effect slots. Nested heaps remain separately owned.
/// The original request ticket must travel with the returned backing.
pub(super) fn allocate<T>(
    count: usize,
    policy: ExecutionAllocationPolicy<'_, '_>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<(Vec<T>, Option<CandidateBackingReservation>), Denial> {
    check_request(request)?;
    policy
        .check_live()
        .map_err(|cause| Denial::allocation_denied("candidate backing", cause))?;
    let layout = std::alloc::Layout::array::<T>(count).map_err(|_| {
        Denial::new(
            Kind::CandidateCapacityExceeded,
            "candidate backing layout overflow",
        )
    })?;
    let bytes = u64::try_from(layout.size()).map_err(|_| {
        Denial::new(
            Kind::CandidateCapacityExceeded,
            "candidate backing byte quote overflow",
        )
    })?;
    let memory = match policy {
        ExecutionAllocationPolicy::SystemAllocation => None,
        ExecutionAllocationPolicy::Execution(lease) => Some(CandidateBackingReservation {
            memory: lease
                .reserve_memory(bytes)
                .map_err(Denial::candidate_memory_denied)?,
            status: lease.status(),
        }),
    };
    policy
        .check_live()
        .map_err(|cause| Denial::allocation_denied("candidate backing", cause))?;
    check_request(request)?;
    // Locals drop in reverse order: destroy the allocation before refunding it.
    let mut effects = Vec::new();
    effects.try_reserve_exact(count).map_err(|_| {
        Denial::new(
            Kind::AllocationDenied,
            "candidate backing allocator refusal",
        )
    })?;
    if std::mem::size_of::<T>() != 0 && effects.capacity() != count {
        return Err(Denial::new(
            Kind::AllocationDenied,
            "candidate backing capacity differs from admitted layout",
        ));
    }
    policy
        .check_live()
        .map_err(|cause| Denial::allocation_denied("candidate backing", cause))?;
    check_request(request)?;
    Ok((effects, memory))
}

/// A platform sidecar grows the same backing under its original ticket.
pub(in crate::domain_computation::primary_graph::application_attempt) fn extend<T>(
    effects: &mut Vec<T>,
    memory: &mut Option<CandidateBackingReservation>,
    additional: Vec<T>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<(), Denial> {
    check_request(request)?;
    if let Some(memory) = memory.as_ref() {
        memory.check_live()?;
    }
    let count = effects.len().checked_add(additional.len()).ok_or_else(|| {
        Denial::new(
            Kind::CandidateCapacityExceeded,
            "candidate backing count overflow",
        )
    })?;
    if count > effects.capacity() {
        let quote = |count| {
            std::alloc::Layout::array::<T>(count)
                .ok()
                .and_then(|layout| u64::try_from(layout.size()).ok())
                .ok_or_else(|| {
                    Denial::new(
                        Kind::CandidateCapacityExceeded,
                        "candidate backing growth layout overflow",
                    )
                })
        };
        let previous = quote(effects.capacity())?;
        let next = quote(count)?;
        let coexistence = previous.checked_add(next).ok_or_else(|| {
            Denial::new(
                Kind::CandidateCapacityExceeded,
                "candidate backing coexistence overflow",
            )
        })?;
        if let Some(memory) = memory.as_mut() {
            memory.resize(coexistence)?;
        }
        if effects.try_reserve_exact(count - effects.len()).is_err() {
            if let Some(memory) = memory.as_mut() {
                memory.resize(previous)?;
            }
            return Err(Denial::new(
                Kind::AllocationDenied,
                "candidate backing growth allocator refusal",
            ));
        }
        if std::mem::size_of::<T>() != 0 && effects.capacity() != count {
            return Err(Denial::new(
                Kind::AllocationDenied,
                "candidate backing growth capacity mismatch",
            ));
        }
        if let Some(memory) = memory.as_mut() {
            memory.resize(next)?;
        }
    }
    check_request(request)?;
    if let Some(memory) = memory.as_ref() {
        memory.check_live()?;
    }
    effects.extend(additional);
    check_request(request)
}

pub(super) fn check_request(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<(), Denial> {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
    match request.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => Err(Denial::new(
            Kind::Cancelled,
            "candidate growth request cancelled",
        )),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(Denial::new(
            Kind::DeadlineExceeded,
            "candidate growth deadline elapsed",
        )),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests;
