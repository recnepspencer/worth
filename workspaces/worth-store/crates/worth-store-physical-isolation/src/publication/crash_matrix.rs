use super::{
    ExecutedPublicationRecoveryReceipt, PublicationCrashStage, RecoveredPublicationStructure,
    RecoveredPublicationStructureKind,
};
use super::{PhysicalPublicationDenial, PhysicalPublicationReceipt};

#[derive(Debug, Clone, Copy)]
pub struct PublicationCrashRecoveryOutcome {
    recovery_receipt: ExecutedPublicationRecoveryReceipt,
    recovered: RecoveredPublicationStructure,
    mixed_tree: bool,
}

impl PublicationCrashRecoveryOutcome {
    pub fn admit_recovery_receipt(
        receipt: &PhysicalPublicationReceipt,
        recovery_receipt: ExecutedPublicationRecoveryReceipt,
    ) -> Result<Self, PhysicalPublicationDenial> {
        let recovered = bind_recovery_receipt_to_publication(receipt, recovery_receipt)?;
        if recovered.kind() == RecoveredPublicationStructureKind::MixedOldAndNewStructure {
            return Err(PhysicalPublicationDenial::MixedTreeAfterCrash);
        }
        Ok(Self {
            recovery_receipt,
            recovered,
            mixed_tree: false,
        })
    }

    pub const fn reject_mixed_tree_attempt() -> PhysicalPublicationDenial {
        PhysicalPublicationDenial::MixedTreeAfterCrash
    }

    pub const fn recovery_receipt(self) -> ExecutedPublicationRecoveryReceipt {
        self.recovery_receipt
    }

    pub const fn stage(self) -> PublicationCrashStage {
        self.recovery_receipt.stage()
    }

    pub const fn recovered(self) -> RecoveredPublicationStructure {
        self.recovered
    }

    pub const fn mixed_tree(self) -> bool {
        self.mixed_tree
    }
}

fn bind_recovery_receipt_to_publication(
    receipt: &PhysicalPublicationReceipt,
    recovery_receipt: ExecutedPublicationRecoveryReceipt,
) -> Result<RecoveredPublicationStructure, PhysicalPublicationDenial> {
    match recovery_receipt.recovered_kind() {
        RecoveredPublicationStructureKind::OldStableStructure => Ok(
            RecoveredPublicationStructure::old_stable_for_publication_admission(
                receipt.old_root().epoch().get(),
                receipt.old_root().manifest_epoch().get(),
            ),
        ),
        RecoveredPublicationStructureKind::NewStableStructure => Ok(
            RecoveredPublicationStructure::new_stable_for_publication_admission(
                receipt.new_root().epoch().get(),
                receipt.new_root().manifest_epoch().get(),
            ),
        ),
        RecoveredPublicationStructureKind::MixedOldAndNewStructure => {
            Err(PhysicalPublicationDenial::MixedTreeAfterCrash)
        }
    }
}
