//! A live head cell lends checked custody before a retirement can have effects.
use super::owner::RelationalBranchRetentionOwnerInner;
use super::{RelationalHeadRetentionObligation, RelationalRetentionAcquisitionDenial};
use crate::branch::{RelationalBranchIdentity, RelationalBranchRoot};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(crate) struct RelationalBranchHeadRetentionCell {
    obligation: Mutex<Option<Arc<Mutex<RelationalHeadRetentionObligation>>>>,
}
/// Pinned native obligation; only `reserve` can check and construct it.
#[derive(Debug)]
pub(super) struct CheckedHeadObligation {
    obligation: Arc<Mutex<RelationalHeadRetentionObligation>>,
    owner: Arc<RelationalBranchRetentionOwnerInner>,
}
impl RelationalBranchHeadRetentionCell {
    pub(crate) fn fresh() -> Arc<Self> {
        Arc::new(Self {
            obligation: Mutex::new(None),
        })
    }
    pub(crate) fn install(
        &self,
        obligation: RelationalHeadRetentionObligation,
    ) -> Result<(), RelationalHeadRetentionObligation> {
        let mut slot = self
            .obligation
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if slot.is_some() {
            return Err(obligation);
        }
        *slot = Some(Arc::new(Mutex::new(obligation)));
        Ok(())
    }
    pub(crate) fn binding(
        &self,
    ) -> Result<super::RelationalBranchRetentionBinding, RelationalRetentionAcquisitionDenial> {
        let slot = self
            .obligation
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let obligation = slot
            .as_ref()
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if !obligation.is_live() {
            return Err(RelationalRetentionAcquisitionDenial::OwnerUnavailable);
        }
        Ok(obligation.binding())
    }
    pub(crate) fn reset(&self, obligation: RelationalHeadRetentionObligation) {
        let previous = self
            .obligation
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .replace(Arc::new(Mutex::new(obligation)));
        drop(previous);
    }
    pub(super) fn reserve(
        &self,
        owner: &Arc<RelationalBranchRetentionOwnerInner>,
        identity: &RelationalBranchIdentity,
        root: &Arc<RelationalBranchRoot>,
    ) -> Result<CheckedHeadObligation, RelationalRetentionAcquisitionDenial> {
        let obligation = self
            .obligation
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .cloned()
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?;
        obligation
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .validate_binding(owner, identity, root)?;
        Ok(CheckedHeadObligation {
            obligation,
            owner: owner.clone(),
        })
    }
}
impl CheckedHeadObligation {
    pub(super) fn transfer(self, next_root: &Arc<RelationalBranchRoot>) {
        self.obligation
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .transfer(&self.owner, next_root);
    }
    pub(super) fn consume(self) {
        self.obligation
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .consume();
    }
}
