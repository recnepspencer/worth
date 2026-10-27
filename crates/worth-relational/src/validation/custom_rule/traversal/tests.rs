//! Real native graph construction and scoped read entry for traversal courts.
use std::sync::Arc;

use crate::facade::identity::{EntityId, KindId, PartitionId};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    KindAspectContractDeclarations, RelationIntegrityDeclarations, RelationKindRegistration,
};
use crate::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, EntityReference, EntitySpec, MutationIntent,
    RelationSpec, WorkerIntentBatch,
};
use crate::symbols::data::ClientKey;
use crate::tests::support::{
    release_test_commit_snapshot, test_owner_begin_transaction_for_main, AspectSchemaFixture,
};
use crate::validation::custom_rule::{CustomInvariantWorkMeter, StructuralRelationView};
use crate::validation::data::CustomInvariantAccessContract;
use crate::validation::engine::{state_view::InvariantStateView, InvariantObservation};
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};

use super::complete_result::*;

mod differential;
mod proposed;
mod raw_roots;
mod stops;

struct Graph {
    runtime: RelationalRuntime,
    entities: Vec<EntityId>,
}

impl Graph {
    fn new(
        execution: &ExecutionRequest<'_, '_>,
        count: usize,
        edges: &[(usize, usize, KindId)],
    ) -> Self {
        Self::with_rules(execution, count, edges, Vec::new())
    }

    fn with_rules(
        execution: &ExecutionRequest<'_, '_>,
        count: usize,
        edges: &[(usize, usize, KindId)],
        rules: Vec<crate::validation::custom_rule::CustomInvariantRegistration>,
    ) -> Self {
        let fixture = AspectSchemaFixture::default();
        let schema = fixture
            .build_registry()
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(3),
                kind_name: "other-edge".into(),
                schema_id: fixture.schema_id,
                schema_version_id: fixture.schema_version_id,
                cross_context_policy: fixture.cross_context_policy,
                cascade_delete_policy: fixture.cascade_delete_policy,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
            .unwrap();
        let builder = rules.into_iter().fold(
            RelationalRuntimeApi::builder().schema_registry(schema),
            |builder, rule| builder.custom_invariant(rule),
        );
        let mut graph = Self {
            runtime: builder.build(),
            entities: Vec::new(),
        };
        graph.add_entities(execution, count);
        let mut batch = WorkerIntentBatch::new("edges");
        for (index, &(source, target, kind_id)) in edges.iter().enumerate() {
            batch
                .intents
                .push(MutationIntent::Create(CreateIntent::Relation(
                    RelationSpec {
                        partition_id: PartitionId::main(),
                        kind_id,
                        client_key: ClientKey::raw(format!("edge-{index}")),
                        source: EntityReference::Existing(graph.entities[source]),
                        target: EntityReference::Existing(graph.entities[target]),
                        fields: AspectFieldPatch::default(),
                    },
                )));
        }
        if !edges.is_empty() {
            let mut txn = test_owner_begin_transaction_for_main(&graph.runtime);
            txn.push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            let outcome = txn.commit(execution, &graph.runtime).unwrap();
            release_test_commit_snapshot(&graph.runtime, &outcome);
        }
        graph
    }

    fn add_entities(&mut self, execution: &ExecutionRequest<'_, '_>, count: usize) {
        let mut txn = test_owner_begin_transaction_for_main(&self.runtime);
        let mut batch = WorkerIntentBatch::new("vertices");
        let creates = (self.entities.len()..self.entities.len() + count)
            .map(|index| CreatedEntityRef {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw(format!("vertex-{index}")),
            })
            .collect::<Vec<_>>();
        for created in &creates {
            batch
                .intents
                .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: created.partition_id,
                    kind_id: created.kind_id,
                    client_key: created.client_key.clone(),
                    fields: AspectFieldPatch::default(),
                })));
        }
        txn.push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let outcome = txn.commit(execution, &self.runtime).unwrap();
        self.entities.extend(
            creates
                .iter()
                .map(|created| outcome.created_entity(created).unwrap()),
        );
        release_test_commit_snapshot(&self.runtime, &outcome);
    }

    fn read<Output>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        work: CustomInvariantWorkMeter,
        read: impl FnOnce(StructuralRelationView<'_>) -> Output,
    ) -> Output {
        let observation =
            InvariantObservation::committed(self.runtime.storage_access().current_edition());
        let state = InvariantStateView::new(
            observation.committed_partition_access(),
            self.runtime.current_version_id(),
        );
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
