use super::*;

#[test]
fn schema_transition_validation_rejects_unstratified_change_sets() {
    let proposed = ProposedSchemaTransition {
        source_schema_id: SchemaId("geom".to_string()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("geom".to_string()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("geom".to_string()),
                SchemaVersionId(2),
                None,
                Arc::<str>::from("length"),
            ),
            Vec::new(),
            SchemaPublicationImpact::None,
            SchemaSubscriberImpact::None,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("length"),
                required: false,
                default_expression: None,
            },
        )],
    };

    let error = validate_schema_transition(proposed, None).unwrap_err();
    assert!(error
        .detail()
        .contains("does not declare any schema strata"));
}

#[test]
fn schema_transition_validation_requires_explicit_policy_for_narrowing() {
    let proposed = ProposedSchemaTransition {
        source_schema_id: SchemaId("chip".to_string()),
        source_schema_version_id: SchemaVersionId(8),
        target_schema_id: SchemaId("chip".to_string()),
        target_schema_version_id: SchemaVersionId(9),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("chip".to_string()),
                SchemaVersionId(9),
                None,
                Arc::<str>::from("timing_margin"),
            ),
            vec![SchemaStratum::StructuralShape],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::RenegotiationRequired,
            HistoricalInterpretationSensitivity::SensitiveToPublicationMeaning,
            SchemaDiffDetail::RemovedField {
                field: field_key("timing_margin"),
            },
        )],
    };

    let error = validate_schema_transition(proposed, None).unwrap_err();
    assert!(error
        .detail()
        .contains("requires an explicit preservation policy"));
}

#[test]
fn schema_transition_classification_and_lowering_are_deterministic_for_visible_bridge_cases() {
    let proposed = ProposedSchemaTransition {
        source_schema_id: SchemaId("web".to_string()),
        source_schema_version_id: SchemaVersionId(3),
        target_schema_id: SchemaId("web".to_string()),
        target_schema_version_id: SchemaVersionId(4),
        diff_atoms: vec![
            SchemaDiffAtom::new(
                SchemaElementRef::new(
                    SchemaElementKind::Field,
                    SchemaId("web".to_string()),
                    SchemaVersionId(4),
                    None,
                    Arc::<str>::from("optional_tag"),
                ),
                vec![
                    SchemaStratum::StructuralShape,
                    SchemaStratum::PublicationContract,
                ],
                SchemaPublicationImpact::ObservableSurfaceChanged,
                SchemaSubscriberImpact::ConsumableSurfaceChanged,
                HistoricalInterpretationSensitivity::NotSensitive,
                SchemaDiffDetail::AddedField {
                    field: field_key("optional_tag"),
                    required: false,
                    default_expression: Some(Arc::<str>::from("null").into()),
                },
            )
            .with_boundary_visibility_proof(
                crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
            ),
            SchemaDiffAtom::new(
                SchemaElementRef::new(
                    SchemaElementKind::ProjectionContract,
                    SchemaId("web".to_string()),
                    SchemaVersionId(4),
                    None,
                    Arc::<str>::from("orders.list"),
                ),
                vec![
                    SchemaStratum::PublicationContract,
                    SchemaStratum::SubscriberContract,
                ],
                SchemaPublicationImpact::ProjectionContractChanged,
                SchemaSubscriberImpact::ConsumableSurfaceChanged,
                HistoricalInterpretationSensitivity::NotSensitive,
                SchemaDiffDetail::ProjectionContractChanged {
                    projection_name: Arc::<str>::from("orders.list").into(),
                },
            )
            .with_boundary_visibility_proof(
                crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
            ),
        ],
    };

    let classified = classify_schema_transition(
        proposed.clone(),
        Some(SchemaReconciliationPolicy::PreserveInformation),
    );
    let validated = validate_schema_transition(
        proposed.clone(),
        Some(SchemaReconciliationPolicy::PreserveInformation),
    )
    .unwrap();
    let lowered = lower_schema_transition(
        validated.clone(),
        Some(SchemaReconciliationPolicy::PreserveInformation),
        DescriptorSemanticsVersion::default(),
        DescriptorCanonicalBasisVersion::default(),
    );
    let lowered_again = lower_schema_transition(
        validate_schema_transition(
            proposed,
            Some(SchemaReconciliationPolicy::PreserveInformation),
        )
        .unwrap(),
        Some(SchemaReconciliationPolicy::PreserveInformation),
        DescriptorSemanticsVersion::default(),
        DescriptorCanonicalBasisVersion::default(),
    );

    assert_eq!(
        classified.continuation,
        SchemaContinuationClassification::ContinueWithVisibleBridge
    );
    assert_eq!(
        validated.continuation,
        SchemaContinuationClassification::ContinueWithVisibleBridge
    );
    assert_eq!(
        validated.bridgeability,
        SchemaBridgeabilityClassification::SubscriberVisible
    );
    assert_eq!(
        lowered.continuation_descriptor.boundary_fingerprint,
        lowered_again.continuation_descriptor.boundary_fingerprint
    );
    assert_eq!(
        lowered.reconciliation_descriptor.policy,
        SchemaReconciliationPolicy::PreserveInformation
    );
}

#[test]
fn consumable_surface_change_requires_explicit_visible_bridge_proof() {
    let proposed = ProposedSchemaTransition {
        source_schema_id: SchemaId("web".to_string()),
        source_schema_version_id: SchemaVersionId(3),
        target_schema_id: SchemaId("web".to_string()),
        target_schema_version_id: SchemaVersionId(4),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("web".to_string()),
                SchemaVersionId(4),
                None,
                Arc::<str>::from("optional_tag"),
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("optional_tag"),
                required: false,
                default_expression: Some(Arc::<str>::from("null").into()),
            },
        )
        .with_boundary_visibility_proof(
            crate::schema::data::SubscriberBoundaryVisibility::VisibleRequiresContractUptake,
        )],
    };

    let classified = classify_schema_transition(
        proposed,
        Some(SchemaReconciliationPolicy::PreserveInformation),
    );

    assert_eq!(
        classified.continuation,
        SchemaContinuationClassification::RequireRenegotiation
    );
}

#[test]
fn schema_boundary_fingerprint_is_canonical_across_diff_atom_orderings() {
    let atom_a = SchemaDiffAtom::new(
        SchemaElementRef::new(
            SchemaElementKind::Field,
            SchemaId("web".to_string()),
            SchemaVersionId(4),
            None,
            Arc::<str>::from("optional_tag"),
        ),
        vec![
            SchemaStratum::PublicationContract,
            SchemaStratum::StructuralShape,
        ],
        SchemaPublicationImpact::ObservableSurfaceChanged,
        SchemaSubscriberImpact::ConsumableSurfaceChanged,
        HistoricalInterpretationSensitivity::NotSensitive,
        SchemaDiffDetail::AddedField {
            field: field_key("optional_tag"),
            required: false,
            default_expression: Some(Arc::<str>::from("null").into()),
        },
    )
    .with_boundary_visibility_proof(
        crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
    );
    let atom_b = SchemaDiffAtom::new(
        SchemaElementRef::new(
            SchemaElementKind::ProjectionContract,
            SchemaId("web".to_string()),
            SchemaVersionId(4),
            None,
            Arc::<str>::from("orders.list"),
        ),
        vec![
            SchemaStratum::SubscriberContract,
            SchemaStratum::PublicationContract,
        ],
        SchemaPublicationImpact::ProjectionContractChanged,
        SchemaSubscriberImpact::ConsumableSurfaceChanged,
        HistoricalInterpretationSensitivity::NotSensitive,
        SchemaDiffDetail::ProjectionContractChanged {
            projection_name: Arc::<str>::from("orders.list").into(),
        },
    )
    .with_boundary_visibility_proof(
        crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
    );

    let lowered_a = lower_schema_transition(
        validate_schema_transition(
            ProposedSchemaTransition {
                source_schema_id: SchemaId("web".to_string()),
                source_schema_version_id: SchemaVersionId(3),
                target_schema_id: SchemaId("web".to_string()),
                target_schema_version_id: SchemaVersionId(4),
                diff_atoms: vec![atom_a.clone(), atom_b.clone()],
            },
            Some(SchemaReconciliationPolicy::PreserveInformation),
        )
        .unwrap(),
        Some(SchemaReconciliationPolicy::PreserveInformation),
        DescriptorSemanticsVersion::default(),
        DescriptorCanonicalBasisVersion::default(),
    );
    let lowered_b = lower_schema_transition(
        validate_schema_transition(
            ProposedSchemaTransition {
                source_schema_id: SchemaId("web".to_string()),
                source_schema_version_id: SchemaVersionId(3),
                target_schema_id: SchemaId("web".to_string()),
                target_schema_version_id: SchemaVersionId(4),
                diff_atoms: vec![atom_b, atom_a],
            },
            Some(SchemaReconciliationPolicy::PreserveInformation),
        )
        .unwrap(),
        Some(SchemaReconciliationPolicy::PreserveInformation),
        DescriptorSemanticsVersion::default(),
        DescriptorCanonicalBasisVersion::default(),
    );

    assert_eq!(
        lowered_a.continuation_descriptor.boundary_fingerprint,
        lowered_b.continuation_descriptor.boundary_fingerprint
    );
}

#[test]
fn type_continuity_denied_schema_transition_is_rejected_not_continued() {
    let proposed = ProposedSchemaTransition {
        source_schema_id: SchemaId("chip".to_string()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("chip".to_string()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("chip".to_string()),
                SchemaVersionId(2),
                None,
                Arc::<str>::from("timing_domain"),
            ),
            vec![
                SchemaStratum::ValueDomain,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::SensitiveToValueMeaning,
            SchemaDiffDetail::TypeChanged {
                field: field_key("timing_domain"),
                from_type: Arc::<str>::from("enum<previous>").into(),
                to_type: Arc::<str>::from("enum<expanded>").into(),
            },
        )],
    };

    let validated = validate_schema_transition(
        proposed,
        Some(SchemaReconciliationPolicy::PreserveInformation),
    )
    .unwrap();

    assert_eq!(
        validated.reconciliation,
        SchemaReconciliationClassification::TypeContinuityDenied
    );
    assert_eq!(
        validated.continuation,
        SchemaContinuationClassification::Rejected
    );
    assert_eq!(
        validated.bridgeability,
        SchemaBridgeabilityClassification::Rejected
    );
    assert_eq!(
        validated.continuation_admission_observation,
        SchemaContinuationAdmissionObservation::RejectedInAllLayers
    );
}
