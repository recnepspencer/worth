use super::*;

#[test]
fn schema_boundary_fingerprint_is_explicit_256_bit_authority_surface() {
    let bytes = [7_u8; 32];
    let fingerprint = SchemaBoundaryFingerprint::new(bytes);

    assert_eq!(fingerprint, SchemaBoundaryFingerprint(bytes));
    assert_ne!(fingerprint, SchemaBoundaryFingerprint::ZERO);
}

#[test]
fn schema_diff_atom_requires_structured_detail_and_strata() {
    let element = SchemaElementRef::new(
        SchemaElementKind::Field,
        SchemaId("chip".to_string()),
        SchemaVersionId(4),
        None,
        Arc::<str>::from("timing_domain"),
    );
    let atom = SchemaDiffAtom::new(
        element.clone(),
        vec![
            SchemaStratum::StructuralShape,
            SchemaStratum::SubscriberContract,
        ],
        SchemaPublicationImpact::ProjectionContractChanged,
        SchemaSubscriberImpact::ContractUpgradeRequired,
        HistoricalInterpretationSensitivity::SensitiveToPublicationMeaning,
        SchemaDiffDetail::TypeChanged {
            field: field_key("timing_domain"),
            from_type: Arc::<str>::from("enum<previous>").into(),
            to_type: Arc::<str>::from("enum<expanded>").into(),
        },
    );

    assert_eq!(atom.element, element);
    assert_eq!(
        atom.strata,
        vec![
            SchemaStratum::StructuralShape,
            SchemaStratum::SubscriberContract
        ]
    );
    assert!(matches!(atom.detail, SchemaDiffDetail::TypeChanged { .. }));
}

#[test]
fn continuation_admission_observation_remains_non_authoritative_summary_only() {
    let proposed = ProposedSchemaTransition {
        source_schema_id: SchemaId("cad".to_string()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("cad".to_string()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: Vec::new(),
    };
    let validated = ValidatedSchemaTransition {
        proposed,
        continuation_admission_observation:
            SchemaContinuationAdmissionObservation::NonRejectedInAtLeastOneLayer,
        reconciliation: SchemaReconciliationClassification::Additive,
        continuation: SchemaContinuationClassification::ContinueWithVisibleBridge,
        bridgeability: SchemaBridgeabilityClassification::SubscriberVisible,
    };

    assert_eq!(
        validated.continuation_admission_observation,
        SchemaContinuationAdmissionObservation::NonRejectedInAtLeastOneLayer
    );
    assert_eq!(
        validated.continuation,
        SchemaContinuationClassification::ContinueWithVisibleBridge
    );
}

#[test]
fn schema_descriptor_constructors_preserve_semantics_version_and_ordering_truth() {
    let fingerprint = SchemaBoundaryFingerprint::new([3_u8; 32]);
    let bridge = SchemaBridgeDescriptor::new(
        fingerprint,
        DescriptorSemanticsVersion(9),
        DescriptorCanonicalBasisVersion(2),
        SchemaContinuationClassification::ContinueWithTransparentBridge,
        SchemaBridgeabilityClassification::Transparent,
        HistoricalInterpretationSensitivity::NotSensitive,
        vec![SchemaStratum::PublicationContract],
    );
    let continuation = SchemaContinuationDescriptor::new(fingerprint, bridge.clone(), 4);
    let lineage = SchemaLineageArtifact::new(
        SchemaId("Worth".to_string()),
        SchemaVersionId(7),
        vec![
            SchemaId("Worth".to_string()),
            SchemaId("worth-feature".to_string()),
        ],
        vec![SchemaVersionId(6), SchemaVersionId(6)],
        Some(BranchId("main".to_string())),
        SchemaReconciliationOrderingMode::CanonicalizedPair,
        SchemaLineageOrderingSemantics::SymmetricResult,
    );
    let reconciliation = SchemaReconciliationDescriptor::new(
        DescriptorSemanticsVersion(9),
        DescriptorCanonicalBasisVersion(2),
        SchemaReconciliationClassification::Additive,
        SchemaReconciliationPolicy::PreserveInformation,
        lineage.clone(),
    );
    let plan = LoweredSchemaTransitionPlan::new(
        ValidatedSchemaTransition {
            proposed: ProposedSchemaTransition {
                source_schema_id: SchemaId("Worth".to_string()),
                source_schema_version_id: SchemaVersionId(6),
                target_schema_id: SchemaId("Worth".to_string()),
                target_schema_version_id: SchemaVersionId(7),
                diff_atoms: Vec::new(),
            },
            continuation_admission_observation:
                SchemaContinuationAdmissionObservation::NonRejectedInAtLeastOneLayer,
            reconciliation: SchemaReconciliationClassification::Additive,
            continuation: SchemaContinuationClassification::ContinueWithTransparentBridge,
            bridgeability: SchemaBridgeabilityClassification::Transparent,
        },
        continuation.clone(),
        reconciliation.clone(),
    );

    assert_eq!(bridge.semantics_version, DescriptorSemanticsVersion(9));
    assert_eq!(continuation.normalized_boundary_count, 4);
    assert_eq!(reconciliation.resulting_lineage, lineage);
    assert_eq!(
        plan.continuation_descriptor.bridge.bridgeability,
        SchemaBridgeabilityClassification::Transparent
    );
}

#[test]
fn schema_transition_summary_derives_changed_strata_without_duplicate_noise() {
    let fingerprint = SchemaBoundaryFingerprint::new([11_u8; 32]);
    let artifact = SchemaTransitionArtifact::new(
        SchemaId("geom".to_string()),
        SchemaVersionId(2),
        SchemaId("geom".to_string()),
        SchemaVersionId(3),
        vec![
            SchemaDiffAtom::new(
                SchemaElementRef::new(
                    SchemaElementKind::Field,
                    SchemaId("geom".to_string()),
                    SchemaVersionId(3),
                    None,
                    Arc::<str>::from("length_units"),
                ),
                vec![
                    SchemaStratum::ValueDomain,
                    SchemaStratum::PublicationContract,
                ],
                SchemaPublicationImpact::ObservableSurfaceChanged,
                SchemaSubscriberImpact::RenegotiationRequired,
                HistoricalInterpretationSensitivity::SensitiveToValueMeaning,
                SchemaDiffDetail::FreeText {
                    detail: Arc::<str>::from("unit semantics widened").into(),
                    declared_intent: FreeFormSchemaDiffIntent::StructuralContinuityDenied,
                },
            ),
            SchemaDiffAtom::new(
                SchemaElementRef::new(
                    SchemaElementKind::ProjectionContract,
                    SchemaId("geom".to_string()),
                    SchemaVersionId(3),
                    None,
                    Arc::<str>::from("mass-properties"),
                ),
                vec![SchemaStratum::PublicationContract],
                SchemaPublicationImpact::ProjectionContractChanged,
                SchemaSubscriberImpact::ConsumableSurfaceChanged,
                HistoricalInterpretationSensitivity::SensitiveToPublicationMeaning,
                SchemaDiffDetail::ProjectionContractChanged {
                    projection_name: Arc::<str>::from("mass-properties").into(),
                },
            ),
        ],
        SchemaContinuationDescriptor::new(
            fingerprint,
            SchemaBridgeDescriptor::new(
                fingerprint,
                DescriptorSemanticsVersion::default(),
                DescriptorCanonicalBasisVersion::default(),
                SchemaContinuationClassification::RequireRenegotiation,
                SchemaBridgeabilityClassification::RenegotiationOnly,
                HistoricalInterpretationSensitivity::SensitiveToPublicationMeaning,
                vec![
                    SchemaStratum::ValueDomain,
                    SchemaStratum::PublicationContract,
                ],
            ),
            1,
        ),
        SchemaReconciliationDescriptor::new(
            DescriptorSemanticsVersion::default(),
            DescriptorCanonicalBasisVersion::default(),
            SchemaReconciliationClassification::Narrowing,
            SchemaReconciliationPolicy::RejectLossyNarrowing,
            SchemaLineageArtifact::new(
                SchemaId("geom".to_string()),
                SchemaVersionId(3),
                vec![SchemaId("geom".to_string())],
                vec![SchemaVersionId(2)],
                None,
                SchemaReconciliationOrderingMode::CanonicalizedPair,
                SchemaLineageOrderingSemantics::SymmetricResult,
            ),
        ),
    );

    let summary = SchemaTransitionSummary::from_artifact(&artifact);

    assert_eq!(summary.changed_atom_count, 2);
    assert_eq!(
        summary.changed_strata,
        vec![
            SchemaStratum::ValueDomain,
            SchemaStratum::PublicationContract
        ]
    );
    assert_eq!(
        summary.continuation,
        SchemaContinuationClassification::RequireRenegotiation
    );
}

#[test]
fn descriptor_semantics_policy_supports_explicit_historical_versions() {
    let policy = crate::schema::data::DescriptorSemanticsSupportPolicy::new(
        DescriptorSemanticsVersion(3),
        [DescriptorSemanticsVersion(1), DescriptorSemanticsVersion(2)],
    );

    assert_eq!(
        policy.current_write_version(),
        DescriptorSemanticsVersion(3)
    );
    assert!(policy.supports(DescriptorSemanticsVersion(1)));
    assert!(policy.supports(DescriptorSemanticsVersion(2)));
    assert!(policy.supports(DescriptorSemanticsVersion(3)));
    assert!(!policy.supports(DescriptorSemanticsVersion(4)));
}
