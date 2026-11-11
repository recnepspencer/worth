//! An untouched executable binding does not require an unrecorded revision ID.
use crate::capabilities::AspectPlanSource;
use crate::durability::data::RelationalCheckpointArtifactPolicy;
use crate::identity::data::KindId;
use crate::schema::data::AspectBinding;
use crate::storage::data::RecordLifecycleState;
use crate::tests::support::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::{AspectValue, ContractValidationInput, InternedString};
#[test]
fn complete_preserves_partial_update_without_demanding_untouched_aspect_reference() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let plan = runtime.entity_aspect_plan(KindId(1)).unwrap();
    assert!(plan.executable_bindings.len() >= 2);
    let untouched = plan
        .executable_bindings
        .iter()
        .find(|binding| matches!(binding.target, AspectBinding::LifecycleTransition))
        .unwrap();
    assert_eq!(untouched.aspect_key().as_str(), "lifecycle");
    let created = create_entity_outcome(&execution, &runtime, "original-name");
    let entity = changed_entities(&created)[0];
    release_test_commit_snapshot(&runtime, &created);
    let updated = update_entity(&execution, &runtime, entity, "updated-name");
    let symbols = runtime.services.symbols.interner_snapshot();
    let untouched_symbol = symbols
        .symbol("lifecycle")
        .expect("creation published real lifecycle meaning");
    let changed_symbol = symbols.symbol("name").unwrap();
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(&execution, updated.commit.commit_id)
        .unwrap()
        .unwrap();
    assert!(envelope
        .revision_symbol_references
        .binary_search(&changed_symbol)
        .is_ok());
    assert!(
        envelope
            .revision_symbol_references
            .binary_search(&untouched_symbol)
            .is_err(),
        "a name-only update has no lifecycle revision reference"
    );
    let original = runtime
        .history
        .branch_cell(runtime.main_branch_identity().branch_id())
        .unwrap()
        .root()
        .unwrap();
    let axes = original.axes();
    let lease = original
        .resident_full_region_witness(entity.partition_id)
        .unwrap()
        .unwrap();
    let full = lease.full_view();
    let arena = &full.partition().entity_arena;
    let physical = arena.physical_index(entity.slot_index()).unwrap();
    let lifecycle = arena.lifecycle[physical];
    assert_eq!(lifecycle, RecordLifecycleState::Live);
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(&execution, RelationalCheckpointArtifactPolicy::Embedded)
        .unwrap();
    let decoded = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        &execution,
        complete.checkpoint().bytes(),
    )
    .unwrap()
    .checkpoint;
    let image = decoded
        .branch_roots
        .iter()
        .find(|image| image.commit_id == updated.commit.commit_id)
        .expect("actual updated root emitted");
    let image = image
        .partition_images
        .iter()
        .find(|image| image.partition_id == entity.partition_id)
        .unwrap();
    assert_eq!(image.entity_arena.lifecycle[physical], lifecycle);
    let state = image.entity_arena.extra[physical]
        .authoritative_aspect_state
        .as_ref()
        .unwrap();
    let name = state
        .entries()
        .iter()
        .find(|entry| entry.basis().key().as_str() == "name")
        .unwrap();
    let ContractValidationInput::Scalar(AspectValue::String(value)) = name.value() else {
        panic!("actual name value required")
    };
    let text = match value {
        InternedString::Raw(text) => text.as_str(),
        InternedString::Stored(text) => text.as_str(),
        InternedString::Symbol(symbol) => decoded
            .symbol_table
            .entries
            .iter()
            .find(|(id, _)| id.0 == symbol.0)
            .unwrap()
            .1
            .as_str(),
    };
    assert_eq!(text, "updated-name");
    assert_eq!(original.axes(), axes);
    assert_eq!(arena.lifecycle[physical], lifecycle);
    release_test_commit_snapshot(&runtime, &updated);
}
