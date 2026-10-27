use std::sync::{Arc, Mutex};

use crate::branch::{RelationalBranchIdentity, RelationalBranchRoot};

/// Branch-local owner cell for the one live-head retention token. This cell is
/// carried beside publication coordination; it is not a currentness source.
#[derive(Debug)]
pub(crate) struct RelationalBranchHeadRetentionCell {
    obligation: Mutex<Option<super::RelationalHeadRetentionObligation>>,
}

impl RelationalBranchHeadRetentionCell {
    pub(crate) fn fresh() -> Arc<Self> {
        Arc::new(Self {
            obligation: Mutex::new(None),
        })
    }

    pub(crate) fn install(
        &self,
        obligation: super::RelationalHeadRetentionObligation,
    ) -> Result<(), super::RelationalHeadRetentionObligation> {
        let mut slot = self
            .obligation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if slot.is_some() {
            return Err(obligation);
        }
        *slot = Some(obligation);
        Ok(())
    }

    pub(crate) fn binding(
        &self,
    ) -> Result<super::RelationalBranchRetentionBinding, super::RelationalRetentionAcquisitionDenial>
    {
        self.obligation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(super::RelationalHeadRetentionObligation::binding)
            .ok_or(super::RelationalRetentionAcquisitionDenial::OwnerUnavailable)
    }

    pub(crate) fn reset(&self, obligation: super::RelationalHeadRetentionObligation) {
        let previous = self
            .obligation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .replace(obligation);
        drop(previous);
    }

    pub(crate) fn transfer(
        &self,
        owner_identity: usize,
        identity: &RelationalBranchIdentity,
        previous_root: &Arc<RelationalBranchRoot>,
        next_root: &Arc<RelationalBranchRoot>,
    ) {
        self.obligation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_mut()
            .expect(/* Proof: A live head cell is constructed with its obligation; only the consuming retirement transition takes it, after excluding further live operations. */ "live branch owns its head retention obligation")
            .transfer(owner_identity, identity, previous_root, next_root);
    }

    pub(crate) fn consume(
        &self,
        owner_identity: usize,
        identity: &RelationalBranchIdentity,
        previous_root: &Arc<RelationalBranchRoot>,
    ) {
        let obligation = self
            .obligation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
            .expect(/* Proof: A live head cell is constructed with its obligation; only the consuming retirement transition takes it, after excluding further live operations. */ "live branch owns its head retention obligation");
        obligation.consume(owner_identity, identity, previous_root);
    }
}
