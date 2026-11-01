//! Freeze the actual semantic union, without cloning mutable count storage.
use super::*;
use worth_execution::ExecutionAllocationPolicy;

struct Keys {
    values: Vec<Member>,
    _memory: Option<ExecutionMemoryReservation>,
}
fn live_keys(
    registry: &MemberCustodyRegistry,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<Keys, Refusal> {
    let (count, active) = {
        let state = read(registry, request.is_some())?;
        (state.counts.len(), state.selection.upgrade())
    };
    if active.is_some() {
        return Err(Refusal::Storage(Cause::SourceUnavailable));
    }
    let bytes = std::alloc::Layout::array::<Member>(count)
        .map_err(|_| Refusal::Bounds)?
        .size();
    let memory = request
        .map(|request| {
            request
                .reserve_retained_memory(bytes as u64)
                .map_err(Refusal::from)
        })
        .transpose()?
        .flatten();
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| Refusal::Storage(Cause::MemoryExhausted))?;
    {
        let state = read(registry, request.is_some())?;
        if state.counts.len() > values.capacity() {
            return Err(Refusal::Busy);
        }
        for key in state.counts.keys() {
            if let Some(request) = request {
                request.consult()?;
            }
            values.push(*key);
        }
    }
    Ok(Keys {
        values,
        _memory: memory,
    })
}
fn read(
    registry: &MemberCustodyRegistry,
    controlled: bool,
) -> Result<std::sync::RwLockReadGuard<'_, CustodyState>, Refusal> {
    if !controlled {
        return Ok(registry
            .state
            .read()
            .unwrap_or_else(|error| error.into_inner()));
    }
    match registry.state.try_read() {
        Ok(state) => Ok(state),
        Err(std::sync::TryLockError::Poisoned(error)) => Ok(error.into_inner()),
        Err(std::sync::TryLockError::WouldBlock) => Err(Refusal::Busy),
    }
}
impl MemberCustodyRegistry {
    pub(in crate::cold_source) fn protect_selection_for_request(
        &self,
        next: impl IntoIterator<Item = Member>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RetainedMemberPublication, Refusal> {
        self.protect(next, Some(request))
    }
    fn protect(
        &self,
        next: impl IntoIterator<Item = Member>,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<RetainedMemberPublication, Refusal> {
        let old = live_keys(self, request)?;
        let mut members = ExecutionOrderedMap::default();
        for member in next.into_iter().chain(old.values.iter().copied()) {
            match request {
                Some(request) => members.insert_cloned_native_for_request(&member, &(), *request),
                None => members.insert_cloned_native_in(
                    &member,
                    &(),
                    ExecutionAllocationPolicy::SystemAllocation,
                ),
            }
            .map_err(|cause| match cause {
                worth_execution::ExecutionOrderedMapDenial::Request(cause) => {
                    Refusal::Request(cause.into())
                }
                _ => Refusal::Storage(Cause::MemoryExhausted),
            })?;
        }
        // Fixed Arc/Weak control is the preexisting OPEN metadata obligation.
        // Actual scalable roster nodes retain their own original request grants.
        let selection = Arc::new(FrozenSelection { members });
        let active;
        {
            let mut state = match request {
                Some(_) => match self.state.try_write() {
                    Ok(state) => state,
                    Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
                    Err(std::sync::TryLockError::WouldBlock) => return Err(Refusal::Busy),
                },
                None => self
                    .state
                    .write()
                    .unwrap_or_else(|error| error.into_inner()),
            };
            active = state.selection.upgrade();
            if active.is_some() {
                return Err(Refusal::Storage(Cause::SourceUnavailable));
            }
            for key in state.counts.keys() {
                if let Some(request) = request {
                    request.consult()?;
                }
                if !selection.members.contains_key(key) {
                    return Err(Refusal::Busy);
                }
            }
            if let Some(request) = request {
                request.consult()?;
            }
            state.selection = Arc::downgrade(&selection);
        }
        drop(active);
        Ok(RetainedMemberPublication { selection })
    }
}
