//! Derive selected-prefix postings only from live-verified or owner-certified
//! canonical evidence; anything else marks the ordinal incomplete.
use super::*;
use branch::history::access::relevant_writes::posting_key::AuthoredWritePostingKey as Key;
use branch::history::access::relevant_writes::selection::RelationalAuthoredEntityIndexDefinitionOwnerPort as _;
use branch::history::access::relevant_writes::selection::RelationalAuthoredWriteSelector as Selector;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_branch::internal_port as branch;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::catalog_publication::CapturedIndexDefinitionCatalog;

pub(super) fn append(
    generation: &mut AuthoredWriteGeneration,
    execution: ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    envelope: &CanonicalCommitEnvelope,
    catalog: Option<&CapturedIndexDefinitionCatalog>,
) -> Result<(), Denial> {
    let writes = envelope.owner_authored_index_writes();
    // Live packets carry their issued coverage; readmitted ones are certified
    // by the receiving catalog owner captured once for this generation.
    let coverage = match (writes.coverage(), catalog) {
        (Some(coverage), _) => Some(coverage),
        (None, Some(catalog)) => catalog.certify_readmitted_writes(
            writes,
            envelope.owner_index_definition_delta().next_index_id,
            context,
        )?,
        (None, None) => None,
    };
    let (Some(coverage), Some((_, entries))) = (coverage, writes.persisted_entries()) else {
        branch::history::access::relevant_writes::ordered_visit(
            context,
            generation.incomplete_indexes.len(),
        )?;
        generation
            .incomplete_indexes
            .insert_cloned_native_for_request(&generation.ordinal, &(), execution)?;
        return Ok(());
    };
    if generation
        .index_run_last
        .is_none_or(|prior| !coverage.extends(prior))
    {
        generation.index_run_start = generation.ordinal;
    }
    generation.index_run_last = Some(coverage);
    branch::history::access::relevant_writes::ordered_visit(
        context,
        generation.index_coverage.len(),
    )?;
    generation.index_coverage.insert_cloned_native_for_request(
        &generation.ordinal,
        &coverage,
        execution,
    )?;
    for entry in entries {
        branch::history::access::relevant_writes::visit(context)?;
        for (kind, value) in [
            (entry.old_kind, entry.old_key.as_ref()),
            (entry.new_kind, entry.new_key.as_ref()),
        ] {
            let (Some(kind), Some(value)) = (kind, value) else {
                continue;
            };
            let bytes = entry
                .definition
                .owned_bytes()
                .checked_add(value.owned_allocation_capacity_bytes())
                .ok_or(Denial::SizeOverflow)?;
            let _scratch = execution.reserve_retained_memory(bytes)?;
            generation.append(
                execution,
                context,
                &Key::Ordinary(Selector::EntityFieldIndex {
                    definition: entry.definition.clone(),
                    kind,
                    value: value.clone(),
                }),
                envelope.commit.commit_id,
            )?;
        }
    }
    Ok(())
}
