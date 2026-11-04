//! Original producer facts cross a real Query checkpoint, not just native cold admission.

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_output_demand::settlement::observation::court::CourtWorthQueryOutputDemandSettlementForTest;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::*;
use worth_execution::ExecutionAllocationPolicy as WorthQueryCheckpointCapturePolicy;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::OptionalSchema;
use worth_relational::facade::transactions::*;

pub fn materialize_source(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<OptionalSchema>,
) {
    let scope = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    runtime
        .with_application_advancement(&scope, |phase| materialize_source_in_phase(runtime, &phase))
        .unwrap();
}
pub fn materialize_source_in_phase<Schema: BirthSchema>(
    runtime: &crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Application<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
) {
    let scope = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let execution = phase
        .execution_request_for_for_test(
            &(*(*runtime.runtime_for_test()).product_runtime_for_test()),
        )
        .unwrap();
    let source = runtime
        .on_branch(runtime.current_world())
        .select(phase)
        .unwrap()
        .resolve_entity(
            phase,
            BodyKey::reference(),
            crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned(),
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
                entity_id: source.entity_id_for_test(),
                aspect_patch: patch,
            }),
        ));
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation_on_application(
        &execution, runtime, batch,
    );
}

#[test]
fn tracked_aspect_absence_survives_query_checkpoint_then_invalidates_on_other_field() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<OptionalSchema>(false);
    let initial = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&runtime, true).unwrap();
    let ids = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::readmit(&runtime, &initial, true);
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
    let restored = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::restore::<OptionalSchema>(checkpoint);
    let retained = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand_with_contacts(
        &restored,
        true,
        0,
        WorthQueryOutputSettlementPosture::RecoveredPerformed,
    )
    .unwrap();
    assert_eq!(crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::readmit(&restored, &retained, true), ids);
    restored
        .with_host_advancement(|phase| {
            let selected = restored
                .on_branch(restored.current_world())
                .select(&phase)
                .unwrap();
            retained.assert_original_provenance(None, None, |settled| {
                selected.require_current_output_settlements(&phase, [settled])
            });
        })
        .unwrap();
    restored.with_host_advancement(|phase| {
        let read = retained.retained_read();
        let selected = restored.select_application_read_observation(&phase, &read).unwrap();
        let seed = selected.original_settled_producer_for_test().unwrap();
        let worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::settlement::observation::OriginalSettledProducerSource::Restored { checkpoint } = seed.source() else {
            panic!("actual checkpoint restoration cannot manufacture a Native conditional observer")
        };
        let source = retained.restored_source.as_ref().unwrap();
        assert_eq!((*checkpoint.scope_for_test()), (*source.source_scope_for_test()));
        assert_eq!((*checkpoint.source_partition_for_test()),(* (*source.checkpoint_for_test()).source_partition_for_test()));
        assert_eq!(seed.read().selected_commit(), read.selected_commit());
    }).unwrap();
    materialize_source(&restored);
    // The handler still reads SourceSelection=None. Only whole-aspect presence
    // changed; a field-only posting or lost original fact would reuse falsely.
    let refreshed = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand_with_contacts(
        &restored,
        true,
        1,
        WorthQueryOutputSettlementPosture::Performed,
    )
    .unwrap();
    assert!(!refreshed.is_stably_reused());
    assert_eq!(crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::readmit(&restored, &refreshed, true), ids);
}

#[test]
fn restored_projection_reads_the_checkpoint_without_authoring_initial_state() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<OptionalSchema>(false);
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
        crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::restore_with_projection::<OptionalSchema>(checkpoint);
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
                            .resolve_entity(BodyKey::reference(), crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned())
                            .unwrap();
                        assert_eq!(
                            reader.field(&anchor, BodyKey::reference()),
                            Some(crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned())
                        );
                        anchor
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let retained_selector = completed.into_parts().0;
            let runtime_selector = (*restored
                .mutation_projection_for_test())
                .project(
                    &phase,
                    |reader| {
                        reader
                            .resolve_entity(BodyKey::reference(), crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::ANCHOR.to_owned())
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
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<OptionalSchema>(false);
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&runtime, true).unwrap();
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
