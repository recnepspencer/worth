#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalReadIoAttempt {
    blocking_storage_io: bool,
    structural_latch_held: bool,
}

impl PhysicalReadIoAttempt {
    pub const fn blocking_storage_io_while_structural_latch_held() -> Self {
        Self {
            blocking_storage_io: true,
            structural_latch_held: true,
        }
    }

    pub const fn local_nonblocking_byte_access() -> Self {
        Self {
            blocking_storage_io: false,
            structural_latch_held: false,
        }
    }

    pub const fn requires_declared_structural_latch_io_cost(self) -> bool {
        self.blocking_storage_io && self.structural_latch_held
    }
}
