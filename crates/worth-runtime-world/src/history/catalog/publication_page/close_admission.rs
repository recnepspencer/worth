//! Close uses the same catalog frontier without acquiring delivery custody.
use super::*;
use crate::lifecycle::RuntimeWorldCloseDenial as Denial;

impl CompositeHistoryCatalog {
    pub(crate) fn close_frontier(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RuntimeWorldPublicationFrontier, Denial> {
        let mut after = None;
        let mut frontier: Option<RuntimeWorldPublicationFrontier> = None;
        loop {
            request.check_live().map_err(Denial::Stopped)?;
            let state = self.state.try_lock().map_err(|_| Denial::Busy)?;
            let image = frontier.get_or_insert_with(|| RuntimeWorldPublicationFrontier {
                owner: state.owner,
                catalog_affinity: self.allocation_affinity(),
                revision: state.publication_revision.current(),
                upper_bound: state.inspection_order.last().cloned(),
            });
            if image.revision == u64::MAX || image.revision != state.publication_revision.current()
            {
                return Err(Denial::PublicationFrontierChanged);
            }
            let next = state
                .inspection_order
                .range((
                    after.as_ref().map_or(Bound::Unbounded, Bound::Excluded),
                    Bound::Unbounded,
                ))
                .next();
            let Some(identity) = next else {
                return Ok(frontier.expect("frontier"));
            };
            if state
                .entries
                .get(identity)
                .and_then(|slot| slot.get())
                .and_then(|entry| entry.publication.as_ref())
                .is_some_and(|publication| publication.awaits_delivery())
            {
                return Err(Denial::UndeliveredPublication);
            }
            after = Some(identity.clone());
        }
    }

    pub(crate) fn try_validate_close_frontier(
        &self,
        frontier: &RuntimeWorldPublicationFrontier,
    ) -> Result<(), Denial> {
        let state = self.state.try_lock().map_err(|_| Denial::Busy)?;
        if frontier.owner != state.owner
            || frontier.catalog_affinity != self.allocation_affinity()
            || frontier.revision == u64::MAX
            || frontier.revision != state.publication_revision.current()
        {
            return Err(Denial::PublicationFrontierChanged);
        }
        Ok(())
    }
}
