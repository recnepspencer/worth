//! Original semantic summaries include the actual late Native ownership effects.
use contracts::transactions::data::commit_log::CommitChangeSummary;
use contracts::transactions::data::commit_log::CommitLog;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::draft::CanonicalCommitEnvelopeDraft;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn complete(
    log: &mut CommitLog,
    envelope: &CanonicalCommitEnvelopeDraft,
    changed: &[contracts::transactions::data::primitives::RecordRef],
    additional_aspects: &[worth_relational_mutation::internal_port::authority::mutation::canonical_deltas::data::CanonicalRecordAspectDelta],
    additional_adjacency: usize,
) -> Result<(), TransactionCommitError> {
    let denied = || {
        contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
            "original publication summaries are unavailable",
        )
    };
    let original = log.summary();
    let change = original.change_summary.as_ref().ok_or_else(denied)?;
    let mut aspects = original.aspect_summary.clone().ok_or_else(denied)?;
    let mut publication = original.publication_summary.clone().ok_or_else(denied)?;
    let adjacency_delta_count = change
        .adjacency_delta_count
        .checked_add(additional_adjacency)
        .ok_or_else(|| {
            TransactionCommitError::execution(contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                kind: contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                    contracts::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
                ),
                partition_identity: None,
            })
        })?;
    let extra = super::super::super::artifact_execution::preparation::summarize_commit_aspects(
        additional_aspects,
    );
    aspects.changed_entity_aspect_count += extra.changed_entity_aspect_count;
    aspects.changed_relation_aspect_count += extra.changed_relation_aspect_count;
    aspects.opaque_aspect_delta_count += extra.opaque_aspect_delta_count;
    aspects.zero_aspect_structural_delta_count += extra.zero_aspect_structural_delta_count;
    aspects.touched_aspects.extend(extra.touched_aspects);
    aspects.touched_aspects =
        contracts::publication::patch::data::aspect_key_ordering::ordered_aspect_keys(
            aspects.touched_aspects,
        );
    publication.patch_record_count = envelope.patch.authoritative_record_patches.len();
    publication.lineage_event_count = envelope.published_lineage().lineage_events().len();
    log.record_changed_records(&CommitChangeSummary {
        changed_record_count: changed.len(),
        adjacency_delta_count,
    });
    log.record_aspect_summary(&aspects);
    log.record_publication_artifacts(&publication);
    Ok(())
}
