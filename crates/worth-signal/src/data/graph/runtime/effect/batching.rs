use crate::logic::evaluation::{EffectComparison, EvaluationEffect};
mod retained_charge;

#[derive(Debug)]
pub(crate) struct ApplyCommitPacket {
    pub(crate) effect: EvaluationEffect,
    pub(crate) comparison: EffectComparison,
    pub(crate) pending_snapshot: Option<crate::logic::evaluation::PendingDependencySnapshot>,
    pub(crate) defer_snapshot_commit: bool,
}

#[derive(Debug)]
pub(crate) struct PreparedParallelApplyCommitPacket(pub(super) ApplyCommitPacket);

impl From<ApplyCommitPacket> for PreparedParallelApplyCommitPacket {
    fn from(packet: ApplyCommitPacket) -> Self {
        Self(packet)
    }
}
