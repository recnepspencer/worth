// Payload admission, replacement, and transfer retain one ledger charge.
pub(crate) use {crate::memory_reservation::ordinary_context::authority, crate::memory_reservation::ordinary_context::request, worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel, crate::memory_reservation::ordinary_context::PROCESS_BYTES, crate::memory_reservation::ordinary_context::SERIAL};
pub(crate) use {std::cell::Cell};
pub(crate) use {worth_execution::ExecutionMemoryReservation, worth_execution::ExecutionResourceLease, worth_execution::LeaseDenial};

pub(crate) struct Payload {
    pub(crate) backing: Vec<u8>,
    pub(crate) ticket: ExecutionMemoryReservation,
}

pub(crate) fn payload(
    lease: &ExecutionResourceLease<'_>,
    bytes: usize,
    allocations: &Cell<usize>,
) -> Result<Payload, MemoryLimitDenial> {
    let ticket = lease.reserve_memory(u64::try_from(bytes).unwrap())?;
    // No payload allocation is attempted until the actual ledger admits it.
    allocations.set(allocations.get() + 1);
    let mut backing = Vec::new();
    backing.try_reserve_exact(bytes).unwrap();
    assert_eq!(
        backing.capacity(),
        bytes,
        "check actual admitted Vec backing"
    );
    backing.resize(bytes, 7);
    Ok(Payload { backing, ticket })
}