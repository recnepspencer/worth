//! Actual four-partition ring, optional independently paged incoming relation label.
use super::{
    checkpoint_receiver::reopen,
    region_publications::{create_entity, create_relation_in_partition},
    ring_symbol_publications,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{identity::*, symbols::Symbol};
use worth_testkit_relational::{
    cold_recovery::{capture_source, ColdRecoverySourceFixture},
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};
pub(super) struct RingRecipe {
    original: ColdRecoverySourceFixture,
    entities: Vec<EntityId>,
    remote_symbol: Option<Symbol>,
}
pub(super) struct RingFixture{pub runtime:worth_relational::facade::runtime::RelationalRuntime,pub source:std::sync::Arc<worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource>,pub medium:std::sync::Arc<TestDurableMedium>,pub directory:super::selected_directory::Directory,pub entities:Vec<EntityId>,pub remote_symbol:Option<Symbol>}
impl RingRecipe {
    pub fn new(request: &ExecutionRequest<'_, '_>, paged: bool) -> Self {
        let medium = TestDurableMedium::new();
        let producer = durable_runtime(&medium);
        let mut entities = Vec::new();
        for id in 1..=4 {
            entities.push(create_entity(request, &producer, "source", PartitionId(id)));
            create_entity(request, &producer, "target", PartitionId(id));
        }
        let mut remote_symbol = None;
        for (i, &a) in entities.iter().enumerate() {
            let target = entities[(i + 1) % entities.len()];
            if paged && i == 1 {
                ring_symbol_publications::populate(request, &producer);
            }
            if paged && i + 1 == entities.len() {
                remote_symbol = Some(ring_symbol_publications::publish_edge(
                    request,
                    &producer,
                    a,
                    target,
                    "remote-paged-edge",
                ));
            } else {
                create_relation_in_partition(request, &producer, a, target, "edge", a.partition_id);
            }
        }
        if paged {
            create_entity(request, &producer, "source", PartitionId::main());
        }
        let original = capture_source(request, producer, medium, entities[0], false);
        Self {
            original,
            entities,
            remote_symbol,
        }
    }
    pub fn reopen(&self, request: &ExecutionRequest<'_, '_>) -> RingFixture {
        {
            let reader = reopen(request, &self.original);
            RingFixture {
                runtime: reader.runtime,
                source: reader.source,
                medium: reader.medium,
                directory: reader.directory,
                entities: self.entities.clone(),
                remote_symbol: self.remote_symbol,
            }
        }
    }
}
