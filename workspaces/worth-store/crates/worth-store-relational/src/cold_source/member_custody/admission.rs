//! Prepare all actual leaves and owning slots before the custody transaction.
use super::*;
use worth_execution::{ExecutionAllocationPolicy, ExecutionOrderedMapDenial};

struct PreparedOwner {
    members: Vec<Member>,
    retired: Vec<Cell>,
    cells: Vec<Option<Cell>>,
    memory: Option<ExecutionMemoryReservation>,
    _preparation_memory: Option<ExecutionMemoryReservation>,
}
fn reserve(
    request: Option<&ExecutionRequest<'_, '_>>,
    bytes: usize,
) -> Result<Option<ExecutionMemoryReservation>, Refusal> {
    request
        .map(|request| {
            request
                .reserve_retained_memory(bytes as u64)
                .map_err(Refusal::from)
        })
        .transpose()
        .map(Option::flatten)
}
fn allocation(cause: ExecutionOrderedMapDenial) -> Refusal {
    match cause {
        ExecutionOrderedMapDenial::Request(cause) => Refusal::Request(cause.into()),
        _ => Refusal::Storage(Cause::MemoryExhausted),
    }
}
fn prepare(
    members: &[Member],
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<PreparedOwner, Refusal> {
    let roster = std::alloc::Layout::array::<Member>(members.len())
        .map_err(|_| Refusal::Bounds)?
        .size();
    let retired = std::alloc::Layout::array::<Cell>(members.len())
        .map_err(|_| Refusal::Bounds)?
        .size();
    let memory = reserve(request, roster.checked_add(retired).ok_or(Refusal::Bounds)?)?;
    let mut selected = Vec::new();
    selected
        .try_reserve_exact(members.len())
        .map_err(|_| Refusal::Storage(Cause::MemoryExhausted))?;
    selected.extend_from_slice(members);
    selected.sort_unstable();
    selected.dedup();
    let mut retired_cells = Vec::new();
    retired_cells
        .try_reserve_exact(members.len())
        .map_err(|_| Refusal::Storage(Cause::MemoryExhausted))?;
    let bytes = std::alloc::Layout::array::<Option<Cell>>(selected.len())
        .map_err(|_| Refusal::Bounds)?
        .size();
    let preparation_memory = reserve(request, bytes)?;
    let mut cells = Vec::new();
    cells
        .try_reserve_exact(selected.len())
        .map_err(|_| Refusal::Storage(Cause::MemoryExhausted))?;
    for member in &selected {
        cells.push(Some(
            match request {
                Some(request) => Cell::prepare_for_request(*member, 1, *request),
                None => Cell::prepare_in(*member, 1, ExecutionAllocationPolicy::SystemAllocation),
            }
            .map_err(allocation)?,
        ));
    }
    Ok(PreparedOwner {
        members: selected,
        retired: retired_cells,
        cells,
        memory,
        _preparation_memory: preparation_memory,
    })
}
impl MemberCustodyRegistry {
    pub(super) fn admit(
        &self,
        members: &[Member],
        serving: Arc<ServingPhysicalRuntime>,
        request: Option<&ExecutionRequest<'_, '_>>,
        validate: impl FnOnce(&[Member]) -> Result<(), Refusal>,
    ) -> Result<MemberCustodyOwner, Refusal> {
        let mut prepared = prepare(members, request)?;
        // Declared outside the state scope: a possible final frozen inventory
        // and all unused prepared cells are destroyed only outside the gate.
        let frozen;
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
            frozen = state.selection.upgrade();
            if frozen.as_ref().is_some_and(|frozen| {
                prepared
                    .members
                    .iter()
                    .any(|member| !frozen.members.contains_key(member))
            }) {
                return Err(Refusal::Storage(Cause::MissingMember));
            }
            validate(&prepared.members)?;
            if prepared
                .members
                .iter()
                .any(|member| state.counts.get(member) == Some(&usize::MAX))
            {
                return Err(Refusal::Storage(Cause::MemoryExhausted));
            }
            let new = prepared
                .members
                .iter()
                .filter(|member| !state.counts.contains_key(member))
                .count();
            state.counts.len().checked_add(new).ok_or(Refusal::Bounds)?;
            if let Some(request) = request {
                request.consult()?;
            }
            // No allocations, request polls, external callbacks or fallible
            // admissions occur after the first count change.
            for (index, member) in prepared.members.iter().enumerate() {
                if let Some(count) = state.counts.get(member).copied() {
                    state.counts.set_existing_unique(member,count+1)
                        .expect(/* Proof: counts is private and NEVER cloned. Only these unique Copy ports ever create/edit its nodes and entries; no detached cell retains children or exposes an Arc. The same exclusive gate also preflighted every count and total capacity. */ "private count storage stays unique")
                        .expect("existing count remains present under the same gate");
                } else {
                    let cell = prepared.cells[index]
                        .take()
                        .expect("one original prepared leaf for this unique roster key");
                    state.counts.insert_vacant_prepared(cell)
                        .expect(/* Proof: The private never-cloned unique-count invariant and preflighted cardinality cover SDK uniqueness/capacity; lookup under this same exclusive gate selected this exact key vacant, and its prepared singleton carries that key. */ "admitted original leaf installs without failure");
                }
            }
        }
        drop(frozen);
        Ok(MemberCustodyOwner {
            serving: Some(serving),
            state: self.state.clone(),
            members: prepared.members,
            next: 0,
            retired_cells: prepared.retired,
            _memory: prepared.memory,
        })
    }
}
