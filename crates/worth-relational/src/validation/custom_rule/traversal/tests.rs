//! Native partition construction and scoped reads for structural traversal.
use crate::identity::data::{EntityId, KindId, PartitionId, VersionId};
use crate::storage::overlay::{small_relation_partition::relation_partition, PartitionState};
use crate::validation::custom_rule::{CustomInvariantWorkMeter, StructuralRelationView};
use crate::validation::data::descriptor::CustomInvariantAccessContract;
use crate::validation::engine::state_view::InvariantStateView;
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;

use super::complete_result::*;

mod differential;
mod raw_roots;
mod stops;

struct Graph {
    partitions: BTreeMap<PartitionId, PartitionState>,
    entities: Vec<EntityId>,
    edges: Vec<(usize, usize, KindId)>,
}

impl Graph {
    fn new(count: usize, edges: &[(usize, usize, KindId)]) -> Self {
        let id = PartitionId::main();
        let (partition, entities, _) = relation_partition(id, count, edges);
        Self {
            partitions: BTreeMap::from([(id, partition)]),
            entities,
            edges: edges.to_vec(),
        }
    }

    fn add_entities(&mut self, count: usize) {
        let id = PartitionId::main();
        let (partition, entities, _) =
            relation_partition(id, self.entities.len() + count, &self.edges);
        self.partitions.insert(id, partition);
        self.entities = entities;
    }

    fn read<Output>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        work: CustomInvariantWorkMeter,
        read: impl FnOnce(StructuralRelationView<'_>) -> Output,
    ) -> Output {
        let state = InvariantStateView::new(&self.partitions, VersionId(1));
        read(StructuralRelationView::new(
            state,
            *execution,
            work,
            Arc::new(CustomInvariantAccessContract {
                read_entity_kinds: vec![KindId(1)],
                read_relation_kinds: vec![KindId(2), KindId(3)],
                ..CustomInvariantAccessContract::default()
            }),
        ))
    }
}

fn unbounded() -> CustomInvariantWorkMeter {
    CustomInvariantWorkMeter::new(None)
}

fn visits(walk: &CompleteStructuralWalk) -> Vec<(EntityId, usize)> {
    walk.steps()
        .iter()
        .map(|step| {
            assert_eq!(step.kind, KindId(1));
            (step.entity, step.distance)
        })
        .collect()
}
