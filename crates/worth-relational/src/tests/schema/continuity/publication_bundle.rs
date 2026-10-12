use super::*;

#[test]
fn schema_continuity_publication_rejects_incomplete_canonical_bundle() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "a");
    let mut envelope = outcome.envelope().clone();
    let fingerprint = SchemaBoundaryFingerprint::new([13_u8; 32]);
    let transition = SchemaTransitionArtifact::new(
        SchemaId("test".to_string()),
        SchemaVersionId(1),
        SchemaId("test".to_string()),
        SchemaVersionId(1),
        vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(1),
                Some(KindId(1)),
                Arc::<str>::from("tag"),
            ),
            vec![SchemaStratum::StructuralShape],
            SchemaPublicationImpact::None,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: None,
            },
        )],
        SchemaContinuationDescriptor::new(
            fingerprint,
            SchemaBridgeDescriptor::new(
                fingerprint,
                DescriptorSemanticsVersion::default(),
                DescriptorCanonicalBasisVersion::default(),
                SchemaContinuationClassification::ContinueWithTransparentBridge,
                SchemaBridgeabilityClassification::Transparent,
                HistoricalInterpretationSensitivity::NotSensitive,
                vec![SchemaStratum::StructuralShape],
            ),
            1,
        ),
        SchemaReconciliationDescriptor::new(
            DescriptorSemanticsVersion::default(),
            DescriptorCanonicalBasisVersion::default(),
            SchemaReconciliationClassification::Additive,
            SchemaReconciliationPolicy::PreserveInformation,
            SchemaLineageArtifact::new(
                SchemaId("test".to_string()),
                SchemaVersionId(1),
                vec![SchemaId("test".to_string())],
                vec![SchemaVersionId(1)],
                Some(BranchId("main".to_string())),
                SchemaReconciliationOrderingMode::CanonicalizedPair,
                SchemaLineageOrderingSemantics::SymmetricResult,
            ),
        ),
    );
    envelope.schema_transition = Some(transition.clone());
    envelope.schema_continuation_descriptor = Some(transition.continuation_descriptor.clone());
    envelope.schema_reconciliation_descriptor = None;
    let plan = SchemaContinuityPlan {
        target_schema_registry: None,
        target_schema_version: envelope.schema_version,
        target_schema_authority: envelope.schema_authority.clone(),
        descriptor_semantics_version: DescriptorSemanticsVersion::default(),
        schema_transition: Some(transition),
        schema_continuation_descriptor: envelope.schema_continuation_descriptor.clone(),
        schema_reconciliation_descriptor: None,
    };

    let preparation = runtime.preparation_runtime_snapshot();
    let error = validate_schema_continuity_publication(
        &preparation,
        &BranchId("main".to_string()),
        &plan,
        &envelope,
    )
    .unwrap_err();

    match error {
        crate::transactions::data::TransactionCommitError::Conflict { error, .. } => {
            assert!(matches!(
                error.class,
                ConflictClass::UnsupportedBridgeDescriptor { .. }
            ));
            assert!(error.detail().contains("must appear together"));
        }
        other => panic!("expected continuity publication conflict, got {other:?}"),
    }
}

#[test]
fn schema_continuity_publication_rejects_descriptor_semantics_mismatch() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "a");
    let fingerprint = SchemaBoundaryFingerprint::new([17_u8; 32]);
    let transition = SchemaTransitionArtifact::new(
        SchemaId("test".to_string()),
        SchemaVersionId(1),
        SchemaId("test".to_string()),
        SchemaVersionId(1),
        vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(1),
                Some(KindId(1)),
                Arc::<str>::from("tag"),
            ),
            vec![SchemaStratum::PublicationContract],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some(Arc::<str>::from("null")),
            },
        )],
        SchemaContinuationDescriptor::new(
            fingerprint,
            SchemaBridgeDescriptor::new(
                fingerprint,
                DescriptorSemanticsVersion(99),
                DescriptorCanonicalBasisVersion::default(),
                SchemaContinuationClassification::ContinueWithVisibleBridge,
                SchemaBridgeabilityClassification::SubscriberVisible,
                HistoricalInterpretationSensitivity::NotSensitive,
                vec![SchemaStratum::PublicationContract],
            ),
            1,
        ),
        SchemaReconciliationDescriptor::new(
            DescriptorSemanticsVersion(99),
            DescriptorCanonicalBasisVersion::default(),
            SchemaReconciliationClassification::Additive,
            SchemaReconciliationPolicy::PreserveInformation,
            SchemaLineageArtifact::new(
                SchemaId("test".to_string()),
                SchemaVersionId(1),
                vec![SchemaId("test".to_string())],
                vec![SchemaVersionId(1)],
                Some(BranchId("main".to_string())),
                SchemaReconciliationOrderingMode::CanonicalizedPair,
                SchemaLineageOrderingSemantics::SymmetricResult,
            ),
        ),
    );
    let mut envelope = outcome.envelope().clone();
    envelope.schema_transition = Some(transition.clone());
    envelope.schema_continuation_descriptor = Some(transition.continuation_descriptor.clone());
    envelope.schema_reconciliation_descriptor = Some(transition.reconciliation_descriptor.clone());
    envelope.descriptor_semantics_version = DescriptorSemanticsVersion::default();
    let plan = SchemaContinuityPlan {
        target_schema_registry: None,
        target_schema_version: envelope.schema_version,
        target_schema_authority: envelope.schema_authority.clone(),
        descriptor_semantics_version: DescriptorSemanticsVersion::default(),
        schema_transition: Some(transition.clone()),
        schema_continuation_descriptor: Some(transition.continuation_descriptor.clone()),
        schema_reconciliation_descriptor: Some(transition.reconciliation_descriptor.clone()),
    };

    let preparation = runtime.preparation_runtime_snapshot();
    let error = validate_schema_continuity_publication(
        &preparation,
        &BranchId("main".to_string()),
        &plan,
        &envelope,
    )
    .unwrap_err();

    match error {
        crate::transactions::data::TransactionCommitError::Conflict { error, .. } => {
            assert!(matches!(
                error.class,
                ConflictClass::UnsupportedBridgeDescriptor { .. }
            ));
            assert!(error.detail().contains("descriptor semantics version"));
        }
        other => panic!("expected continuity publication conflict, got {other:?}"),
    }
}

#[test]
fn shared_continuity_bundle_validator_reports_boundary_fingerprint_mismatch() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "a");
    let fingerprint = SchemaBoundaryFingerprint::new([23_u8; 32]);
    let bridge_fingerprint = SchemaBoundaryFingerprint::new([24_u8; 32]);
    let transition = SchemaTransitionArtifact::new(
        SchemaId("test".to_string()),
        SchemaVersionId(1),
        SchemaId("test".to_string()),
        SchemaVersionId(1),
        vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(1),
                Some(KindId(1)),
                Arc::<str>::from("tag"),
            ),
            vec![SchemaStratum::PublicationContract],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some(Arc::<str>::from("null")),
            },
        )],
        SchemaContinuationDescriptor::new(
            fingerprint,
            SchemaBridgeDescriptor::new(
                bridge_fingerprint,
                DescriptorSemanticsVersion::default(),
                DescriptorCanonicalBasisVersion::default(),
                SchemaContinuationClassification::ContinueWithVisibleBridge,
                SchemaBridgeabilityClassification::SubscriberVisible,
                HistoricalInterpretationSensitivity::NotSensitive,
                vec![SchemaStratum::PublicationContract],
            ),
            1,
        ),
        SchemaReconciliationDescriptor::new(
            DescriptorSemanticsVersion::default(),
            DescriptorCanonicalBasisVersion::default(),
            SchemaReconciliationClassification::Additive,
            SchemaReconciliationPolicy::PreserveInformation,
            SchemaLineageArtifact::new(
                SchemaId("test".to_string()),
                SchemaVersionId(1),
                vec![SchemaId("test".to_string())],
                vec![SchemaVersionId(1)],
                Some(BranchId("main".to_string())),
                SchemaReconciliationOrderingMode::CanonicalizedPair,
                SchemaLineageOrderingSemantics::SymmetricResult,
            ),
        ),
    );
    let mut envelope = outcome.envelope().clone();
    envelope.schema_transition = Some(transition.clone());
    envelope.schema_continuation_descriptor = Some(transition.continuation_descriptor.clone());
    envelope.schema_reconciliation_descriptor = Some(transition.reconciliation_descriptor.clone());

    let issue = validate_schema_continuity_bundle(&envelope).unwrap_err();

    assert!(matches!(
        issue,
        SchemaContinuityBundleIssue::ContinuationBoundaryFingerprintMismatch {
            boundary_fingerprint
        } if boundary_fingerprint == fingerprint
    ));
}
