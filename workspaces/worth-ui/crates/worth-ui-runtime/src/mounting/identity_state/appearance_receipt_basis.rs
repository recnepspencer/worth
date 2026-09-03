use worth_ui_host_contract::{UiMountIncarnation, UiMountedInstanceIdentity};

use super::UiMountedIdentityState;
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
        let publication = self
            .current_publication
            .as_ref()
            .ok_or(UiMountedAppearanceReceiptBasisDenial::MissingCurrentPublication)?;
        let current_receipts = self
            .current_receipt_basis
            .as_ref()
            .ok_or(UiMountedAppearanceReceiptBasisDenial::MissingCurrentPublication)?;
        if self.current_frame != Some(publication.frame())
            || current_receipts.frame() != publication.frame()
        {
            return Err(UiMountedAppearanceReceiptBasisDenial::MissingCurrentPublication);
        }
        let current_instance = self
            .instances
            .get(&instance)
            .ok_or(UiMountedAppearanceReceiptBasisDenial::CurrentInstanceUnavailable)?;
        if current_instance.basis.mount_incarnation() != incarnation {
            return Err(UiMountedAppearanceReceiptBasisDenial::CurrentIncarnationMismatch);
        }
        let owner = current_receipts
            .receipt_for(instance)
            .ok_or(UiMountedAppearanceReceiptBasisDenial::MissingCurrentPublication)?;
        if successor.affinity().is_none() {
            return Err(UiMountedAppearanceReceiptBasisDenial::SuccessorWithoutAffinity);
        }
        let successor_receipt = successor
            .receipt_for(instance)
            .ok_or(UiMountedAppearanceReceiptBasisDenial::SuccessorNotPresented)?;
        Ok(UiMountedAppearanceReceiptBasis::from_mounted_authority(
            owner,
            successor_receipt,
            instance,
            incarnation,
        ))
    }
}
