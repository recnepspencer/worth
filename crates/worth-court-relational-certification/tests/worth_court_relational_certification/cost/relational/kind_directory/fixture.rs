//! Two real installed entity kinds sharing one declared native aspect shape.
use crate::content::historical_kind::fixture::World;
use crate::leased_preparation::ordinary_context::support;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    transactions::*,
};

pub(super) fn world() -> World {
    let registry = support::demo_schema_registry();
    let unrelated = worth_relational::facade::schema::EntityKindRegistration {
        kind_id: KindId(3),
        kind_name: "directory.unrelated".into(),
        schema_id: worth_relational::facade::schema::SchemaId("demo".into()),
        schema_version_id: worth_relational::facade::schema::SchemaVersionId(1),
        aspect_contract_declarations:
            worth_relational::facade::schema::KindAspectContractDeclarations::new(vec![
                support::entity_string_field_aspect(
                    support::aspect_key("name"),
                    support::field_key("name"),
                ),
            ]),
    };
    World::with_schema(registry.register_entity_kind(unrelated).unwrap())
}
pub(super) fn create_other(
    world: &mut World,
    request: &ExecutionRequest<'_, '_>,
    partition: u32,
    label: &str,
) -> EntityId {
    let mut spec = World::entity_spec(partition, label);
    spec.kind_id = KindId(3);
    let committed = world
        .commit(request, MutationIntent::Create(CreateIntent::Entity(spec)))
        .unwrap();
    let id = support::changed_entity(&committed).unwrap();
    world.entities.push(id);
    world.release(&committed);
    id
}
pub(super) fn pin(
    world: &World,
    request: &ExecutionRequest<'_, '_>,
) -> worth_relational::facade::runtime::RelationalPinnedRead<()> {
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let view = world
        .runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    world
        .runtime
        .read_truth()
        .pin_view_read_capability(&view, &[], request)
        .unwrap()
}

pub(super) fn populated(request: &ExecutionRequest<'_, '_>, others: usize, spread: bool) -> World {
    let mut result = world();
    result.create(request, 7, "selected-a");
    result.create(request, 7, "selected-b");
    for index in 0..others {
        create_other(
            &mut result,
            request,
            if spread { 100 + index as u32 } else { 7 },
            &format!("other-{index}"),
        );
    }
    result
}
