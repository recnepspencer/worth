use worth_ui_host_contract::{UiMountIncarnation, UiMountedInstanceIdentity};

use super::UiMountedIdentityState;
use crate::mounting::appearance_receipt_basis::UiMountedAppearanceOwnerReceipt;
use crate::mounting::{
    UiMountedAppearanceReceiptBasis, UiMountedAppearanceReceiptBasisDenial,
    UiMountedNodeReceiptBasis,
};

impl UiMountedIdentityState {
    pub(crate) fn seal_appearance_receipt_basis(
        &self,
        instance: UiMountedInstanceIdentity,
        incarnation: UiMountIncarnation,
        successor: &UiMountedNodeReceiptBasis,
    ) -> Result<UiMountedAppearanceReceiptBasis, UiMountedAppearanceReceiptBasisDenial> {
        let owner = self.presented_appearance_owner(instance, incarnation)?;
        if successor.affinity().is_none() {
            return Err(UiMountedAppearanceReceiptBasisDenial::SuccessorWithoutAffinity);
        }
        let successor_receipt = successor
            .receipt_for(instance)
            .ok_or(UiMountedAppearanceReceiptBasisDenial::SuccessorNotPresented)?;
        Ok(UiMountedAppearanceReceiptBasis::from_mounted_authority(
            owner.map_or(
                UiMountedAppearanceOwnerReceipt::NoPresentedOwnerBasis,
                UiMountedAppearanceOwnerReceipt::Presented,
            ),
            successor_receipt,
            instance,
            incarnation,
        ))
    }

    pub(crate) fn validate_appearance_receipt_basis(
        &self,
        basis: UiMountedAppearanceReceiptBasis,
    ) -> Result<(), UiMountedAppearanceReceiptBasisDenial> {
        let owner =
            self.presented_appearance_owner(basis.mounted_instance(), basis.incarnation())?;
        if owner != basis.owner_node_receipt() {
            return Err(UiMountedAppearanceReceiptBasisDenial::OwnerBasisChanged);
        }
        Ok(())
    }

    fn presented_appearance_owner(
        &self,
        instance: UiMountedInstanceIdentity,
        incarnation: UiMountIncarnation,
    ) -> Result<
        Option<worth_ui_host_contract::UiMountedNodeReceiptIdentity>,
        UiMountedAppearanceReceiptBasisDenial,
    > {
        let current = self
            .instances
            .get(&instance)
            .ok_or(UiMountedAppearanceReceiptBasisDenial::CurrentInstanceUnavailable)?;
        if current.basis.mount_incarnation() != incarnation {
            return Err(UiMountedAppearanceReceiptBasisDenial::CurrentIncarnationMismatch);
        }
        let Some(publication) = self.current_publication.as_ref() else {
            // advance_frame establishes candidate identities without presentation.
            return Ok(None);
        };
        let receipts = self
            .current_receipt_basis
            .as_ref()
            .ok_or(UiMountedAppearanceReceiptBasisDenial::MissingCurrentPublication)?;
        if self.current_frame != Some(publication.frame())
            || receipts.frame() != publication.frame()
        {
            return Err(UiMountedAppearanceReceiptBasisDenial::MissingCurrentPublication);
        }
        Ok(receipts.receipt_for(instance))
    }
}
