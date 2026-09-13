use super::{FlatSegments, Segment, SegmentedStorage, SegmentedStore};
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageForkCharge as ForkCharge, RetainedStorageForkPreparation,
};

impl<T: Clone + RetainedStorageMeasurement, Id: Clone + RetainedStorageMeasurement>
    RetainedStorageForkPreparation for SegmentedStore<T, Id>
{
    fn prepare_fork_charge(&mut self, work: &mut Preparation) -> Result<ForkCharge, Denial> {
        work.visit()?;
        let Self {
            storage,
            interner,
            id: _,
        } = self;
        let charge = match storage {
            SegmentedStorage::Exclusive(flat) => {
                let source = flat.retained_heap_charge(work)?;
                // Conversion creates an empty appended vector, then forks it.
                // Its explicit empty preparation allocates no backing storage.
                let mut appended =
                    crate::data::persistent_vector::PersistentVector::<Vec<T>>::new();
                let retained = source
                    .checked_add(arc_allocation_charge::<
                        crate::data::retained_storage::RetainedStorageBacking<FlatSegments<T>>,
                    >()?)?
                    .checked_add(appended.prepare_fork_charge(work)?.retained)?;
                ForkCharge::from_charges(source, retained)?
            }
            SegmentedStorage::ForkShared { base, appended } => {
                ForkCharge::unchanged(base.retained_heap_charge(work)?)
                    .checked_add(appended.prepare_fork_charge(work)?)?
            }
        };
        charge.checked_add(interner.prepare_fork_charge(work)?)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::graph::DependencyEdgeStore;
    use crate::data::{aspect::Aspect, dependency::DependencyEdge, handle::NodeId};

    #[test]
    fn forked_edge_segments_charge_base_and_appended_scope_payloads() {
        let mut source = DependencyEdgeStore::default();
        let base = source.insert_from_slice(&[DependencyEdge::whole_partition(
            NodeId::new(0, 0),
            Aspect::new(0),
            "base".repeat(4_096),
        )]);
        let mut retained = source.fork_persistent();
        let appended = retained.insert_from_slice(&[DependencyEdge::whole_partition(
            NodeId::new(1, 0),
            Aspect::new(0),
            "appended".repeat(4_096),
        )]);
        drop(source);
        assert_eq!(retained.get(base).len(), 1);
        assert_eq!(retained.get(appended).len(), 1);
        let mut work = Preparation::new(1_000);
        let charge = retained.retained_heap_charge(&mut work).unwrap();
        assert!(charge.bytes() >= 12 * 4_096);
        assert_eq!(
            retained
                .retained_heap_charge(&mut Preparation::new(work.visits()))
                .unwrap(),
            charge
        );
        assert!(matches!(
            retained.retained_heap_charge(&mut Preparation::new(work.visits() - 1)),
            Err(Denial::WorkExhausted { .. })
        ));
    }
}
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for Segment {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { start: _, len: _ } = self;
        Ok(Charge::ZERO)
    }
}
impl<T: RetainedStorageMeasurement> RetainedStorageMeasurement for FlatSegments<T> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { items, segments } = self;
        items
            .retained_heap_charge(work)?
            .checked_add(segments.retained_heap_charge(work)?)
    }
}
impl<T: Clone + RetainedStorageMeasurement, Id: Clone + RetainedStorageMeasurement>
    RetainedStorageMeasurement for SegmentedStore<T, Id>
{
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            storage,
            interner,
            id: _,
        } = self;
        let charge = match storage {
            SegmentedStorage::Exclusive(flat) => flat.retained_heap_charge(work)?,
            SegmentedStorage::ForkShared { base, appended } => base
                .retained_heap_charge(work)?
                .checked_add(appended.retained_heap_charge(work)?)?,
        };
        charge.checked_add(interner.retained_heap_charge(work)?)
    }
}
