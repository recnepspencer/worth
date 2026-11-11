//! Replace only the authentic current node; preserve the original ancestry/index cut.
use super::*;

impl PerformedPublicationStream {
    pub(in super::super) fn is_recovered_latest(
        &self,
        source: &PositionedCanonicalCommit,
        route: &Arc<CanonicalPublicationRoute>,
    ) -> bool {
        self.head.load_full().is_some_and(|head| {
            head.position == source.position() && Arc::ptr_eq(&head.route, route)
        })
    }

    pub(in super::super) fn replace_recovered_latest(
        &self,
        source: &PositionedCanonicalCommit,
        previous: &Arc<CanonicalPublicationRoute>,
        route: Arc<CanonicalPublicationRoute>,
    ) -> Result<(), &'static str> {
        let _reservation = self
            .reservation
            .enter(PatchPositionReservationAdmission::DeferWhenHeld)
            .map_err(|_| "recovered source stream reservation is contended")?;
        let current = self
            .head
            .load_full()
            .ok_or("recovered source has no actual latest stream node")?;
        if current.position != source.position() || !Arc::ptr_eq(&current.route, previous) {
            return Err("recovered source does not match the actual latest stream node");
        }
        let replacement = Arc::new(PerformedPublicationNode {
            index_definition_frontier: current.index_definition_frontier,
            position: current.position,
            route,
            previous: current.previous.clone(),
        });
        let mut index = self.index.lock().unwrap_or_else(|error| error.into_inner());
        if index
            .by_patch
            .get(&current.position)
            .is_some_and(|node| Arc::ptr_eq(node, &current))
        {
            index.by_patch.insert(current.position, replacement.clone());
        }
        if index
            .indexed_head
            .as_ref()
            .is_some_and(|node| Arc::ptr_eq(node, &current))
        {
            index.indexed_head = Some(replacement.clone());
        }
        self.head.store(Some(replacement));
        Ok(())
    }
}
