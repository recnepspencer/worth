use super::*;

#[test]
fn schema_reconciliation_classification_test() {
    let additive = validate_schema_transition(
        schema_transition_for_subscriber_impact(
            SchemaVersionId(2),
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
        ),
        Some(SchemaReconciliationPolicy::PreserveInformation),
    )
    .unwrap();
    let additive_plan = lower_schema_transition(
        additive.clone(),
        Some(SchemaReconciliationPolicy::PreserveInformation),
        DescriptorSemanticsVersion::default(),
        DescriptorCanonicalBasisVersion::default(),
    );

    let narrowing = ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_string()),
        source_schema_version_id: SchemaVersionId(2),
        target_schema_id: SchemaId("test".to_string()),
        target_schema_version_id: SchemaVersionId(3),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                SchemaVersionId(3),
                Some(KindId(1)),
                "obsolete_field",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::RenegotiationRequired,
            HistoricalInterpretationSensitivity::SensitiveToPublicationMeaning,
            SchemaDiffDetail::RemovedField {
                field: field_key("obsolete_field"),
            },
        )],
    };
    let narrowing_error = validate_schema_transition(narrowing.clone(), None).unwrap_err();
    let narrowing_validated = validate_schema_transition(
        narrowing,
        Some(SchemaReconciliationPolicy::PreserveInformation),
    )
    .unwrap();
    let narrowing_plan = lower_schema_transition(
        narrowing_validated.clone(),
        Some(SchemaReconciliationPolicy::PreserveInformation),
        DescriptorSemanticsVersion::default(),
        DescriptorCanonicalBasisVersion::default(),
    );

    let type_conflict = classify_schema_transition(
        ProposedSchemaTransition {
            source_schema_id: SchemaId("test".to_string()),
            source_schema_version_id: SchemaVersionId(3),
            target_schema_id: SchemaId("test".to_string()),
            target_schema_version_id: SchemaVersionId(4),
            diff_atoms: vec![SchemaDiffAtom::new(
                SchemaElementRef::new(
                    SchemaElementKind::Field,
                    SchemaId("test".to_string()),
                    SchemaVersionId(4),
                    Some(KindId(1)),
                    "timing_domain",
                ),
                vec![
                    SchemaStratum::ValueDomain,
                    SchemaStratum::PublicationContract,
                ],
                SchemaPublicationImpact::ObservableSurfaceChanged,
                SchemaSubscriberImpact::RenegotiationRequired,
                HistoricalInterpretationSensitivity::SensitiveToValueMeaning,
                SchemaDiffDetail::TypeChanged {
                    field: field_key("timing_domain"),
                    from_type: "enum<previous>".into(),
                    to_type: "enum<expanded>".into(),
                },
            )],
        },
        Some(SchemaReconciliationPolicy::PreserveInformation),
    );

    let structural_conflict = classify_schema_transition(
        ProposedSchemaTransition {
            source_schema_id: SchemaId("test".to_string()),
            source_schema_version_id: SchemaVersionId(4),
            target_schema_id: SchemaId("test".to_string()),
            target_schema_version_id: SchemaVersionId(5),
            diff_atoms: vec![SchemaDiffAtom::new(
                SchemaElementRef::new(
                    SchemaElementKind::ProjectionContract,
                    SchemaId("test".to_string()),
                    SchemaVersionId(5),
                    None,
                    "mass-properties",
                ),
                vec![
                    SchemaStratum::BehavioralSemantics,
                    SchemaStratum::PublicationContract,
                ],
                SchemaPublicationImpact::ProjectionContractChanged,
                SchemaSubscriberImpact::RenegotiationRequired,
                HistoricalInterpretationSensitivity::SensitiveToPublicationMeaning,
                SchemaDiffDetail::FreeText {
                    detail: "projection semantics became ambiguous".into(),
                    declared_intent: FreeFormSchemaDiffIntent::StructuralContinuityDenied,
                },
            )],
        },
        Some(SchemaReconciliationPolicy::PreserveInformation),
    );

    let schema_conflict_localization_report = [
        additive_plan.validated.proposed.diff_atoms[0]
            .element
            .element_name
            .to_string(),
        narrowing_validated.proposed.diff_atoms[0]
            .element
            .element_name
            .to_string(),
        type_conflict.proposed.diff_atoms[0]
            .element
            .element_name
            .to_string(),
        structural_conflict.proposed.diff_atoms[0]
            .element
            .element_name
            .to_string(),
    ];
    let replayed_additive_reconciliation = lower_schema_transition(
        additive,
        Some(SchemaReconciliationPolicy::PreserveInformation),
        DescriptorSemanticsVersion::default(),
        DescriptorCanonicalBasisVersion::default(),
    )
    .reconciliation_descriptor;
    let descriptor_semantics_versions = [
        lower_schema_transition(
            narrowing_validated.clone(),
            Some(SchemaReconciliationPolicy::PreserveInformation),
            DescriptorSemanticsVersion::default(),
            DescriptorCanonicalBasisVersion::default(),
        )
        .continuation_descriptor
        .bridge
        .semantics_version,
        additive_plan
            .continuation_descriptor
            .bridge
            .semantics_version,
    ];

    assert!(narrowing_error
        .detail()
        .contains("requires an explicit preservation policy"));
    assert_eq!(
        additive_plan.reconciliation_descriptor.classification,
        SchemaReconciliationClassification::Additive
    );
    assert_eq!(
        narrowing_plan.reconciliation_descriptor.classification,
        SchemaReconciliationClassification::Narrowing
    );
    assert_eq!(
        type_conflict.reconciliation,
        SchemaReconciliationClassification::TypeContinuityDenied
    );
    assert_eq!(
        structural_conflict.reconciliation,
        SchemaReconciliationClassification::StructuralContinuityDenied
    );
    assert_eq!(
        additive_plan
            .reconciliation_descriptor
            .resulting_lineage
            .resulting_schema_version_id,
        SchemaVersionId(2)
    );
    assert_eq!(
        narrowing_plan
            .reconciliation_descriptor
            .resulting_lineage
            .resulting_schema_version_id,
        SchemaVersionId(3)
    );
    assert_ne!(
        SchemaReconciliationPolicy::PreserveInformation,
        SchemaReconciliationPolicy::RejectLossyNarrowing
    );
    assert_eq!(
        schema_conflict_localization_report,
        ["tag", "obsolete_field", "timing_domain", "mass-properties"]
    );
    assert_eq!(
        additive_plan.reconciliation_descriptor,
        replayed_additive_reconciliation
    );
    assert_eq!(
        descriptor_semantics_versions,
        [
            DescriptorSemanticsVersion::default(),
            DescriptorSemanticsVersion::default()
        ]
    );
}
