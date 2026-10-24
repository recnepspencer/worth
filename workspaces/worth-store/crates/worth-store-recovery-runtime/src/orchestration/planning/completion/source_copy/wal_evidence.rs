use super::{PlanningContext, ResolvedPlanningBasis};
use sha2::{Digest, Sha256};
use worth_store_physical_format::{PhysicalExtentCopyRecord, PhysicalExtentCopyResolutionKind};

pub(super) fn verify(
    context: &PlanningContext,
    basis: &ResolvedPlanningBasis,
) -> Result<std::collections::BTreeSet<([u8; 32], u64)>, ()> {
    let cutoff = context.selection.covered_wal_end_exclusive();
    let mut intents = std::collections::BTreeMap::new();
    for (range, payload) in basis.sample.extent_copy_frames() {
        if range.end_exclusive().get() != range.start().get().checked_add(1).ok_or(())? {
            return Err(());
        }
        let record = PhysicalExtentCopyRecord::decode(payload, context.authority.record_format)
            .map_err(|_| ())?;
        match record {
            PhysicalExtentCopyRecord::Intent(intent) => {
                let digest: [u8; 32] = Sha256::digest(payload).into();
                if intents
                    .insert(
                        intent.operation(),
                        (intent, range.start().get(), digest, None),
                    )
                    .is_some()
                {
                    return Err(());
                }
            }
            PhysicalExtentCopyRecord::Resolved(resolution) => {
                // A checkpoint covering both frames releases the copy
                // obligation, so WAL reclamation may retire the Intent's
                // segment while the covered Resolved stays in the retained
                // first segment: only a covered orphan is skipped.
                let Some((_, lsn, digest, prior)) = intents.get_mut(&resolution.operation()) else {
                    if range.end_exclusive().get() <= cutoff {
                        continue;
                    } else {
                        return Err(());
                    }
                };
                if *lsn != resolution.intent_lsn()
                    || *digest != resolution.intent_digest()
                    || resolution.intent_lsn() >= range.start().get()
                    || prior.is_some()
                {
                    return Err(());
                }
                *prior = Some(resolution.kind());
            }
        }
    }
    let mut published = std::collections::BTreeSet::new();
    for copy in basis.redo.source_copies() {
        let recipe = copy.recipe();
        let (intent, lsn, digest, resolution) = intents.get(&copy.operation()).ok_or(())?;
        if *intent != recipe.intent()
            || *lsn != recipe.intent_lsn()
            || *digest != recipe.intent_digest()
            || *lsn >= copy.publication_lsn()
        {
            return Err(());
        }
        match resolution {
            Some(PhysicalExtentCopyResolutionKind::Cancelled) => return Err(()),
            Some(PhysicalExtentCopyResolutionKind::Published {
                root_generation,
                publication_lsn,
            }) => {
                if *publication_lsn != copy.publication_lsn()
                    || Some(*root_generation)
                        != copy.projection().source_root_generation().checked_add(1)
                {
                    return Err(());
                }
                published.insert((copy.operation(), copy.publication_lsn()));
            }
            _ => {}
        }
    }
    Ok(published)
}
