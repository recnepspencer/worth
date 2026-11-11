//! Original branch publication has one concrete owner effect, never a fake transaction.
use super::prepared::SignalPreparedDurableAdvance;
use super::SignalDurableTransactionResult;
use crate::branch::{
    owner_services::owner::fork_reservation::SignalPreparedOwnedFork, SignalBranchForkOutcome,
};
use crate::state::SignalBranchId;

pub(super) enum PreparedSignalOwnerEffect<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    Advanced(Box<SignalPreparedDurableAdvance<D, I, T>>),
    Forked(Box<SignalPreparedOwnedFork<D, I, T>>),
}
#[derive(Clone, Copy)]
pub(super) enum ReservedSignalBranches {
    Advanced(SignalBranchId),
    Forked {
        source: SignalBranchId,
        target: SignalBranchId,
    },
}
impl ReservedSignalBranches {
    pub(super) fn kind(self) -> worth_foundational::facade::SignalBranchEffectKind {
        use worth_foundational::facade::SignalBranchEffectKind as Kind;
        match self {
            Self::Advanced(_) => Kind::Advanced,
            Self::Forked { .. } => Kind::Forked,
        }
    }

    pub(super) fn target(self) -> SignalBranchId {
        match self {
            Self::Advanced(branch) => branch,
            Self::Forked { target, .. } => target,
        }
    }
    pub(super) fn identities(self) -> impl Iterator<Item = SignalBranchId> {
        match self {
            Self::Advanced(branch) => [Some(branch), None],
            Self::Forked { source, target } => [Some(source), Some(target)],
        }
        .into_iter()
        .flatten()
    }
    pub(super) fn count(self) -> usize {
        match self {
            Self::Advanced(_) => 1,
            Self::Forked { .. } => 2,
        }
    }
}

pub enum SignalDurableOwnerEffect {
    Advanced(SignalDurableTransactionResult),
    Forked(SignalDurableForkResult),
}
pub struct SignalDurableForkResult {
    pub(super) outcome: SignalBranchForkOutcome,
    pub(super) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl SignalDurableOwnerEffect {
    pub fn kind(&self) -> worth_foundational::facade::SignalBranchEffectKind {
        use worth_foundational::facade::SignalBranchEffectKind as Kind;
        match self {
            Self::Advanced(_) => Kind::Advanced,
            Self::Forked(_) => Kind::Forked,
        }
    }
}
impl SignalDurableForkResult {
    pub fn result(&self) -> &SignalBranchForkOutcome {
        &self.outcome
    }
}
