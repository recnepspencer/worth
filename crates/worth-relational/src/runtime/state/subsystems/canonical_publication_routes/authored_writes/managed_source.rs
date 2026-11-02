//! Optional Source impact postings derive only from the actual frozen envelope.
use super::*;
use crate::history::data::{CanonicalCommitEnvelope, LiveManagedSourceHistory};

pub(super) fn append(
    generation: &mut AuthoredWriteGeneration,
    execution: ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    envelope: &Arc<CanonicalCommitEnvelope>,
) -> Result<(), Denial> {
    let Some(packet) = LiveManagedSourceHistory::verified(envelope) else {
        crate::history::visit_authored_write(context)?;
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
        crate::history::visit_authored_write(context)?;
        generation.append(
            execution,
            context,
            &Key::ManagedSource(*key),
            envelope.commit.commit_id,
        )?;
    }
    Ok(())
}
