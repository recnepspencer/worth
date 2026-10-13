use super::InvariantStateView;
use crate::identity::data::{EntityId, PartitionId, RelationId};
use crate::storage::overlay::PartitionState;
use std::cell::RefCell;
use std::ops::ControlFlow;
use std::sync::Arc;

struct TouchedSlots<'state> {
    partition_id: PartitionId,
    partition: &'state PartitionState,
    slots: Arc<[usize]>,
}

impl<'state> InvariantStateView<'state> {
    fn touched_partitions(&self, visit: &impl Fn(usize) -> bool) -> Option<Arc<[PartitionId]>> {
        match &self.candidate_inputs {
            Some((inputs, basis)) => inputs.touched_partitions(
                *basis,
                self.version_id,
                || {
                    source_count(
                        |callback| self.state.visit_touched_partition_ids(callback),
                        visit,
                    )
                },
                |ids| {
                    source_fill(
                        |callback| self.state.visit_touched_partition_ids(callback),
                        ids,
                        visit,
                    )
                },
            ),
            None => self.state.touched_partition_ids().map(Arc::from),
        }
    }

    fn touched_slots(
        &self,
        partition: PartitionId,
        entities: bool,
        visit: &impl Fn(usize) -> bool,
    ) -> Option<Arc<[usize]>> {
        let source = |callback: &mut dyn FnMut(usize) -> ControlFlow<()>| {
            if entities {
                self.state.visit_touched_entity_slots(partition, callback)
            } else {
                self.state.visit_touched_relation_slots(partition, callback)
            }
        };
        match &self.candidate_inputs {
            Some((inputs, basis)) => {
                let count = || source_count(source, visit);
                let fill = |slots: &mut Vec<usize>| source_fill(source, slots, visit);
                if entities {
                    inputs.touched_entity_slots(*basis, self.version_id, partition, count, fill)
                } else {
                    inputs.touched_relation_slots(*basis, self.version_id, partition, count, fill)
                }
            }
            None => if entities {
                self.state.touched_entity_slots(partition)
            } else {
                self.state.touched_relation_slots(partition)
            }
            .map(Arc::from),
        }
    }

    fn with_touched_sources<R>(
        &self,
        entities: bool,
        mut charge: impl FnMut(usize) -> bool,
        mut visit: impl FnMut(usize) -> bool,
        read: impl FnOnce(&[TouchedSlots<'state>], &dyn Fn(usize) -> bool) -> Option<R>,
    ) -> Option<R> {
        let visits = RefCell::new(&mut visit);
        let visit = |units| visits.borrow_mut()(units);
        let partition_ids = self.touched_partitions(&visit)?;
        let collect = |sources: &mut Vec<TouchedSlots<'state>>| {
            for partition_id in partition_ids.iter().copied() {
                if !visit(0) {
                    return None;
                }
                let partition = self.state.get_partition(partition_id)?;
                let slots = self.touched_slots(partition_id, entities, &visit);
                if !visit(0) {
                    return None;
                }
                let Some(slots) = slots else { continue };
                if !charge(slots.len()) {
                    return None;
                }
                sources.push(TouchedSlots {
                    partition_id,
                    partition,
                    slots,
                });
            }
            if sources.is_empty() {
                return None;
            }
            read(sources, &visit)
        };
        match &self.candidate_inputs {
            Some((inputs, _)) => inputs
                .with_temporary_vec(partition_ids.len(), collect)
                .flatten(),
            None => collect(&mut Vec::with_capacity(partition_ids.len())),
        }
    }

    pub(crate) fn touched_visible_entity_ids_with_budget(
        &self,
        charge: impl FnMut(usize) -> bool,
        visit: impl FnMut(usize) -> bool,
    ) -> Option<Arc<[EntityId]>> {
        self.with_touched_sources(true, charge, visit, |sources, visit| {
            let count = sources.iter().map(|source| source.slots.len()).sum();
            let fill = |ids: &mut Vec<EntityId>| {
                for source in sources {
                    for slot in source.slots.iter().copied() {
                        if !visit(1) {
                            return false;
                        }
                        let Some(metadata) = source
                            .partition
                            .entity_arena
                            .metadata_history_at(slot)
                            .and_then(|history| self.visible_entity_metadata(history))
                        else {
                            continue;
                        };
                        ids.push(EntityId::new(
                            source.partition_id,
                            slot as u64,
                            metadata.generation,
                        ));
                    }
                }
                true
            };
            match &self.candidate_inputs {
                Some((inputs, basis)) => {
                    inputs.touched_entities(*basis, self.version_id, count, fill)
                }
                None => {
                    let mut ids = Vec::with_capacity(count);
                    fill(&mut ids).then(|| Arc::from(ids))
                }
            }
        })
    }

    pub(crate) fn touched_visible_relation_ids_with_budget(
        &self,
        charge: impl FnMut(usize) -> bool,
        visit: impl FnMut(usize) -> bool,
    ) -> Option<Arc<[RelationId]>> {
        self.with_touched_sources(false, charge, visit, |sources, visit| {
            let count = sources.iter().map(|source| source.slots.len()).sum();
            let fill = |ids: &mut Vec<RelationId>| {
                for source in sources {
                    for slot in source.slots.iter().copied() {
                        if !visit(1) {
                            return false;
                        }
                        if let Some(metadata) = self.relation_metadata_at(
                            &source.partition.relation_arena,
                            source.partition_id,
                            slot,
                        ) {
                            ids.push(metadata.relation_id);
                        }
                    }
                }
                true
            };
            match &self.candidate_inputs {
                Some((inputs, basis)) => {
                    inputs.touched_relations(*basis, self.version_id, count, fill)
                }
                None => {
                    let mut ids = Vec::with_capacity(count);
                    fill(&mut ids).then(|| Arc::from(ids))
                }
            }
        })
    }
}

fn source_count<T>(
    source: impl FnOnce(&mut dyn FnMut(T) -> ControlFlow<()>) -> bool,
    visit: &impl Fn(usize) -> bool,
) -> Option<Option<usize>> {
    let mut count = 0;
    let mut failed = false;
    let present = source(&mut |_| {
        if !visit(1) {
            failed = true;
            return ControlFlow::Break(());
        }
        count += 1;
        ControlFlow::Continue(())
    });
    (!failed).then_some(present.then_some(count))
}

fn source_fill<T>(
    source: impl FnOnce(&mut dyn FnMut(T) -> ControlFlow<()>) -> bool,
    values: &mut Vec<T>,
    visit: &impl Fn(usize) -> bool,
) -> bool {
    let mut failed = false;
    source(&mut |value| {
        if !visit(1) {
            failed = true;
            return ControlFlow::Break(());
        }
        values.push(value);
        ControlFlow::Continue(())
    });
    !failed
}
