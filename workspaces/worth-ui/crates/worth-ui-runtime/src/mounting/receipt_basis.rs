use worth_ui_host_contract::{
    UiMountedContractIdentityExhaustion, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedNodeReceiptAffinity, UiMountedNodeReceiptIdentity, UiMountedNodeReceiptIssuer,
};

#[derive(Clone)]
pub(crate) struct UiMountedNodeReceiptBasis {
    issuer: UiMountedNodeReceiptIssuer,
    presented_instances:
        crate::runtime::persistent_index::UiPersistentOrdSet<UiMountedInstanceIdentity>,
}

impl UiMountedNodeReceiptBasis {
    pub(crate) fn mint(
        frame: UiMountedFrameIdentity,
        presented_instances: crate::runtime::persistent_index::UiPersistentOrdSet<
            UiMountedInstanceIdentity,
        >,
    ) -> Result<Self, UiMountedContractIdentityExhaustion> {
        Ok(Self {
            issuer: UiMountedNodeReceiptIssuer::mint_for(frame)?,
            presented_instances,
        })
    }

    pub(crate) fn frame(&self) -> UiMountedFrameIdentity {
        self.issuer.frame_identity()
    }

    pub(crate) const fn issuer(&self) -> UiMountedNodeReceiptIssuer {
        self.issuer
    }

    pub(crate) fn affinity(&self) -> Option<UiMountedNodeReceiptAffinity> {
        (!self.presented_instances.is_empty()).then(|| self.issuer.receipt_affinity())
    }

    pub(crate) fn receipt_for(
        &self,
        mounted_instance: UiMountedInstanceIdentity,
    ) -> Option<UiMountedNodeReceiptIdentity> {
        self.receipt_for_with_probes(mounted_instance).0
    }

    pub(crate) fn receipt_for_with_probes(
        &self,
        mounted_instance: UiMountedInstanceIdentity,
    ) -> (Option<UiMountedNodeReceiptIdentity>, usize) {
        let (presented, probes) = self
            .presented_instances
            .contains_with_probes(&mounted_instance);
        (
            presented.then(|| self.issuer.receipt_for(mounted_instance)),
            probes,
        )
    }

    pub(crate) fn len(&self) -> usize {
        self.presented_instances.len()
    }

    pub(crate) fn receipts(
        &self,
    ) -> impl Iterator<Item = (UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity)> + '_ {
        self.presented_instances
            .iter()
            .copied()
            .map(|instance| (instance, self.issuer.receipt_for(instance)))
    }

    pub(crate) fn retained_structural_bytes(&self) -> Option<usize> {
        std::mem::size_of::<Self>()
            .checked_add(self.presented_instances.retained_structural_bytes()?)
    }

    pub(crate) fn remove(&mut self, mounted_instance: UiMountedInstanceIdentity) {
        self.presented_instances.remove_with_work(&mounted_instance);
    }
}
