//! Original producer facts cross a real Query checkpoint, not just native cold admission.
use super::*;
use crate::facade::application_installation::WorthQueryCheckpointCapturePolicy;
use installation::OptionalSchema;
use worth_relational::facade::transactions::*;

pub(super) fn materialize_source(runtime: &installation::Application<OptionalSchema>) {
    let scope = primary_graph::tests::fixture::live_scope();
    runtime
        .with_application_advancement(&scope, |phase| materialize_source_in_phase(runtime, &phase))
        .unwrap();
}
pub(super) fn materialize_source_in_phase<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
) {
    let scope = primary_graph::tests::fixture::live_scope();
    let execution = phase
        .execution_request_for(&runtime.product_runtime)
        .unwrap();
    let source = runtime
        .on_branch(runtime.current_world())
        .select(phase)
        .unwrap()
        .resolve_entity(
            phase,
            BodyKey::reference(),
            installation::ANCHOR.to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let field = SourceContent::reference::<Schema>();
    let native = runtime
        .installed_schema()
        .native_contracts()
        .aspect(field.entity(), field.aspect())
        .unwrap();
    let contract = native.contract();
    let patch = worth_foundational::facade::PortableRecordAspectPatch::new([
        worth_foundational::facade::PortableAspectPatchOperation::SetWhole {
            basis: worth_foundational::facade::PortableAspectContractBasis::from_contract(contract),
            value: worth_foundational::facade::ContractValidationInput::Struct(
                worth_foundational::facade::StructAspectValue::new([(
                    worth_foundational::facade::FieldKey::new(field.field()).unwrap(),
                    StringApplicationValueBinding::encode(&"complete-required-content".to_owned())
                        .unwrap(),
                )])
                .unwrap(),
            ),
        },
    ]);
    let batch =
        WorkerIntentBatch::new("materialize-optional-source-aspect").push(MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                entity_id: source.entity_id(),
                aspect_patch: patch,
            }),
        ));
    primary_graph::tests::fixture::publish_relational_mutation_on_application(
        &execution, runtime, batch,
    );
}

#[test]
fn tracked_aspect_absence_survives_query_checkpoint_then_invalidates_on_other_field() {
    let runtime = installation::install::<OptionalSchema>(false);
    let initial = journey::demand(&runtime, true).unwrap();
    let ids = court::readmit(&runtime, &initial, true);
    let checkpoint = runtime
        .with_host_advancement(|phase| {
            runtime
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    drop(runtime);
    let restored = installation::restore::<OptionalSchema>(checkpoint);
    let retained = journey::demand_with_contacts(
        &restored,
        true,
        0,
        WorthQueryOutputSettlementPosture::RecoveredPerformed,
    )
    .unwrap();
    assert_eq!(court::readmit(&restored, &retained, true), ids);
    materialize_source(&restored);
    // The handler still reads SourceSelection=None. Only whole-aspect presence
    // changed; a field-only posting or lost original fact would reuse falsely.
    let refreshed = journey::demand_with_contacts(
        &restored,
        true,
        1,
        WorthQueryOutputSettlementPosture::Performed,
    )
    .unwrap();
    assert!(!refreshed.is_stably_reused());
    assert_eq!(court::readmit(&restored, &refreshed, true), ids);
}

#[test]
fn restored_projection_reads_the_checkpoint_without_authoring_initial_state() {
    let runtime = installation::install::<OptionalSchema>(false);
    let checkpoint = runtime
        .with_host_advancement(|phase| {
            runtime
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    drop(runtime);
    let (restored, projection) =
        installation::restore_with_projection::<OptionalSchema>(checkpoint);
    let before = restored
        .with_host_advancement(|phase| {
            restored
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    restored
        .with_host_advancement(|phase| {
            let completed = projection
                .project(
                    &phase,
                    |reader| {
                        let anchor = reader
                            .resolve_entity(BodyKey::reference(), installation::ANCHOR.to_owned())
                            .unwrap();
                        assert_eq!(
                            reader.field(&anchor, BodyKey::reference()),
                            Some(installation::ANCHOR.to_owned())
                        );
                        anchor
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let retained_selector = completed.into_parts().0;
            let runtime_selector = restored
                .mutation_projection
                .project(
                    &phase,
                    |reader| {
                        reader
                            .resolve_entity(BodyKey::reference(), installation::ANCHOR.to_owned())
                            .unwrap()
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0;
            assert_eq!(
                retained_selector.entity_id(),
                runtime_selector.entity_id(),
                "both readers observe the same restored row"
            );
            assert_ne!(
                retained_selector, runtime_selector,
                "retained projection must own a distinct selector authority namespace"
            );
        })
        .unwrap();
    let after = restored
        .with_host_advancement(|phase| {
            restored
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    assert_eq!(
        before.bytes(),
        after.bytes(),
        "restoring and observing author no model state"
    );
}

#[test]
fn checkpoint_inspection_compares_query_header_count_and_output_metadata_bytes() {
    let runtime = installation::install::<OptionalSchema>(false);
    journey::demand(&runtime, true).unwrap();
    let checkpoint = runtime
        .with_host_advancement(|phase| {
            runtime
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    runtime
        .with_host_advancement(|phase| {
            let before = runtime
                .inspect_application_checkpoint(&phase, checkpoint.bytes())
                .unwrap();
            // Independent current wire attacks live at the SDK owner, never in a
            // consuming application's copy of the format.
            let body = &checkpoint.bytes()[8 + 32..];
            let native_len =
                usize::try_from(u64::from_be_bytes(body[14..22].try_into().unwrap())).unwrap();
            assert!(
                body.len() > 30 + native_len,
                "fixture has real output metadata"
            );
            for offset in [6, 22, 30 + native_len] {
                let after = checkpoint
                    .rewrite_authenticated_checkpoint_body_for_test(|body| body[offset] ^= 1);
                let decoded = runtime
                    .inspect_application_checkpoint(&phase, after.bytes())
                    .unwrap();
                assert_ne!(
                    before.model, decoded.model,
                    "ignored Query metadata at body offset {offset}"
                );
            }
        })
        .unwrap();
}
