use crate::history::ProductHeadHistoryProtectionObligation;
use crate::retention::{
    ProductHeadRetentionObligation, ProductHeadRetentionTransfer, RetentionTransferReceipt,
};

use super::ProductBranchReferenceSnapshot;

/// The complete authority needed to install one product-reference image.
/// Component and immutable-history custody are deliberately one move-only
/// value so a failed CAS cannot strand either half.
#[must_use = "a product reference image must retain its complete head proof"]
pub(crate) struct ProductBranchHeadProtection {
    snapshot: ProductBranchReferenceSnapshot,
    product_head: ProductHeadRetentionObligation,
    product_head_history: ProductHeadHistoryProtectionObligation,
    transfer_receipt: Option<RetentionTransferReceipt>,
}

impl std::fmt::Debug for ProductBranchHeadProtection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductBranchHeadProtection")
            .field("snapshot", &self.snapshot)
            .field("product_head", &self.product_head)
            .field("product_head_history", &self.product_head_history)
            .field("transfer_receipt", &self.transfer_receipt)
            .finish()
    }
}

/// A failed protection admission returns every consumed artifact intact.
pub(crate) struct ProductBranchHeadProtectionAdmissionFailure {
    denial: ProductBranchHeadProtectionDenial,
    protection: ProductBranchHeadProtection,
}

impl ProductBranchHeadProtectionAdmissionFailure {
    #[cfg(test)]
    pub(crate) const fn denial(&self) -> ProductBranchHeadProtectionDenial {
        self.denial
    }

    pub(crate) fn into_protection(self) -> ProductBranchHeadProtection {
        self.protection
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProductBranchHeadProtectionDenial {
    SnapshotOwnerMismatch,
    CommitOwnerMismatch,
    BasisOwnerMismatch,
    ComponentBasisMismatch,
    HistoryOwnerMismatch,
    HistoryCommitMismatch,
    ReceiptOwnerMismatch,
    ReceiptBasisMismatch,
    ReceiptComponentMismatch,
    ReceiptSourceMismatch,
    ReceiptDestinationMismatch,
}

impl ProductBranchHeadProtection {
    /// Admit a transferred product-head pair only when it names the exact
    /// commit occurrence, basis, owner, and transfer provenance of the image.
    pub(crate) fn owner_issued(
        snapshot: ProductBranchReferenceSnapshot,
        transfer: ProductHeadRetentionTransfer,
        product_head_history: ProductHeadHistoryProtectionObligation,
    ) -> Result<Self, ProductBranchHeadProtectionAdmissionFailure> {
        let (product_head, transfer_receipt) = transfer.into_parts();
        let protection = Self {
            snapshot,
            product_head,
            product_head_history,
            transfer_receipt: Some(transfer_receipt),
        };
        match protection.validate() {
            Ok(()) => Ok(protection),
            Err(denial) => Err(ProductBranchHeadProtectionAdmissionFailure { denial, protection }),
        }
    }

    /// Bootstrap receives the product-head component authority directly from
    /// the retention owner. There is no publication transfer receipt for this
    /// first image, so the evidence slot remains intentionally empty.
    pub(crate) fn bootstrap_issued(
        snapshot: ProductBranchReferenceSnapshot,
        product_head: ProductHeadRetentionObligation,
        product_head_history: ProductHeadHistoryProtectionObligation,
    ) -> Result<Self, ProductBranchHeadProtectionAdmissionFailure> {
        let protection = Self {
            snapshot,
            product_head,
            product_head_history,
            transfer_receipt: None,
        };
        match protection.validate() {
            Ok(()) => Ok(protection),
            Err(denial) => Err(ProductBranchHeadProtectionAdmissionFailure { denial, protection }),
        }
    }

    pub(crate) fn snapshot(&self) -> &ProductBranchReferenceSnapshot {
        &self.snapshot
    }

    pub(crate) fn product_head(&self) -> &ProductHeadRetentionObligation {
        &self.product_head
    }

    pub(crate) fn retain_component_pins(
        &mut self,
    ) -> Result<
        crate::retention::RetainedPartialRetentionObligation,
        crate::retention::RetentionTransferDenial,
    > {
        self.product_head.try_transfer_retained()
    }

    pub(crate) fn product_head_history(&self) -> &ProductHeadHistoryProtectionObligation {
        &self.product_head_history
    }

    pub(crate) fn transfer_receipt(&self) -> Option<&RetentionTransferReceipt> {
        self.transfer_receipt.as_ref()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        ProductBranchReferenceSnapshot,
        ProductHeadRetentionObligation,
        ProductHeadHistoryProtectionObligation,
        Option<RetentionTransferReceipt>,
    ) {
        (
            self.snapshot,
            self.product_head,
            self.product_head_history,
            self.transfer_receipt,
        )
    }

    pub(super) fn into_admission_failure(
        self,
        denial: ProductBranchHeadProtectionDenial,
    ) -> ProductBranchHeadProtectionAdmissionFailure {
        ProductBranchHeadProtectionAdmissionFailure {
            denial,
            protection: self,
        }
    }

    pub(super) fn validate(&self) -> Result<(), ProductBranchHeadProtectionDenial> {
        let snapshot_owner = self.snapshot.owner();
        let commit = self.snapshot.commit();
        if self.product_head.owner_identity() != snapshot_owner {
            return Err(ProductBranchHeadProtectionDenial::SnapshotOwnerMismatch);
        }
        if commit.identity().owner_identity() != snapshot_owner {
            return Err(ProductBranchHeadProtectionDenial::CommitOwnerMismatch);
        }
        if commit.basis().owner_identity() != snapshot_owner {
            return Err(ProductBranchHeadProtectionDenial::BasisOwnerMismatch);
        }
        if !self.product_head.matches_basis(commit.basis()) {
            return Err(ProductBranchHeadProtectionDenial::ComponentBasisMismatch);
        }
        if self.product_head_history.owner_identity() != snapshot_owner {
            return Err(ProductBranchHeadProtectionDenial::HistoryOwnerMismatch);
        }
        if !self.product_head_history.matches_commit(commit) {
            return Err(ProductBranchHeadProtectionDenial::HistoryCommitMismatch);
        }
        if let Some(receipt) = &self.transfer_receipt {
            if receipt.owner_identity() != snapshot_owner {
                return Err(ProductBranchHeadProtectionDenial::ReceiptOwnerMismatch);
            }
            if receipt.basis() != commit.basis().identity()
                || receipt.basis() != self.product_head.basis()
            {
                return Err(ProductBranchHeadProtectionDenial::ReceiptBasisMismatch);
            }
            if receipt.relational_key() != self.product_head.relational().key()
                || receipt.signal_key() != self.product_head.signal().key()
            {
                return Err(ProductBranchHeadProtectionDenial::ReceiptComponentMismatch);
            }
            if receipt.source()
                != crate::retention::ComponentBasisDependencyClass::ActivePublicationAttempt
            {
                return Err(ProductBranchHeadProtectionDenial::ReceiptSourceMismatch);
            }
            if receipt.destination()
                != crate::retention::ComponentBasisDependencyClass::ProductBranchHead
            {
                return Err(ProductBranchHeadProtectionDenial::ReceiptDestinationMismatch);
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for ProductBranchHeadProtectionAdmissionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProductBranchHeadProtectionAdmissionFailure")
            .field("denial", &self.denial)
            .field("protection", &self.protection)
            .finish()
    }
}
