use crate::capability::MosaicStateSlotDescriptor;
use crate::runtime::{WorthUiDurableStateFamilyId, WorthUiDurableStateReconciliationReceipt};

use super::digest::{fold, fold_text};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiStateSlotPlanMeaning {
    owner_identity: String,
    descriptor: MosaicStateSlotDescriptor,
    succession: WorthUiStateSlotSuccession,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiStateSlotSuccession {
    Launch,
    Reconciled(WorthUiDurableStateReconciliationReceipt),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiStateSlotMeaningDenial {
    ForeignOwnerSuccession,
    ForeignFamilySuccession,
}

impl WorthUiStateSlotPlanMeaning {
    pub(crate) fn new(
        owner_identity: String,
        descriptor: MosaicStateSlotDescriptor,
        succession: WorthUiStateSlotSuccession,
    ) -> Result<Self, WorthUiStateSlotMeaningDenial> {
        if let WorthUiStateSlotSuccession::Reconciled(receipt) = &succession {
            let expected_family = durable_family_for_slot(&descriptor);
            if receipt.identity_basis() != owner_identity {
                return Err(WorthUiStateSlotMeaningDenial::ForeignOwnerSuccession);
            }
            if receipt.family_id() != &expected_family {
                return Err(WorthUiStateSlotMeaningDenial::ForeignFamilySuccession);
            }
        }
        Ok(Self {
            owner_identity,
            descriptor,
            succession,
        })
    }

    pub(crate) fn slot_id(&self) -> &str {
        self.descriptor.id().as_str()
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let digest = fold_text(0x7374_6174_6500_0001, &self.owner_identity);
        let digest = fold_text(digest, self.slot_id());
        match &self.succession {
            WorthUiStateSlotSuccession::Launch => fold(digest, 1),
            WorthUiStateSlotSuccession::Reconciled(receipt) => {
                let digest = fold(digest, receipt.outcome() as u64 + 2);
                fold(digest, durable_family_tag(receipt.family_id()))
            }
        }
    }

    pub(crate) fn descriptor(&self) -> &MosaicStateSlotDescriptor {
        &self.descriptor
    }

    pub(crate) fn succession(&self) -> &WorthUiStateSlotSuccession {
        &self.succession
    }

    pub(crate) fn same_mounted_layout_meaning(&self, successor: &Self) -> bool {
        self.owner_identity == successor.owner_identity && self.descriptor == successor.descriptor
    }
}

pub(crate) fn durable_family_for_slot(
    descriptor: &MosaicStateSlotDescriptor,
) -> WorthUiDurableStateFamilyId {
    use crate::capability::MosaicStateSlotKind as Kind;
    match descriptor.kind() {
        Kind::SplitterPosition => WorthUiDurableStateFamilyId::SplitterPosition,
        Kind::ScrollPosition => WorthUiDurableStateFamilyId::ScrollAnchor,
        Kind::FocusedRegion => WorthUiDurableStateFamilyId::FocusChain,
        Kind::SelectionToken => WorthUiDurableStateFamilyId::SelectionRange,
        Kind::DraftInputState => WorthUiDurableStateFamilyId::TextEditBuffer,
        Kind::ActiveStackItem | Kind::ActivePrimarySurface | Kind::ActiveAuxiliarySurface => {
            WorthUiDurableStateFamilyId::TabState
        }
        Kind::RegionVisibility | Kind::CollapsedPosture | Kind::PinnedPosture => {
            WorthUiDurableStateFamilyId::PanelVisibility
        }
    }
}

fn durable_family_tag(family: &WorthUiDurableStateFamilyId) -> u64 {
    match family {
        WorthUiDurableStateFamilyId::FocusChain => 1,
        WorthUiDurableStateFamilyId::ScrollAnchor => 2,
        WorthUiDurableStateFamilyId::SelectionRange => 3,
        WorthUiDurableStateFamilyId::TextEditBuffer => 4,
        WorthUiDurableStateFamilyId::SplitterPosition => 5,
        WorthUiDurableStateFamilyId::TabState => 6,
        WorthUiDurableStateFamilyId::PanelVisibility => 7,
        WorthUiDurableStateFamilyId::Custom(value) => fold_text(8, value),
    }
}
