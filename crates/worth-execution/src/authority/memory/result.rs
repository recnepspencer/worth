use super::{ExecutionMemoryReservation, MemoryLimitDenial, MemoryLimitLevel};

/// A measured scope lends its original reservation for result allocation.
/// The known input/framework charge cannot be removed through this view.
/// Source owners reserve clone/growth coexistence before allocating and release
/// obsolete backing only after its last holder goes away.
///
/// ```compile_fail
/// use worth_execution::{ExecutionResultMemory, SerialMemoryBudget};
/// let mut held = SerialMemoryBudget::new(None).reserve(0).unwrap();
/// let view = ExecutionResultMemory { reservation: &mut held, base: 0 };
/// ```
pub struct ExecutionResultMemory<'scope> {
    reservation: &'scope mut ExecutionMemoryReservation,
    base: u64,
}

impl<'scope> ExecutionResultMemory<'scope> {
    pub(crate) fn new(reservation: &'scope mut ExecutionMemoryReservation, base: u64) -> Self {
        Self { reservation, base }
    }

    pub fn bytes(&self) -> u64 {
        self.reservation.bytes() - self.base
    }

    pub fn resize(&mut self, bytes: u64) -> Result<(), MemoryLimitDenial> {
        let total = self.base.checked_add(bytes).ok_or_else(|| {
            MemoryLimitDenial::overflow(bytes, self.base, MemoryLimitLevel::Declared)
        })?;
        self.reservation.resize(total)
    }
}
