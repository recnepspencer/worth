//! Original semantic summaries include the actual late Native ownership effects.
use crate::history::data::CanonicalCommitEnvelopeDraft;
use crate::transactions::data::{CommitChangeSummary, CommitLog, TransactionCommitError};

pub(super) fn complete(
    log: &mut CommitLog,
    envelope: &CanonicalCommitEnvelopeDraft,
    changed: &[crate::transactions::data::RecordRef],
    additional_aspects: &[crate::authority::mutation::CanonicalRecordAspectDelta],
    additional_adjacency: usize,
) -> Result<(), TransactionCommitError> {
    let denied = || crate::mvcc::origin_denial("original publication summaries are unavailable");
    let original = log.summary();
    let change = original.change_summary.as_ref().ok_or_else(denied)?;
    let mut aspects = original.aspect_summary.clone().ok_or_else(denied)?;
    let mut publication = original.publication_summary.clone().ok_or_else(denied)?;
    let adjacency_delta_count = change
        .adjacency_delta_count
        .checked_add(additional_adjacency)
        .ok_or_else(|| {
            TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                    crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
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
        crate::publication::patch::data::ordered_aspect_keys(aspects.touched_aspects);
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
