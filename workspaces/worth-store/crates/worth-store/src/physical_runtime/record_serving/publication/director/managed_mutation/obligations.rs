use crate::physical_runtime::durability::{PendingPublicationLease, PhysicalCurrentRootOwner};
use crate::physical_runtime::{PhysicalMutationIdentity, PhysicalMutationTerminalFact};

pub(super) struct RewriteGrowthGuard<'a> {
    owner: &'a PhysicalCurrentRootOwner,
    identity: PhysicalMutationIdentity,
    committed: bool,
}

impl<'a> RewriteGrowthGuard<'a> {
    pub(super) fn new(
        owner: &'a PhysicalCurrentRootOwner,
        identity: PhysicalMutationIdentity,
    ) -> Self {
        Self {
            owner,
            identity,
            committed: false,
        }
    }

    pub(super) fn commit(&mut self) {
        if self.committed {
            return;
        }
        // Common namespace-durable advance settled exactly this member's
        // candidate and displaced-source obligations.
        self.committed = true;
    }

    pub(super) fn retain_unresolved(&mut self) {
        if self.committed {
            return;
        }
        self.owner
            .retain_unresolved_rewrite_candidate(self.identity);
        self.committed = true;
    }
}

impl Drop for RewriteGrowthGuard<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.owner.release_rewrite_candidate(self.identity);
        }
    }
}

pub(super) fn keep_unresolved(
    mut growth: RewriteGrowthGuard<'_>,
    pending: PendingPublicationLease,
    terminal: PhysicalMutationTerminalFact,
    copy: Option<&super::super::extent_copy::CopyFailureObligation>,
) -> PhysicalMutationTerminalFact {
    let unresolved = matches!(&terminal, PhysicalMutationTerminalFact::Indeterminate(_));
    if unresolved && !copy.is_some_and(|copy| copy.permits_root_release()) {
        growth.retain_unresolved();
        pending.retain_unresolved();
    }
    terminal
}
