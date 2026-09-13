use super::{ActiveAttemptResourceLease, ActiveHistoryCustody, ActivePinCustody};
use crate::branch::{ProductBranchHeadProtection, ProductBranchReferenceSnapshot};

impl ActiveAttemptResourceLease<'_> {
    /// Both claims remain guarded until their retag succeeds. The assembled
    /// head returns immediately to the same lease before another step runs.
    pub(super) fn assemble_head(&mut self, snapshot: ProductBranchReferenceSnapshot) {
        let resources = self.resources_mut();
        let ActivePinCustody::Bound(pins) = &mut resources.pins else {
            panic!("head assembly requires bound publication claims")
        };
        let transfer = pins
            .try_transfer_product_head(snapshot.commit().basis())
            .expect("admitted publication claims transfer together");
        resources.pins = ActivePinCustody::TransferredToProduct;
        let ActiveHistoryCustody::Installed(history) = std::mem::replace(
            &mut resources.history_custody,
            ActiveHistoryCustody::TransferredToProduct,
        ) else {
            panic!("head assembly requires installed history protection")
        };
        match ProductBranchHeadProtection::owner_issued(snapshot, transfer, history) {
            Ok(head) => resources.product_head = Some(head),
            Err(failure) => {
                resources.product_head = Some(failure.into_protection());
                panic!("admitted successor custody binds one complete head proof");
            }
        }
    }
}
