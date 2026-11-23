//! Public pin and fixed reached-region read after real cold checkpoint recovery.
use crate::content::historical_kind::fixture::World;
use std::{collections::BTreeSet, sync::Arc};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    durability::*, identity::KindId, runtime::RelationalRuntime, transactions::*,
};
use worth_testkit_relational::maintenance::assert_change_bounded;
mod queries;
use super::cold_checkpoint::{reopen, ColdCheckpointSource as Source};

fn pin(
    runtime: &RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> worth_relational::facade::runtime::RelationalPinnedRead<()> {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let view = runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    runtime
        .read_truth()
        .pin_view_read_capability(&view, &[], request)
        .unwrap()
}
struct ReadWork {
    members: u64,
    pin_members: u64,
    symbol_at_pin: bool,
}
fn fixed_read_work(regions: u32, history: usize, width: usize) -> ReadWork {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let mut world = World::new();
    for name in ["selected", "target-a", "target-b"] {
        world.create(&request, 7, name);
    }
    for index in 0..regions {
        world.create(&request, 100 + index, "unrelated");
    }
    // Force a real symbol-backed field outside the initial dictionary pages.
    let (_, text_basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    for index in 0..512 {
        let bytes = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
            format!("unreached-dictionary-value-{index:04}")
                .into_bytes()
                .into(),
        ));
        world
            .runtime
            .admit_field_text(&request, &text_basis, &bytes)
            .unwrap();
    }
    let input = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
        b"selected native symbol".as_slice().into(),
    ));
    let text = world
        .runtime
        .admit_field_text(&request, &text_basis, &input)
        .unwrap();
    let value = text.field_value_for(&text_basis).unwrap();
    use worth_foundational::facade::{
        AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath,
        ContractValidatedAspectValueView, FieldKey, InternedString, LocatorAuthority,
    };
    let AspectValue::String(InternedString::Symbol(text_symbol)) = &value else {
        panic!("native text owns a symbol");
    };
    let text_symbol = worth_relational::facade::symbols::Symbol(text_symbol.0);
    let fields = AspectFieldPatch::from_locator(
        AspectFieldLocator::new(
            LocatorAuthority::Planned,
            AspectKey::new("name").unwrap(),
            CanonicalFieldPath::single(FieldKey::new("name").unwrap()),
        ),
        value,
    );
    let updated = world
        .commit(
            &request,
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                UpdateEntityFieldsIntent {
                    entity_id: world.entities[0],
                    fields,
                },
            )),
        )
        .unwrap();
    world.release(&updated);
    world.relate(&request, 0, 1, 7, "selected-edge");
    let relation = world.relations[0];
    for step in 0..history {
        let update = MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
            UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: KindId(2),
                source: EntityReference::Existing(world.entities[0]),
                target: EntityReference::Existing(world.entities[1 + step % 2]),
            },
        ));
        let committed = world.commit(&request, update).unwrap();
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
    let source = Source::capture(&world.runtime, &request, &checkpoint);
    let runtime = reopen(
        &request,
        &checkpoint,
        Arc::clone(&source),
        World::new().runtime,
    );
    assert!(
        source.region_read_count() == 0,
        "recovery leaves record regions cold"
    );
    // Recovery reads checkpoint metadata (the index catalog); count only reads after it.
    let recovered = source.reads.lock().unwrap().len() as u64;
    let read = pin(&runtime, &request);
    let pin_members = source.reads.lock().unwrap().len() as u64 - recovered;
    let selected = world.entities[0];
    let symbol_at_pin = read.symbols().resolve(text_symbol).is_some();
    let actual = read
        .project_read()
        .read_exact_entity_state(&request, selected, |metadata, state| {
            let ContractValidatedAspectValueView::Scalar(AspectValue::String(
                InternedString::Symbol(symbol),
            )) = state
                .unwrap()
                .get(&AspectKey::new("name").unwrap())
                .unwrap()
                .view()
            else {
                panic!("selected field keeps its symbol-backed native value");
            };
            (metadata.entity_id, symbol.0)
        })
        .unwrap();
    assert_eq!(actual, Some((selected, text_symbol.0)));
    assert_eq!(
        read.symbols().resolve(text_symbol),
        Some("selected native symbol")
    );
    assert_eq!(
        read.with_symbol_text(&request, text_symbol, |text| text
            .unwrap()
            .as_str()
            .to_owned())
            .unwrap(),
        "selected native symbol"
    );
    let frontier = std::iter::once(selected)
        .chain(world.entities.iter().skip(3).take(width - 1).copied())
        .collect::<BTreeSet<_>>();
    let relations = read
        .bounded_outgoing_relations_for_frontier(&request, &frontier, KindId(2), Some(1024))
        .unwrap()
        .into_records();
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].relation_id, relation);
    let work = source.reads.lock().unwrap().len() as u64 - recovered;
    assert!(work > 0, "the fixed read must actually cross cold storage");
    let again = read
        .project_read()
        .read_exact_entity_state(&request, selected, |m, _| m.entity_id)
        .unwrap();
    assert_eq!(again, Some(selected));
    assert_eq!(
        source.reads.lock().unwrap().len() as u64 - recovered,
        work,
        "accepted region is reused"
    );
    ReadWork {
        members: work,
        pin_members,
        symbol_at_pin,
    }
}
#[test]
fn lazy_pin_and_fixed_read_do_not_grow_with_unrelated_regions_or_history() {
    let model = (fixed_read_work(8, 8, 1), fixed_read_work(16, 8, 1));
    let history = (fixed_read_work(4, 16, 1), fixed_read_work(4, 32, 1));
    eprintln!(
        "SDK fixed read cold member calls: model {}->{}; history {}->{}",
        model.0.members, model.1.members, history.0.members, history.1.members
    );
    assert_change_bounded(
        "lazy pin and reached regions",
        "model",
        model.0.members,
        model.1.members,
    );
    assert_change_bounded(
        "lazy pin and reached regions",
        "history",
        history.0.members,
        history.1.members,
    );
    for evidence in [model.0, model.1, history.0, history.1] {
        assert_eq!(evidence.pin_members, 0, "pinning does no I/O");
        assert!(
            !evidence.symbol_at_pin,
            "pinning does not load field symbol pages"
        );
    }
}

#[test]
fn wide_frontier_does_not_escalate_to_unreached_regions() {
    let base = fixed_read_work(64, 4, 33);
    let doubled = fixed_read_work(128, 4, 33);
    eprintln!(
        "SDK fixed wide frontier member calls: {}->{}",
        base.members, doubled.members
    );
    assert_change_bounded(
        "wide frontier residency",
        "unreached regions",
        base.members,
        doubled.members,
    );
    assert_eq!(
        base.pin_members, doubled.pin_members,
        "neither pin performs I/O"
    );
}
