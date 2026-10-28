use std::sync::{Arc, Weak};

use crate::branch::{RelationalBranchIdentity, RelationalBranchRoot};

use super::owner::RelationalBranchRetentionOwnerInner;

/// One owner-issued live-head obligation. The canonical branch coordination
/// cell owns this token; detached reference snapshots never clone it.
#[derive(Debug)]
pub(crate) struct RelationalHeadRetentionObligation {
    owner: Weak<RelationalBranchRetentionOwnerInner>,
    owner_identity: usize,
    identity: RelationalBranchIdentity,
    root: Arc<RelationalBranchRoot>,
    binding: super::RelationalBranchRetentionBinding,
    terminal: bool,
}

impl RelationalHeadRetentionObligation {
    pub(super) fn new(
        owner: &Arc<RelationalBranchRetentionOwnerInner>,
        identity: RelationalBranchIdentity,
        root: Arc<RelationalBranchRoot>,
    ) -> Self {
        let binding = super::RelationalBranchRetentionBinding::new(owner, Some(identity.clone()));
        binding.record_head_install();
        Self {
            owner: Arc::downgrade(owner),
            owner_identity: Arc::as_ptr(owner) as usize,
            identity,
            root,
            binding,
            terminal: false,
        }
    }

    pub(crate) fn binding(&self) -> super::RelationalBranchRetentionBinding {
        self.binding.clone()
    }

    pub(super) fn validate_binding(
        &self,
        owner: &Arc<RelationalBranchRetentionOwnerInner>,
        identity: &RelationalBranchIdentity,
    ) -> Result<(), super::RelationalRetentionAcquisitionDenial> {
        if self.terminal
            || self.owner_identity != Arc::as_ptr(owner) as usize
            || &self.identity != identity
        {
            return Err(super::RelationalRetentionAcquisitionDenial::BindingMismatch);
        }
        Ok(())
    }
    pub(super) fn validate_root(
        &self,
        root: &Arc<RelationalBranchRoot>,
    ) -> Result<(), super::RelationalRetentionAcquisitionDenial> {
        if self.terminal || !Arc::ptr_eq(&self.root, root) {
            return Err(super::RelationalRetentionAcquisitionDenial::BindingMismatch);
        }
        Ok(())
    }
    pub(super) fn is_live(&self) -> bool {
        !self.terminal
    }
    pub(super) fn transfer(
        &mut self,
        owner: &Arc<RelationalBranchRetentionOwnerInner>,
        next_root: &Arc<RelationalBranchRoot>,
    ) {
        self.root = Arc::clone(next_root);
        self.binding.record_head_transfer();
        owner
            .head_transfer_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    pub(super) fn consume(&mut self) {
        self.release();
    }

    fn release(&mut self) {
        if self.terminal {
            return;
        }
        self.terminal = true;
        let Some(owner) = self.owner.upgrade() else {
            return;
        };
        super::owner::release_live_head_capacity(&owner);
    }
}

impl Drop for RelationalHeadRetentionObligation {
    fn drop(&mut self) {
        self.release();
    }
}
