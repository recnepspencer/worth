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

    fn require_binding(
        &self,
        owner_identity: usize,
        identity: &RelationalBranchIdentity,
        root: &Arc<RelationalBranchRoot>,
    ) {
        assert!(!self.terminal);
        assert_eq!(self.owner_identity, owner_identity);
        assert_eq!(&self.identity, identity);
        assert!(Arc::ptr_eq(&self.root, root));
    }

    pub(crate) fn transfer(
        &mut self,
        owner_identity: usize,
        identity: &RelationalBranchIdentity,
        previous_root: &Arc<RelationalBranchRoot>,
        next_root: &Arc<RelationalBranchRoot>,
    ) {
        self.require_binding(owner_identity, identity, previous_root);
        self.root = Arc::clone(next_root);
        self.binding.record_head_transfer();
        self.owner
            .upgrade()
            .expect(/* Proof: The enclosing operation retains the strong retention-owner handle for the entire guard lifetime; this Weak refers to that same owner. */ "live head obligation retains an available owner")
            .head_transfer_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub(crate) fn consume(
        mut self,
        owner_identity: usize,
        identity: &RelationalBranchIdentity,
        previous_root: &Arc<RelationalBranchRoot>,
    ) {
        self.require_binding(owner_identity, identity, previous_root);
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
