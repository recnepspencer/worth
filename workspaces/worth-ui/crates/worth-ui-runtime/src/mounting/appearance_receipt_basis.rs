use worth_ui_host_contract::{
    UiMountIncarnation, UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceReceiptBasis {
    owner: UiMountedAppearanceOwnerReceipt,
    successor_node_receipt: UiMountedNodeReceiptIdentity,
    mounted_instance: UiMountedInstanceIdentity,
    incarnation: UiMountIncarnation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::mounting) enum UiMountedAppearanceOwnerReceipt {
    NoPresentedOwnerBasis,
    Presented(UiMountedNodeReceiptIdentity),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedAppearanceReceiptBasisDenial {
    MissingCurrentPublication,
    CurrentInstanceUnavailable,
    CurrentIncarnationMismatch,
    SuccessorNotPresented,
    SuccessorWithoutAffinity,
    OwnerBasisChanged,
}

impl UiMountedAppearanceReceiptBasis {
    pub(in crate::mounting) const fn from_mounted_authority(
        owner: UiMountedAppearanceOwnerReceipt,
        successor_node_receipt: UiMountedNodeReceiptIdentity,
        mounted_instance: UiMountedInstanceIdentity,
        incarnation: UiMountIncarnation,
    ) -> Self {
        Self {
            owner,
            successor_node_receipt,
            mounted_instance,
            incarnation,
        }
    }

    pub(crate) const fn owner_node_receipt(self) -> Option<UiMountedNodeReceiptIdentity> {
        match self.owner {
            UiMountedAppearanceOwnerReceipt::NoPresentedOwnerBasis => None,
            UiMountedAppearanceOwnerReceipt::Presented(receipt) => Some(receipt),
        }
    }

    pub(crate) const fn successor_node_receipt(self) -> UiMountedNodeReceiptIdentity {
        self.successor_node_receipt
    }

    pub(crate) const fn mounted_instance(self) -> UiMountedInstanceIdentity {
        self.mounted_instance
    }

    pub(crate) const fn incarnation(self) -> UiMountIncarnation {
        self.incarnation
    }
}
