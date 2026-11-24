//! Optional Source impact postings derive only from the actual frozen envelope.
use super::*;
use branch::history::access::relevant_writes::posting_key::AuthoredWritePostingKey as Key;
use branch::history::data::canonical_commit_envelope::live_managed_source_history::LiveManagedSourceHistory;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_branch::internal_port as branch;

pub(super) fn append(
    generation: &mut AuthoredWriteGeneration,
    execution: ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    envelope: &Arc<CanonicalCommitEnvelope>,
) -> Result<(), Denial> {
    let Some(packet) = LiveManagedSourceHistory::verified(envelope) else {
        branch::history::access::relevant_writes::visit(context)?;
        // Selected prefixes are monotone. The greatest missing ordinal suffices
        // for every open source-to-target interval and adds no per-gap tree.
        generation.managed_last_gap = Some(generation.ordinal);
        return Ok(());
    };
    if generation
        .managed_run_catalog
        .as_ref()
        .is_none_or(|catalog| !Arc::ptr_eq(catalog, &packet.catalog))
    {
        generation.managed_run_start = generation.ordinal;
    }
    generation.managed_run_catalog = Some(packet.catalog.clone());
    for (key, _) in packet.impacts.iter() {
        branch::history::access::relevant_writes::visit(context)?;
        generation.append(
            execution,
            context,
            &Key::ManagedSource(*key),
            envelope.commit.commit_id,
        )?;
    }
    Ok(())
}
