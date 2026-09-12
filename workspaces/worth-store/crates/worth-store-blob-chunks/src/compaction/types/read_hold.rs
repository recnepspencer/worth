use worth_store_physical_isolation::PhysicalReadPlanCompletionReceipt;

/// Local read-plan posture; this value does not retain a live Store root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobCompactionReadHold {
    Released(PhysicalReadPlanCompletionReceipt),
    Active(PhysicalReadPlanCompletionReceipt),
}

impl BlobCompactionReadHold {
    pub const fn released(receipt: PhysicalReadPlanCompletionReceipt) -> Self {
        Self::Released(receipt)
    }

    pub const fn active(receipt: PhysicalReadPlanCompletionReceipt) -> Self {
        Self::Active(receipt)
    }

    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active(_))
    }

    pub(crate) const fn released_receipt(self) -> Option<PhysicalReadPlanCompletionReceipt> {
        match self {
            Self::Released(receipt) => Some(receipt),
            Self::Active(_) => None,
        }
    }
}

#[allow(dead_code)]
fn _read_hold_is_part_of_the_boundary(_: BlobCompactionReadHold) {}
