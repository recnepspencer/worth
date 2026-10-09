use crate::facade::schema::{
    DescriptorCanonicalBasisVersion, DescriptorSemanticsVersion, FreeFormSchemaDiffIntent,
    HistoricalInterpretationSensitivity, ProposedSchemaTransition, SchemaDiffAtom,
    SchemaDiffDetail, SchemaElementKind, SchemaElementRef, SchemaId, SchemaPublicationImpact,
    SchemaReconciliationClassification, SchemaReconciliationPolicy, SchemaStratum,
    SchemaSubscriberImpact, SchemaVersionId,
};
use crate::replay::data::{
    digest_schema_transition_decision, digest_schema_transition_descriptor,
    digest_subscriber_boundary_cdc_surface, digest_subscriber_continuation_summary,
};
use crate::schema::{
    classify_schema_transition, lower_schema_transition, validate_schema_transition,
};
use crate::tests::support::*;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;
fn schema_transition_for_subscriber_impact(
    target_schema_version_id: SchemaVersionId,
    subscriber_impact: SchemaSubscriberImpact,
) -> ProposedSchemaTransition {
    ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_string()),
        source_schema_version_id: SchemaVersionId(target_schema_version_id.0 - 1),
        target_schema_id: SchemaId("test".to_string()),
        target_schema_version_id,
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                target_schema_version_id,
                Some(KindId(1)),
                "tag",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            subscriber_impact,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )
        .with_boundary_visibility_proof(match subscriber_impact {
            SchemaSubscriberImpact::ConsumableSurfaceChanged => {
                crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable
            }
            SchemaSubscriberImpact::ContractUpgradeRequired => {
                crate::schema::data::SubscriberBoundaryVisibility::VisibleRequiresContractUptake
            }
            _ => crate::schema::data::SubscriberBoundaryVisibility::NotVisible,
        })],
    }
}

#[test]
fn schema_evolution_cdc_contract_test() {
    let mut runtime = persisted_runtime_with_test_schema();
    let baseline = create_entity_outcome(&runtime, "anchor");
    let baseline_checkpoint =
        checkpoint_for_schema_version(baseline.patch_position(), SchemaVersionId(1));

    let schema_v2 = AspectSchemaFixture {
        schema_version_id: SchemaVersionId(2),
        ..AspectSchemaFixture::default()
    }
    .build_registry();
    runtime.set_schema_registry_for_test(schema_v2);

    let context = crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
        .with_schema_transition(
            schema_transition_for_subscriber_impact(
                SchemaVersionId(2),
                SchemaSubscriberImpact::ConsumableSurfaceChanged,
            ),
            Some(SchemaReconciliationPolicy::PreserveInformation),
        );
    let mut txn = runtime
        .begin_branch_transaction(context.basis(), context.intent().clone())
        .expect("owner-admitted transaction context");
    txn.push_batch(batch_create("boundary"), AllocationPolicy::SystemAllocation)
        .expect("test staging stays within configured resource budgets");
    let committed = txn
        .commit(&runtime, AllocationPolicy::SystemAllocation)
        .unwrap();

    let live_batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            baseline_checkpoint.clone(),
            32,
        ))
        .unwrap();

    let schema_transition_digest = digest_schema_transition_descriptor(
        committed.envelope().schema_transition.as_ref().unwrap(),
        committed.envelope().descriptor_semantics_version,
    );
    let schema_boundary_cdc_digest = digest_subscriber_boundary_cdc_surface(
        &live_batch.patches,
        live_batch.continuation.crossed_boundaries(),
        live_batch.continuation.continuation_summary(),
        &live_batch.recovery_decision,
    );
    let subscriber_contract_matrix =
        digest_subscriber_continuation_summary(live_batch.continuation.continuation_summary());
    let transition_decision_digest = digest_schema_transition_decision(
        committed
            .envelope()
            .schema_continuation_descriptor
            .as_ref()
            .unwrap(),
        committed
            .envelope()
            .schema_reconciliation_descriptor
            .as_ref()
            .unwrap(),
        committed.envelope().descriptor_semantics_version,
    );
    let descriptor_semantics_version = committed.envelope().descriptor_semantics_version;

    let (_recovery, recovered) = checkpoint_and_recover_with(&runtime, || {
        let registry = AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::default()
        }
        .build_registry();
        RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::CertificationCore)
            .schema_registry(registry)
            .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
            .durable_store_layout(DurableStoreLayout {
                root_path: unique_test_store_path("worth-relational-m5-schema-evolution-cdc"),
                segment_commit_capacity: 2,
            })
            .build()
    });

    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(committed.commit.commit_id)
        .expect("recovered canonical envelope");
    let recovered_batch = recovered
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            baseline_checkpoint,
            32,
        ))
        .unwrap();

    assert_eq!(
        schema_transition_digest,
        digest_schema_transition_descriptor(
            recovered_envelope.schema_transition.as_ref().unwrap(),
            recovered_envelope.descriptor_semantics_version,
        )
    );
    assert_eq!(
        schema_boundary_cdc_digest,
        digest_subscriber_boundary_cdc_surface(
            &recovered_batch.patches,
            recovered_batch.continuation.crossed_boundaries(),
            recovered_batch.continuation.continuation_summary(),
            &recovered_batch.recovery_decision,
        )
    );
    assert_eq!(
        subscriber_contract_matrix,
        digest_subscriber_continuation_summary(recovered_batch.continuation.continuation_summary())
    );
    assert_eq!(
        transition_decision_digest,
        digest_schema_transition_decision(
            recovered_envelope
                .schema_continuation_descriptor
                .as_ref()
                .unwrap(),
            recovered_envelope
                .schema_reconciliation_descriptor
                .as_ref()
                .unwrap(),
            recovered_envelope.descriptor_semantics_version,
        )
    );
    assert_eq!(
        descriptor_semantics_version,
        recovered_envelope.descriptor_semantics_version
    );
}

mod reconciliation;
