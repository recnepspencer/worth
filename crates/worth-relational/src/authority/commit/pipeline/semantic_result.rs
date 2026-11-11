//! The single original semantic producer; no publication or measured work is recorded.
use crate::history::data::CanonicalCommitEnvelopeBody;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{
    CommitCreatedEntityBindings, CommitCreatedRelationBindings, CommitLog, CommitSchemaSummary,
    CommitStructuralSummary, RecordRef, RelationalCommitPublicationMeaning,
    RelationalCommitSemanticResult, RelationalCommitSemanticResultRefusal as Refusal,
    RelationalCommitStructuralMeaning, TransactionCommitError,
};
use crate::validation::engine::InvariantExecutionResult;
use std::sync::Arc;

pub(super) struct SemanticResultInput<'a> {
    pub(super) structural: &'a CommitStructuralSummary,
    pub(super) validation: &'a [InvariantExecutionResult],
    pub(super) entities: &'a CommitCreatedEntityBindings,
    pub(super) relations: &'a CommitCreatedRelationBindings,
    pub(super) log: &'a CommitLog,
    pub(super) changed: &'a [RecordRef],
    pub(super) envelope: &'a CanonicalCommitEnvelopeBody,
}

pub(super) fn prepare_semantic_result(
    runtime: &RelationalPreparationRuntime,
    input: SemanticResultInput<'_>,
) -> Result<Arc<RelationalCommitSemanticResult>, TransactionCommitError> {
    let denied =
        |refusal| crate::mvcc::origin_denial(format!("original semantic result: {refusal:?}"));
    let original = input.log.summary();
    let history = original
        .history_summary
        .clone()
        .ok_or_else(|| denied(Refusal::MissingOriginalSummary))?;
    let change = original
        .change_summary
        .clone()
        .ok_or_else(|| denied(Refusal::MissingOriginalSummary))?;
    let aspects = original
        .aspect_summary
        .clone()
        .ok_or_else(|| denied(Refusal::MissingOriginalSummary))?;
    let publication = original
        .publication_summary
        .as_ref()
        .ok_or_else(|| denied(Refusal::MissingOriginalSummary))?;
    let (created_entities, created_relations, validation) = runtime.services.symbols.with_read(|symbols| {
        Ok::<_, Refusal>((input.entities.canonical_meaning(symbols)?, input.relations.canonical_meaning(symbols)?,
            input.validation.iter().map(|execution| crate::transactions::data::RelationalInvariantExecutionMeaning::from_original(execution, symbols))
                .collect::<Result<Vec<_>, _>>()?))
    }).map_err(denied)?;
    Ok(Arc::new(RelationalCommitSemanticResult {
        decoded_memory: None,
        receipt: input.envelope.commit.clone(),
        created_entities,
        created_relations,
        changed_records: input.changed.to_vec(),
        structural: RelationalCommitStructuralMeaning {
            invariant_groups: input.structural.invariant_groups,
            commit_topology: input.structural.commit_topology,
            touched_partitions: input.structural.touched_partitions.clone(),
            descriptor_semantics_version: input.structural.descriptor_semantics_version,
        },
        schema: CommitSchemaSummary {
            transition: input
                .envelope
                .schema_transition
                .as_ref()
                .map(crate::schema::data::SchemaTransitionSummary::from_artifact),
            descriptor_semantics_version: input.envelope.descriptor_semantics_version,
        },
        history,
        change,
        aspects,
        publication: RelationalCommitPublicationMeaning {
            patch_record_count: input.envelope.patch.authoritative_record_patches.len(),
            lineage_event_count: publication.lineage_event_count,
            merge_parent_count: publication.merge_parent_count,
        },
        validation,
    }))
}
