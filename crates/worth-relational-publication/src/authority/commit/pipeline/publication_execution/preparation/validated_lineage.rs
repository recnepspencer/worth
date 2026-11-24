//! The validated lineage batch a publication installs: its reserved events
//! and its decoded canonical group lineage frame.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelopeBody, CanonicalCommitEnvelopeBodyOwnerPort as _,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::state::ValidatedLineageEventBatch;

use contracts::publication::bundle::publication_status::PublicationStage;
use contracts::publication::data::publication_error::PublicationError;
use contracts::transactions::data::commit_log::CommitLog;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

/// Complexity: O(events + group lineage frame bytes).
pub(in crate::internal_port::authority::commit::pipeline::publication_execution) fn validated_lineage_events(
    envelope: &CanonicalCommitEnvelopeBody,
    commit_log: &CommitLog,
) -> Result<ValidatedLineageEventBatch, TransactionCommitError> {
    let lineage = envelope.published_lineage();
    ValidatedLineageEventBatch::from_reserved(lineage.lineage_events().to_vec())
        .with_group_lineage(&envelope.branch_context, lineage.group_lineage())
        .map_err(|refusal| {
            TransactionCommitError::publication(PublicationError::new(
                PublicationStage::BundleAssembly,
                format!("canonical group lineage frame refused: {refusal:?}"),
            ))
            .with_commit_log(commit_log.clone())
        })
}
