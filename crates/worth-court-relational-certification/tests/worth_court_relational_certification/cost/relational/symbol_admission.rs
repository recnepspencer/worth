//! Real cold SDK commits inherit only the changed record's dictionary closure.
use super::cold_checkpoint::{reopen, ColdCheckpointSource};
use crate::content::historical_kind::fixture::World;
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath,
    ContractValidatedAspectValueView, FieldKey, InternedString, LocatorAuthority,
};
use worth_relational::facade::{
    durability::*, inspection::observe_maintenance_work, transactions::*,
};
use worth_testkit_relational::maintenance::assert_change_bounded;

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

fn commit_work(model: usize, history: usize, full: bool) -> u64 {
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
    let (committed, observation) = observe_maintenance_work(512, || world.commit(&request, edit));
    let committed = committed.unwrap();
    assert_eq!(observation.omitted_phases, 0);
    let phases: Vec<_> = observation
        .phases
        .iter()
        .filter(|row| row.phase == "symbol_admission")
        .collect();
    assert!(
        !phases.is_empty(),
        "real cold commit enters dictionary admission"
    );
    let work: u64 = phases
        .iter()
        .map(|row| {
            assert!(row.accounted);
            row.charged_work
        })
        .sum();
    assert!(work > 0, "changed symbol requirements are charged");
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
    work
}
#[test]
fn renewed_cell_symbol_admission_is_flat_in_model() {
    for full in [false, true] {
        let (small, large) = (commit_work(128, 8, full), commit_work(256, 8, full));
        println!("SDK successor symbol admission model full={full}: {small}->{large}");
        assert_change_bounded("SDK symbol admission", "model", small, large);
    }
}
#[test]
fn renewed_cell_symbol_admission_is_flat_in_history() {
    for full in [false, true] {
        let (small, large) = (commit_work(16, 512, full), commit_work(16, 1024, full));
        println!("SDK successor symbol admission history full={full}: {small}->{large}");
        assert_change_bounded("SDK symbol admission", "history", small, large);
    }
}

fn after_bulk_work(records: usize, native: bool) -> u64 {
    use worth_relational::facade::{
        identity::{KindId, PartitionId},
        symbols::ClientKey,
    };
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let mut world = World::new();
    world.create(&request, 7, "fixed target");
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
    let mut world = World {
        runtime,
        entities: world.entities,
        relations: vec![],
    };
    let warm = world
        .commit(&request, intent(&world, &request, "warm target"))
        .unwrap();
    world.release(&warm);
    let bulk = world
        .commit(
            &request,
            MutationIntent::Create(CreateIntent::BulkEntities(BulkEntityCreateIntent {
                partition_id: PartitionId(7),
                kind_id: KindId(1),
                client_keys: (0..records)
                    .map(|index| ClientKey::raw(format!("bulk-{index}")))
                    .collect(),
                field_patches: (0..records)
                    .map(|index| {
                        if native {
                            field_patch(&world, &request, &format!("bulk-native-{index}"))
                        } else {
                            World::entity_spec(7, &format!("bulk-{index}")).fields
                        }
                    })
                    .collect(),
            })),
        )
        .unwrap();
    world.entities.extend(
        bulk.changed_records
            .iter()
            .filter_map(|record| match record {
                RecordRef::Entity(id) => Some(*id),
                _ => None,
            }),
    );
    world.release(&bulk);
    let edit = intent(&world, &request, "fixed after bulk");
    let (committed, observation) = observe_maintenance_work(512, || world.commit(&request, edit));
    world.release(&committed.unwrap());
    assert_eq!(observation.omitted_phases, 0);
    observation
        .phases
        .iter()
        .filter(|row| row.phase == "symbol_admission")
        .map(|row| {
            assert!(row.accounted);
            row.charged_work
        })
        .sum()
}
#[test]
fn a_fixed_edit_does_not_pay_the_previous_large_edit_again() {
    for native in [false, true] {
        let (small, large) = (after_bulk_work(128, native), after_bulk_work(256, native));
        println!("SDK fixed edit after previous bulk native={native}: {small}->{large}");
        assert_change_bounded("SDK symbol admission", "previous change", small, large);
    }
}
