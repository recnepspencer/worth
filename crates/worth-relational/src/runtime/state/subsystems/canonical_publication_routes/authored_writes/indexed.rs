//! Derive selected-prefix postings only from actual verified canonical owners.
use super::*;
use crate::history::data::CanonicalCommitEnvelope;

pub(super) fn append(
    generation: &mut AuthoredWriteGeneration,
    execution: ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), Denial> {
    let Some(entries) = envelope.authored_index_writes.verified_entries() else {
        crate::history::visit_authored_write_lookup(
            context,
            generation.incomplete_indexes.len(),
            8,
        )?;
        generation
            .incomplete_indexes
            .insert_cloned_native_for_request(&generation.ordinal, &(), execution)?;
        return Ok(());
    };
    let coverage = envelope
        .authored_index_writes
        .coverage()
        .expect("verified entries retain actual catalog coverage");
    if generation
        .index_run_last
        .is_none_or(|prior| !coverage.extends(prior))
    {
        generation.index_run_start = generation.ordinal;
    }
    generation.index_run_last = Some(coverage);
    crate::history::visit_authored_write_lookup(context, generation.index_coverage.len(), 8)?;
    generation.index_coverage.insert_cloned_native_for_request(
        &generation.ordinal,
        &coverage,
        execution,
    )?;
    for entry in entries {
        crate::history::visit_authored_write(context)?;
        for (kind, value) in [
            (entry.old_kind, entry.old_key.as_ref()),
            (entry.new_kind, entry.new_key.as_ref()),
        ] {
            let (Some(kind), Some(value)) = (kind, value) else {
                continue;
            };
            let fields = entry.definition.locator.field_path().fields().len();
            crate::history::visit_authored_write_lookup(context, 0, fields as u64)?;
            let bytes = entry
                .definition
                .owned_bytes()
                .checked_add(value.owned_allocation_capacity_bytes())
                .ok_or(Denial::SizeOverflow)?;
            crate::history::visit_authored_write_lookup(context, 0, bytes)?;
            let _scratch = execution.reserve_retained_memory(bytes)?;
            generation.append(
                execution,
                context,
                &Key::EntityFieldIndex {
                    definition: entry.definition.clone(),
                    kind,
                    value: value.clone(),
                },
                envelope.commit.commit_id,
            )?;
        }
    }
    Ok(())
}
