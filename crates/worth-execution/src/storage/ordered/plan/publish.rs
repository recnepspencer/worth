//! Materialize reachable planned cells from one owned path admission.
use super::*;
use crate::storage::ordered::funding::{FundingSource, PathFunding};
impl<K, V> Plan<'_, '_, K, V> {
    fn new_cells(&self, link: PlannedLink<'_, K, V>) -> Result<u64, Denial> {
        let PlannedLink::New(index) = link else {
            return Ok(0);
        };
        let node = self.slots[index].expect("initialized plan");
        self.new_cells(node.left)?
            .checked_add(self.new_cells(node.right)?)
            .and_then(|n| n.checked_add(1))
            .ok_or(Denial::CapacityOverflow)
    }
    pub(in crate::storage::ordered) fn admission(
        &self,
        root: PlannedLink<'_, K, V>,
        entry_bytes: u64,
        source: &dyn FundingSource,
    ) -> Result<PathFunding, Denial> {
        let bytes = if source.uses_accounting() {
            self.new_cells(root)?
                .checked_mul(node::arc_bytes::<Node<K, V>>()?)
                .and_then(|n| n.checked_add(entry_bytes))
                .ok_or(Denial::CapacityOverflow)?
        } else {
            0
        };
        PathFunding::admit(source, bytes)
    }
    pub(in crate::storage::ordered) fn materialize(
        &self,
        root: PlannedLink<'_, K, V>,
        inserted: Option<&Arc<Entry<K, V>>>,
        pool: &PathFunding,
    ) -> Result<Link<K, V>, Denial> {
        match root {
            PlannedLink::Empty => Ok(None),
            PlannedLink::Existing(node) => Ok(Some(Arc::clone(node))),
            PlannedLink::New(index) => {
                let plan = self.slots[index].expect("initialized plan");
                let left = self.materialize(plan.left, inserted, pool)?;
                let right = self.materialize(plan.right, inserted, pool)?;
                let entry = plan
                    .entry
                    .or(inserted)
                    .expect("insertion carries its sealed entry");
                node::build(Arc::clone(entry), left, right, pool).map(Some)
            }
        }
    }
}
