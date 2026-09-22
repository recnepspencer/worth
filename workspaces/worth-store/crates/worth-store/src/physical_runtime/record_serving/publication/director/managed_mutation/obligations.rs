use crate::physical_runtime::durability::{PendingPublicationLease, PhysicalCurrentRootOwner};
use crate::physical_runtime::PhysicalMutationTerminalFact;

pub(super) struct RewriteGrowthGuard<'a> {
    owner: &'a PhysicalCurrentRootOwner,
    committed: bool,
}

impl<'a> RewriteGrowthGuard<'a> {
    pub(super) fn new(owner: &'a PhysicalCurrentRootOwner) -> Self {
        Self {
            owner,
            committed: false,
        }
    }

    pub(super) fn commit(&mut self) {
        if self.committed {
            return;
        }
        self.owner.commit_rewrite_candidate();
        self.committed = true;
    }

    pub(super) fn retain_unresolved(&mut self) {
        if self.committed {
            return;
        }
        self.owner.retain_unresolved_rewrite_candidate();
        self.committed = true;
    }
}

impl Drop for RewriteGrowthGuard<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.owner.release_rewrite_candidate();
        }
    }
}

pub(super) fn keep_unresolved(
    mut growth: RewriteGrowthGuard<'_>,
    pending: PendingPublicationLease,
    terminal: PhysicalMutationTerminalFact,
) -> PhysicalMutationTerminalFact {
    let unresolved = matches!(
        &terminal,
        PhysicalMutationTerminalFact::Indeterminate(_)
    );
    if unresolved {
        growth.retain_unresolved();
        pending.retain_unresolved();
    }
    terminal
}
