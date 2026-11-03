//! Lineage, limits and memory admission of one scope before its kernel may run.
use crate::authority::{
    ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial, ResourceReservation,
    SerialRequest,
};
use crate::backend::meter::RunLimits;

/// Whether the scope admits application payload bytes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ScopeStorage {
    Native,
    Metered,
    /// An accounted payload whose result and error heap are statically empty.
    Unit,
}

/// One scope's admission inputs, free of its payload types.
pub(super) struct ScopeRequest<'a, 'lease> {
    pub(super) lease: Option<&'a ExecutionResourceLease<'lease>>,
    pub(super) retained_bytes: u64,
    pub(super) max_result_bytes: Option<u64>,
    pub(super) charge_parent: bool,
    pub(super) work_ceiling: Option<u64>,
    pub(super) serial: Option<&'a SerialRequest>,
    pub(super) hold: Option<&'a mut ExecutionMemoryReservation>,
    pub(super) storage: ScopeStorage,
    pub(super) result_slot_bytes: usize,
    pub(super) error_slot_bytes: usize,
}

impl ScopeRequest<'_, '_> {
    /// Whether this scope's report carries charged work.
    pub(super) fn is_accounted(&self) -> bool {
        self.storage != ScopeStorage::Native
            || RunLimits::native_accounting(self.lease)
            || self.work_ceiling.is_some()
            || self.serial.is_some_and(|serial| serial.memory().is_some())
    }

    fn limits(&self) -> RunLimits {
        match self.storage {
            ScopeStorage::Native => RunLimits::for_native(self.lease, false),
            ScopeStorage::Metered | ScopeStorage::Unit => RunLimits::for_run(self.lease, false),
        }
        .within_work_ceiling(self.work_ceiling)
        .within_serial(self.serial)
    }

    fn charged_memory_bytes(&self, limits: &RunLimits) -> Option<u64> {
        if !limits.is_accounted() {
            return Some(0);
        }
        u64::try_from(self.result_slot_bytes)
            .ok()
            .and_then(|bytes| bytes.checked_add(u64::try_from(self.error_slot_bytes).ok()?))
            .and_then(|bytes| bytes.checked_add(self.retained_bytes))
            .and_then(|bytes| bytes.checked_add(self.max_result_bytes.unwrap_or(0)))
            .and_then(|bytes| bytes.checked_add(limits.framework_context_bytes(1, 1)?))
    }
}

/// A scope whose worker and memory admission succeeded.
pub(super) struct AdmittedScope<'a, 'lease> {
    pub(super) lease: Option<&'a ExecutionResourceLease<'lease>>,
    pub(super) limits: RunLimits,
    pub(super) memory_bytes: u64,
    pub(super) max_result_bytes: Option<u64>,
    pub(super) charge_parent: bool,
    pub(super) storage: ScopeStorage,
    pub(super) hold: Option<&'a mut ExecutionMemoryReservation>,
    pub(super) reservation: Option<ResourceReservation>,
    pub(super) serial_reservation: Option<ExecutionMemoryReservation>,
}

pub(super) fn admit_scope<'a, 'lease>(
    mut request: ScopeRequest<'a, 'lease>,
) -> Result<AdmittedScope<'a, 'lease>, LeaseDenial> {
    if request.lease.is_none() && RunLimits::has_leased_parent() {
        return Err(LeaseDenial::unrelated_nested_lease());
    }
    let limits = request.limits();
    let memory_bytes = request
        .charged_memory_bytes(&limits)
        .ok_or_else(LeaseDenial::charged_bytes_overflow)?;
    let lease = request.lease;
    // A held scope's memory is the caller's reservation; the entry holds
    // only its worker.
    let entry_bytes = if request.hold.is_some() {
        0
    } else {
        memory_bytes
    };
    let reservation = lease
        .map(|lease| {
            if limits.is_accounted() {
                lease.reserve_entry(entry_bytes)
            } else {
                lease.reserve_worker_entry()
            }
        })
        .transpose()?;
    if let Some(hold) = request.hold.as_deref_mut() {
        hold.take_over(lease, limits.serial_memory(), memory_bytes)?;
    }
    let serial_reservation = match (lease, limits.serial_memory(), &request.hold) {
        (None, Some(budget), None) => Some(
            budget
                .reserve(memory_bytes)
                .map_err(LeaseDenial::MemoryExhausted)?,
        ),
        _ => None,
    };
    Ok(AdmittedScope {
        lease,
        memory_bytes,
        max_result_bytes: request.max_result_bytes,
        charge_parent: request.charge_parent,
        storage: request.storage,
        hold: request.hold,
        reservation,
        serial_reservation,
        limits,
    })
}
