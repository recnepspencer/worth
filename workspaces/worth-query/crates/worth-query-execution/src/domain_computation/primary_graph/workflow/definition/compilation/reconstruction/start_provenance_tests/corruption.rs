//! Current semantic corruption reaches native reads/readmission, not old framing.
use super::*;
use worth_foundational::facade::{
    ContractValidationInput, PortableRecordAspectState, PortableRecordAspectStateEntry,
    StructAspectValue,
};
use worth_relational::facade::durability::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalRecoveryStop,
};

#[test]
fn started_revision_reader_refuses_unmanaged_and_unrostered_current_truth() {
    // Each case starts from the same complete production definition/start
    // topology, installed schema and genuine actor. Native publication must
    // succeed; setup/schema denial cannot satisfy this semantic-read oracle.
    for corrupt in ["unmanaged", "unrostered"] {
        let world = installed_authorization_world(true);
        let (support, original, _) =
            installed_provenance_support(world.application.installed_schema());
        let lease = payer();
        let execution = ExecutionRequest::leased(&lease);
        let layout = world
            .application
            .runtime
            .primary_graph()
            .unwrap()
            .layout
            .workflow()
            .clone();
        let instance = publish_instance_value(
            &world,
            &execution,
            &layout,
            original,
            corrupt,
            admitted_actor(&world),
            |runtime, basis| match corrupt {
                "unmanaged" => AspectValue::String(InternedString::Raw(original.to_string())),
                "unrostered" => start_provenance::admit(
                    &execution,
                    runtime,
                    basis,
                    &unadmitted_program_revision(),
                )
                .unwrap(),
                _ => unreachable!(),
            },
        );
        let selected = selected(&world, &execution);
        let handle = world
            .application
            .runtime
            .primary_graph()
            .unwrap()
            .integration_handle();
        handle.with_runtime(&execution, |runtime| {
            assert!(
                matches!(
                    start_provenance::read(
                        &execution,
                        runtime,
                        selected.1.snapshot_handle(),
                        &layout,
                        instance,
                        &support
                    ),
                    Err(start_provenance::StartProvenanceReadDenial::Corrupt)
                ),
                "{corrupt}"
            );
        });
        // A real paid current counterpart in the same installed native owner.
        let valid = publish_instance(
            &world,
            &execution,
            &layout,
            original,
            "valid",
            admitted_actor(&world),
        );
        let current = super::fixture::selected(&world, &execution);
        handle.with_runtime(&execution, |runtime| {
            assert_eq!(
                start_provenance::read(
                    &execution,
                    runtime,
                    current.1.snapshot_handle(),
                    &layout,
                    valid,
                    &support
                )
                .unwrap(),
                original
            )
        });
    }
}

#[test]
fn started_revision_missing_current_field_is_rejected_by_native_restore_before_publication() {
    let world = installed_authorization_world(true);
    let (support, original, _) = installed_provenance_support(world.application.installed_schema());
    let lease = payer();
    let execution = ExecutionRequest::leased(&lease);
    let layout = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .workflow()
        .clone();
    let instance = publish_instance(
        &world,
        &execution,
        &layout,
        original,
        "missing-current",
        admitted_actor(&world),
    );
    let handle = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .integration_handle();
    let (config, native, plan) = handle.with_runtime(&execution, |runtime| {
        runtime
            .durability_authority()
            .checkpoint(&execution)
            .unwrap();
        (
            runtime.config(),
            runtime
                .durability_authority()
                .native_checkpoint(execution.allocation_policy())
                .unwrap(),
            runtime
                .durability()
                .recovery_plan(
                    &execution,
                    RecoveryVerificationMode::NormalRecoveryVerification,
                )
                .unwrap(),
        )
    });
    let bytes = native.bytes().to_vec();
    let mut corrupted = plan.clone();
    let checkpoint = corrupted
        .checkpoint
        .as_mut()
        .expect("actual captured current checkpoint");
    let partition = checkpoint
        .partition_images
        .iter_mut()
        .find(|image| image.partition_id == instance.partition_id)
        .unwrap();
    let arena = &mut partition.entity_arena;
    let slot = arena
        .slots
        .iter()
        .position(|slot| *slot == instance.local_slot_value())
        .unwrap();
    assert_eq!(arena.kind_ids[slot], Some(layout.instance.entity_kind));
    // Only the mirror is changed. Untouched root digests remain valid; native
    // readmission must validate this divergent mirror before installing it.
    let mut removed = 0;
    for metadata in &mut arena.metadata_history[slot] {
        removed += remove_started_field(&mut metadata.authoritative_aspect_state);
    }
    removed += remove_started_field(&mut arena.extra[slot].authoritative_aspect_state);
    assert!(removed > 0);
    let mut target = RelationalRuntime::new(config.clone());
    let before = native_snapshot(&target);
    let error = target
        .durability_recovery()
        .recover(&execution, corrupted)
        .unwrap_err();
    let RelationalRecoveryStop::Durability(error) = error else {
        panic!("semantic current-field refusal required: {error:?}")
    };
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        error
            .detail
            .contains("checkpoint aspect readmission denied"),
        "{error:?}"
    );
    assert!(
        error.detail.contains("started-program-revision"),
        "must name required field: {error:?}"
    );
    assert_eq!(native_snapshot(&target).version_id(), before.version_id());
    assert_eq!(native.bytes(), bytes);
    // A missing dictionary meaning is injected into the actual captured plan,
    // not published through a producer that already refuses unresolved symbols.
    let mut unresolved = plan.clone();
    let text = original.to_string();
    let dictionary = &mut unresolved.checkpoint.as_mut().unwrap().symbol_table.entries;
    let before_rows = dictionary.len();
    dictionary.retain(|(_, value)| value.as_str() != text);
    assert_eq!(dictionary.len() + 1, before_rows);
    unresolved
        .config
        .identity
        .symbol_table
        .entries
        .retain(|(_, value)| value.as_str() != text);
    let mut unresolved_config = config.clone();
    unresolved_config
        .identity
        .symbol_table
        .entries
        .retain(|(_, value)| value.as_str() != text);
    let mut unresolved_target = RelationalRuntime::new(unresolved_config);
    let untouched = native_snapshot(&unresolved_target);
    let error = unresolved_target
        .durability_recovery()
        .recover(&execution, unresolved)
        .unwrap_err();
    let RelationalRecoveryStop::Durability(error) = error else {
        panic!("native missing-meaning refusal required: {error:?}")
    };
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        error.detail.contains("UnresolvedContentSymbol"),
        "{error:?}"
    );
    assert_eq!(
        native_snapshot(&unresolved_target).version_id(),
        untouched.version_id()
    );
    assert_eq!(native.bytes(), bytes);
    // SAME recovery plan/owner, unaltered current data: real semantic positive.
    target
        .durability_recovery()
        .recover(&execution, plan)
        .unwrap();
    let snapshot = native_snapshot(&target);
    assert_eq!(
        start_provenance::read(&execution, &target, &snapshot, &layout, instance, &support)
            .unwrap(),
        original
    );
    assert_eq!(native.bytes(), bytes);
}

fn remove_started_field(state: &mut Option<PortableRecordAspectState>) -> usize {
    let Some(original) = state.as_ref() else {
        return 0;
    };
    let mut removed = 0;
    let entries = original
        .entries()
        .iter()
        .map(|entry| {
            let value = match entry.value() {
                ContractValidationInput::Struct(fields)
                    if entry.basis().key().as_str() == "workflow-instance" =>
                {
                    let retained = fields
                        .fields()
                        .filter_map(|(key, value)| {
                            if key.as_str() == "started-program-revision" {
                                removed += 1;
                                None
                            } else {
                                Some((key.clone(), value.clone()))
                            }
                        })
                        .collect::<Vec<_>>();
                    ContractValidationInput::Struct(StructAspectValue::new(retained).unwrap())
                }
                value => value.clone(),
            };
            PortableRecordAspectStateEntry::new(entry.basis().clone(), value)
        })
        .collect::<Vec<_>>();
    *state = Some(PortableRecordAspectState::new(entries));
    removed
}
