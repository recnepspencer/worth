#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum C7DurabilityCrashSeam {
    BeforeWalAppend,
    DuringWalAppendPrefix,
    AfterWalWriteBeforeBarrier,
    AfterWalBarrierBeforeDataDispatch,
    DuringDataWritePrefix,
    AfterDataSettlementBeforeRootPublication,
    AfterRootReplacementBeforeNamespaceDurability,
    AfterPhysicalDurabilityBeforeAcknowledgment,
}

impl C7DurabilityCrashSeam {
    pub const ALL: [Self; 8] = [
        Self::BeforeWalAppend,
        Self::DuringWalAppendPrefix,
        Self::AfterWalWriteBeforeBarrier,
        Self::AfterWalBarrierBeforeDataDispatch,
        Self::DuringDataWritePrefix,
        Self::AfterDataSettlementBeforeRootPublication,
        Self::AfterRootReplacementBeforeNamespaceDurability,
        Self::AfterPhysicalDurabilityBeforeAcknowledgment,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::BeforeWalAppend => "before-wal-append",
            Self::DuringWalAppendPrefix => "during-wal-append-prefix",
            Self::AfterWalWriteBeforeBarrier => "after-wal-write-before-barrier",
            Self::AfterWalBarrierBeforeDataDispatch => "after-wal-barrier-before-data-dispatch",
            Self::DuringDataWritePrefix => "during-data-write-prefix",
            Self::AfterDataSettlementBeforeRootPublication => {
                "after-data-settlement-before-root-publication"
            }
            Self::AfterRootReplacementBeforeNamespaceDurability => {
                "after-root-replacement-before-namespace-durability"
            }
            Self::AfterPhysicalDurabilityBeforeAcknowledgment => {
                "after-physical-durability-before-acknowledgment"
            }
        }
    }

    pub const fn interrupts_unsettled_media_effect(self) -> bool {
        !matches!(
            self,
            Self::AfterDataSettlementBeforeRootPublication
                | Self::AfterPhysicalDurabilityBeforeAcknowledgment
        )
    }

    pub fn parse(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|seam| seam.label() == label)
    }
}
