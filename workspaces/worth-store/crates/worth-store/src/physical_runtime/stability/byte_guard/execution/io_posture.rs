#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalReadIoPosture {
    OrdinaryReadNoStructuralLatchIo,
}

impl PhysicalReadIoPosture {
    pub const fn ordinary() -> Self {
        Self::OrdinaryReadNoStructuralLatchIo
    }

    pub const fn permits_blocking_io_while_holding_structural_latch(self) -> bool {
        match self {
            Self::OrdinaryReadNoStructuralLatchIo => false,
        }
    }
}

impl Default for PhysicalReadIoPosture {
    fn default() -> Self {
        Self::ordinary()
    }
}
