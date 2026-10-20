//! The query packet owner reserves native clones before growing any buffer.
use super::QueryReadPacketDenial;
use std::sync::Mutex;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelFailure};

pub(in crate::visibility::materialization::read_records::reader) struct QueryPacketMemory {
    pub(super) ceiling: Option<u64>,
    scratch: Option<Mutex<ExecutionMemoryReservation>>,
    result: Option<Mutex<ExecutionMemoryReservation>>,
}
impl QueryPacketMemory {
    pub(super) fn declared(ceiling: u64) -> Self {
        Self {
            ceiling: Some(ceiling),
            scratch: None,
            result: None,
        }
    }
    pub(super) fn tracked(
        lease: Option<&ExecutionResourceLease<'_>>,
    ) -> Result<Self, worth_execution::LeaseDenial> {
        Ok(Self {
            ceiling: None,
            scratch: Some(Mutex::new(ExecutionMemoryReservation::reserve_in_scope(
                lease, 0,
            )?)),
            result: Some(Mutex::new(ExecutionMemoryReservation::reserve_in_scope(
                lease, 0,
            )?)),
        })
    }
    pub(super) fn add(
        &self,
        charged: u64,
        additional: u64,
    ) -> Result<u64, MapKernelFailure<QueryReadPacketDenial>> {
        let total = charged.checked_add(additional).ok_or_else(|| {
            if self.ceiling.is_none() {
                MapKernelFailure::Domain(QueryReadPacketDenial::ReadRequest(
                    crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                ))
            } else {
                MapKernelFailure::ResultCapacityExceeded
            }
        })?;
        if self.ceiling.is_some_and(|ceiling| total > ceiling) {
            return Err(MapKernelFailure::ResultCapacityExceeded);
        }
        Ok(total)
    }
    pub(super) fn scratch(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        reserve(&self.scratch, bytes)
    }
    pub(super) fn settle_scratch(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        if let Some(ticket) = &self.scratch {
            ticket
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .resize(bytes)
                .map_err(|denial| {
                    MapKernelFailure::Domain(QueryReadPacketDenial::ReadRequest(
                        crate::execution::denial::lease_denial(
                            worth_execution::LeaseDenial::MemoryExhausted(denial),
                        ),
                    ))
                })?;
        }
        Ok(())
    }
    pub(super) fn result(&self, bytes: u64) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        let heap = bytes.saturating_sub(std::mem::size_of::<super::QueryWorkerFragment>() as u64);
        reserve(&self.result, heap)
    }
    /// Retire backing only after its original vector has been dropped.
    pub(super) fn settle_result(
        &self,
        bytes: u64,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        let heap = bytes.saturating_sub(std::mem::size_of::<super::QueryWorkerFragment>() as u64);
        if let Some(ticket) = &self.result {
            ticket
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .resize(heap)
                .map_err(|denial| {
                    MapKernelFailure::Domain(QueryReadPacketDenial::ReadRequest(
                        crate::execution::denial::lease_denial(
                            worth_execution::LeaseDenial::MemoryExhausted(denial),
                        ),
                    ))
                })?;
        }
        Ok(())
    }
    pub(super) fn capture_result(
        &self,
        heap: u64,
    ) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
        reserve(&self.result, heap)
    }
}
fn reserve(
    ticket: &Option<Mutex<ExecutionMemoryReservation>>,
    bytes: u64,
) -> Result<(), MapKernelFailure<QueryReadPacketDenial>> {
    if let Some(ticket) = ticket {
        let mut ticket = ticket
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if bytes > ticket.bytes() {
            ticket.resize(bytes).map_err(|denial| {
                MapKernelFailure::Domain(QueryReadPacketDenial::ReadRequest(
                    crate::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(denial),
                    ),
                ))
            })?;
        }
    }
    Ok(())
}
