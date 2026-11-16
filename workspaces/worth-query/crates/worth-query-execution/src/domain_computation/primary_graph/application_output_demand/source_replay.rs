//! Replay evidence selects existing source custody; it grants no preparation authority.
use super::super::{
    WorthQueryApplicationCommitReceipt, WorthQueryRecoveredApplicationCommitReceipt,
};
#[derive(Clone, Copy)]
pub enum WorthQueryOutputSourceReplay<'receipt> {
    Retained(&'receipt WorthQueryApplicationCommitReceipt),
    Recorded(&'receipt WorthQueryRecoveredApplicationCommitReceipt),
}
impl<'receipt> From<&'receipt WorthQueryApplicationCommitReceipt>
    for WorthQueryOutputSourceReplay<'receipt>
{
    fn from(receipt: &'receipt WorthQueryApplicationCommitReceipt) -> Self {
        Self::Retained(receipt)
    }
}
impl<'receipt> From<&'receipt WorthQueryRecoveredApplicationCommitReceipt>
    for WorthQueryOutputSourceReplay<'receipt>
{
    fn from(receipt: &'receipt WorthQueryRecoveredApplicationCommitReceipt) -> Self {
        Self::Recorded(receipt)
    }
}
impl WorthQueryOutputSourceReplay<'_> {
    pub fn matches_recorded(&self, recorded: &WorthQueryRecoveredApplicationCommitReceipt) -> bool {
        match self {
            Self::Retained(live) => live.matches_recorded_original(recorded),
            Self::Recorded(left) => left.same_recorded_subject_as(recorded),
        }
    }
}
impl WorthQueryRecoveredApplicationCommitReceipt {
    pub fn same_recorded_subject_as(&self, right: &Self) -> bool {
        let left = self;
        left.commit_reference() == right.commit_reference()
            && left.new_product_head() == right.new_product_head()
            && left.old_product_head() == right.old_product_head()
            && left.attempt_identity() == right.attempt_identity()
            && left.native_journal_position() == right.native_journal_position()
            && left.native_artifact_digest() == right.native_artifact_digest()
            && left.world_journal_position() == right.world_journal_position()
            && left.world_artifact_digest() == right.world_artifact_digest()
            && left.publishing_incarnation() == right.publishing_incarnation()
            && left.idempotency_key_identity() == right.idempotency_key_identity()
            && left.outcome_identity() == right.outcome_identity()
            && left.emitted_effect_count() == right.emitted_effect_count()
            && left.exact_output_settlement() == right.exact_output_settlement()
    }
}
#[cfg(test)]
#[path = "source_replay_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
