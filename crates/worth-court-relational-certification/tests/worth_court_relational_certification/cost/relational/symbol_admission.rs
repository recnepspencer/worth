//! Real cold SDK commits inherit only the changed record's dictionary closure.
use super::cold_checkpoint::{reopen, ColdCheckpointSource};
use crate::content::historical_kind::fixture::World;
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath,
    ContractValidatedAspectValueView, FieldKey, InternedString, LocatorAuthority,
};
use worth_relational::facade::{durability::*, transactions::*};

fn field_patch(world: &World, request: &ExecutionRequest<'_, '_>, value: &str) -> AspectFieldPatch {
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let bytes = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
        value.as_bytes().to_vec().into_boxed_slice(),
    ));
    let text = world
        .runtime
        .admit_field_text(request, &basis, &bytes)
        .unwrap();
    AspectFieldPatch::from_locator(
        AspectFieldLocator::new(
            LocatorAuthority::Planned,
            AspectKey::new("name").unwrap(),
            CanonicalFieldPath::single(FieldKey::new("name").unwrap()),
        ),
        text.field_value_for(&basis).unwrap(),
    )
}
fn intent(world: &World, request: &ExecutionRequest<'_, '_>, value: &str) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id: world.entities[0],
            fields: field_patch(world, request, value),
        },
    ))
}

fn commit_value(model: usize, history: usize, full: bool) {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let mut world = World::new();
    for index in 0..model {
        world.create(&request, 7, &format!("model-{index}"));
    }
    for step in 0..history {
        let committed = world
            .commit(
                &request,
                intent(&world, &request, &format!("history-{step}")),
            )
            .unwrap();
        world.release(&committed);
    }
    let checkpoint = world
        .runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let source = ColdCheckpointSource::capture(&world.runtime, &request, &checkpoint);
    let runtime = reopen(&request, &checkpoint, source, World::new().runtime);
    let world = World {
        runtime,
        entities: world.entities,
        relations: world.relations,
    };
    if full {
        let (_, observed) = world
            .runtime
            .observe_branch(&world.runtime.main_branch_identity())
            .unwrap();
        let handle = world
            .runtime
            .snapshots()
            .snapshot_for_observation(&observed.observation())
            .unwrap();
        let rows = world
            .runtime
            .read_truth()
            .read_snapshot(&handle, &request)
            .unwrap()
            .unwrap();
        assert_eq!(rows.entities().len(), model);
        drop(rows);
        world.runtime.snapshots().release_snapshot(&handle).unwrap();
    }
    // First contact with the untrusted cold region is its trust boundary. The
    // next commits must carry that accepted closure instead of collecting it.
    let warm = world
        .commit(&request, intent(&world, &request, "warm after restore"))
        .unwrap();
    world.release(&warm);
    let edit = intent(&world, &request, "measured next value");
    let committed = world.commit(&request, edit).unwrap();
    let pin = world
        .runtime
        .read_truth()
        .pin_read_capability(&committed.snapshot, &[], &request)
        .unwrap();
    let symbol = pin
        .project_read()
        .read_exact_entity_state(&request, world.entities[0], |_, state| {
            let ContractValidatedAspectValueView::Scalar(AspectValue::String(
                InternedString::Symbol(id),
            )) = state
                .unwrap()
                .get(&AspectKey::new("name").unwrap())
                .unwrap()
                .view()
            else {
                panic!("actual Native symbol field");
            };
            worth_relational::facade::symbols::Symbol(id.0)
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        pin.with_symbol_text(&request, symbol, |text| text
            .map(|text| text.as_str().to_owned()))
            .unwrap()
            .as_deref(),
        Some("measured next value")
    );
    world.release(&committed);
}
#[test]
fn renewed_cell_symbol_values_survive_model_reopen() {
    for full in [false, true] {
        commit_value(128, 8, full);
        commit_value(256, 8, full);
    }
}
#[test]
fn renewed_cell_symbol_values_survive_history_reopen() {
    for full in [false, true] {
        commit_value(16, 512, full);
        commit_value(16, 1024, full);
    }
}
