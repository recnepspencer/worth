use worth_foundational::facade::{
    AbsenceLaw, AspectBinding, AspectContract, AspectContractRevision, AspectEvolutionPolicy,
    AspectIdentity, AspectKey, AspectMask, AspectValue, AuthoritativeAspectChangeKind,
    FieldDeclaration, FieldKey, FieldRequirement, ProjectionMask, ScalarAspectType,
    StructAspectShape,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryBasisSupport,
    ApplicationQueryCardinality, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultShape, ErasedApplicationQueryDefinition,
    WorthQueryPortableApplicationQueryParts, WorthQueryPortableApplicationQueryResultShapeParts,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaMember, WorthQueryPortableApplicationSchemaParts,
    WorthQueryPortableApplicationSchemaRecord,
};
use worth_query_declaration::facade::identity::{CanonicalQueryDigest, CanonicalResultShapeDigest};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
use worth_query_installation::facade::{
    WorthQueryArtifactContractReference, WorthQueryArtifactFamilyIdentity,
    WorthQueryArtifactPosture, WorthQueryArtifactProtocolVersion,
    WorthQueryArtifactReuseEquivalence, WorthQueryArtifactSchemaVersion,
    WorthQueryComparatorRequirement, WorthQueryConditionalEvaluationCondition,
    WorthQueryConditionalGraphReadRole, WorthQueryConditionalNodeContext,
    WorthQueryConditionalNodeRole, WorthQueryConditionalTrigger, WorthQueryDeltaComparisonDomain,
    WorthQueryDeltaThreshold, WorthQueryMaintenancePosture,
    WorthQueryOperationReplayComparatorContract, WorthQueryOperationReplayComparatorDenial,
    WorthQueryOutputRelationship, WorthQueryPortableApplicationConditionalOperationBinding,
    WorthQueryPortableApplicationConditionalOperationBindingParts,
    WorthQueryPortableApplicationOperationContractParts,
    WorthQueryPortableApplicationOperationContractRecord, WorthQueryPortableArtifactContractParts,
    WorthQueryPortableConditionalConditionParts, WorthQueryPortableConditionalNodeDeclaration,
    WorthQueryPortableConditionalNodeParts, WorthQueryPortableDeltaThresholdDenial,
    WorthQueryPortableDeltaThresholdParts, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackageIdentity, WorthQueryPortableFamilyIdentityDenial,
    WorthQueryPortablePackageManifest, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecord,
    WorthQueryQuantityValueFamily, WorthQuerySemanticLocality, WorthQuerySemanticTruthDependency,
    WorthQueryTemporalCondition, WorthQueryThresholdBoundary, WorthQueryTypedFamilyIdentity,
    WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
};

#[test]
fn downstream_code_constructs_untrusted_typed_records_without_package_authority() {
    let schema = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: "external.decoder".to_owned(),
            name: "decoded".to_owned(),
            major: 1,
            minor: 0,
            contributions: Vec::new(),
            members: vec![ApplicationSchemaMember::Entity {
                entity: "item".to_owned(),
            }],
        },
    );
    let mut family_counts = [0_u32; 12];
    family_counts[0] = 1;
    family_counts[7] = 1;
    let manifest = WorthQueryPortablePackageManifest::from_untrusted_fields(
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
        WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes([0x41; 32]),
        2,
        0,
        64,
        family_counts,
    );
    let reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap()
    .push_record(
        0,
        WorthQueryPortablePackageRecord::DomainIdentity(WorthQueryPortableDomainIdentity::new(
            "external.decoder",
            1,
            0,
        )),
    )
    .unwrap()
    .push_record(
        1,
        WorthQueryPortablePackageRecord::ApplicationSchema(schema),
    )
    .unwrap();

    assert_eq!(reconstruction.close().unwrap().records().len(), 2);
}

#[test]
fn downstream_decoder_constructs_owned_untrusted_application_query_meaning() {
    let query_type =
        WorthQueryPortableTypeIdentity::from_untrusted("worth.tests.query.v1".to_owned());
    let result_type =
        WorthQueryPortableTypeIdentity::from_untrusted("worth.tests.query-result.v1".to_owned());
    let definition = ErasedApplicationQueryDefinition::from_untrusted_parts(
        WorthQueryPortableApplicationQueryParts {
            name: ["decoded", "query"].join("_"),
            query_type: query_type.clone(),
            parameter_type: WorthQueryPortableTypeIdentity::from_untrusted(
                "worth.tests.query-parameters.v1".to_owned(),
            ),
            result_type: result_type.clone(),
            scope_type: WorthQueryPortableTypeIdentity::from_untrusted(
                "worth.tests.query-scope.v1".to_owned(),
            ),
            root_entity: "DecodedEntity".to_owned(),
            scope_entity: "DecodedEntity".to_owned(),
            parameters: Vec::new(),
            result_shape: ApplicationQueryResultShape::from_untrusted_parts(
                WorthQueryPortableApplicationQueryResultShapeParts {
                    query_type,
                    root_entity: "DecodedEntity".to_owned(),
                    result_type,
                    fields: Vec::new(),
                    relations: Vec::new(),
                },
            ),
            root_paths: Vec::new(),
            cardinality: ApplicationQueryCardinality::ExactlyOne,
            predicates: Vec::new(),
            ordering: Vec::new(),
            continuation: None,
            live_cause: None,
            dependency_ceiling: ApplicationQueryDependencyCeiling::bounded(0, 0, 0),
            disclosure: ApplicationQueryDisclosureContract::public(),
            authorization: ApplicationQueryAuthorizationRequirement::Public,
            basis_support: ApplicationQueryBasisSupport::current_and_pinned(),
            lanes: ApplicationQueryLaneEligibility::one_shot(),
        },
    );

    assert_eq!(definition.name(), "decoded_query");
    assert_eq!(definition.root_entity(), "DecodedEntity");
}

#[test]
fn downstream_code_constructs_untrusted_cross_reference_and_derived_records() {
    let input_type =
        WorthQueryPortableTypeIdentity::from_untrusted("worth.tests.input.v1".to_owned());
    let binding = WorthQueryPortableApplicationConditionalOperationBinding::from_untrusted_parts(
        WorthQueryPortableApplicationConditionalOperationBindingParts {
            schema_owner: "external.decoder".to_owned(),
            schema_name: "decoded".to_owned(),
            application_operation: "apply".to_owned(),
            input_type: input_type.clone(),
            domain_operation_slot: "apply:1".to_owned(),
            domain_operation_canonical_identity: "stored-claim".to_owned(),
        },
    );
    let operation = WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(
        WorthQueryPortableApplicationOperationContractParts {
            schema: "decoded".to_owned(),
            operation: "apply".to_owned(),
            input_type,
            graph_reads: Vec::new(),
            touches: Vec::new(),
            emissions: Vec::new(),
            external_effect: None,
            reconciliation: None,
        },
    );

    assert_eq!(
        binding.domain_operation_canonical_identity(),
        "stored-claim"
    );
    assert_eq!(operation.operation(), "apply");
    let family = WorthQueryArtifactFamilyIdentity::from_untrusted_string(
        "worth.tests.decoded-artifact".to_owned(),
    );
    assert_eq!(family.as_str(), "worth.tests.decoded-artifact");
    assert_public_artifact_parts_type(None);
}

fn assert_public_artifact_parts_type(_: Option<WorthQueryPortableArtifactContractParts>) {}

#[test]
fn downstream_decoder_retains_owned_replay_comparator_identity() {
    let decoded_family = format!("worth.tests.replay.comparator.{}", 1);
    let comparator = WorthQueryOperationReplayComparatorContract::new(decoded_family.clone())
        .expect("runtime-owned portable family should reconstruct");
    drop(decoded_family);

    assert_eq!(comparator.family(), "worth.tests.replay.comparator.1");
    for malformed in ["", " padded", "padded ", "contains whitespace"] {
        assert_eq!(
            WorthQueryOperationReplayComparatorContract::new(malformed),
            Err(WorthQueryOperationReplayComparatorDenial::InvalidPortableFamily)
        );
    }
}

#[test]
fn downstream_decoder_retains_owned_descriptive_artifact_reference() {
    let decoded_family = format!("worth.tests.artifact.{}", 1);
    let reference = WorthQueryArtifactContractReference::from_untrusted_fields(
        WorthQueryArtifactFamilyIdentity::from_untrusted_string(decoded_family.clone()),
        WorthQueryArtifactSchemaVersion::new(2),
        WorthQueryArtifactProtocolVersion::new(3),
    );
    drop(decoded_family);

    assert_eq!(reference.family().as_str(), "worth.tests.artifact.1");
    assert_eq!(reference.schema_version().get(), 2);
    assert_eq!(reference.protocol_version().get(), 3);
}

#[test]
fn downstream_decoder_retains_owned_descriptive_query_digests() {
    let query_source = format!("decoded-query-{}", 7);
    let shape_source = format!("decoded-shape-{}", 9);
    let query = CanonicalQueryDigest::from_untrusted(query_source.clone());
    let shape = CanonicalResultShapeDigest::from_untrusted(shape_source.clone());
    drop((query_source, shape_source));

    assert_eq!(query.as_str(), "decoded-query-7");
    assert_eq!(shape.as_str(), "decoded-shape-9");
}

#[test]
fn downstream_decoder_reconstructs_every_conditional_condition_variant_from_owned_parts() {
    let dependency = semantic_dependency();
    let family = portable_family("worth.tests.condition.v1");
    let threshold =
        WorthQueryDeltaThreshold::from_untrusted_parts(WorthQueryPortableDeltaThresholdParts {
            value: AspectValue::UInt64(7),
            unit: portable_family("worth.tests.unit.count"),
            value_family: WorthQueryQuantityValueFamily::Integer,
            comparison_domain: WorthQueryDeltaComparisonDomain::AbsoluteDifference,
            boundary: WorthQueryThresholdBoundary::Inclusive,
        })
        .unwrap();
    let parameter_z =
        worth_query_installation::facade::WorthQueryPortableConditionParameter::text("z", "last")
            .unwrap();
    let parameter_a =
        worth_query_installation::facade::WorthQueryPortableConditionParameter::text("a", "first")
            .unwrap();
    let variants = vec![
        WorthQueryPortableConditionalConditionParts::AlwaysEligible,
        WorthQueryPortableConditionalConditionParts::AspectFiltered(vec![
            dependency.clone(),
            dependency.clone(),
        ]),
        WorthQueryPortableConditionalConditionParts::DeltaThreshold {
            dependency,
            threshold,
        },
        WorthQueryPortableConditionalConditionParts::OnDemand,
        WorthQueryPortableConditionalConditionParts::Temporal(
            WorthQueryTemporalCondition::SnapshotAdvance,
        ),
        WorthQueryPortableConditionalConditionParts::DomainSpecific {
            family,
            parameters: vec![parameter_z, parameter_a],
        },
    ];

    for parts in variants {
        let condition =
            WorthQueryConditionalEvaluationCondition::from_untrusted_parts(parts.clone());
        assert_eq!(condition.into_parts(), parts);
    }
}

#[test]
fn untrusted_conditional_declaration_preserves_noncanonical_sequences_for_fresh_readmission() {
    let family = portable_family("worth.tests.registered.v1");
    let dependency = semantic_dependency();
    let parameter_z =
        worth_query_installation::facade::WorthQueryPortableConditionParameter::text("z", "last")
            .unwrap();
    let parameter_a =
        worth_query_installation::facade::WorthQueryPortableConditionParameter::text("a", "first")
            .unwrap();
    let parts = WorthQueryPortableConditionalNodeParts {
        identity: "decoded-node".to_owned(),
        role: WorthQueryConditionalNodeRole::OperationGate,
        dependencies: vec![dependency.clone(), dependency],
        outputs: Vec::new(),
        required_context: vec![
            WorthQueryConditionalNodeContext::WorkflowRun,
            WorthQueryConditionalNodeContext::Basis,
            WorthQueryConditionalNodeContext::WorkflowRun,
        ],
        condition: WorthQueryConditionalEvaluationCondition::from_untrusted_parts(
            WorthQueryPortableConditionalConditionParts::DomainSpecific {
                family: family.clone(),
                parameters: vec![parameter_z, parameter_a],
            },
        ),
        trigger: WorthQueryConditionalTrigger::OnDemand(family.clone()),
        dependency_comparator: WorthQueryComparatorRequirement::Registered(family.clone()),
        output_equivalence:
            worth_query_installation::facade::WorthQueryOutputEquivalenceRequirement::Registered(
                family.clone(),
            ),
        artifact_reuse_equivalence: WorthQueryArtifactReuseEquivalence::Registered(family),
        maintenance: WorthQueryMaintenancePosture::OnDemandOnly,
        artifact: WorthQueryArtifactPosture::Ephemeral,
        output_relationship: WorthQueryOutputRelationship::IntermediateOnly,
    };

    let declaration =
        WorthQueryPortableConditionalNodeDeclaration::from_untrusted_parts(parts.clone());

    assert_eq!(declaration.into_parts(), parts);
}

#[test]
fn portable_identity_and_threshold_reconstruction_fail_closed_on_invalid_local_shape() {
    assert_eq!(
        WorthQueryTypedFamilyIdentity::from_untrusted_portable_identity("not-portable".to_owned()),
        Err(WorthQueryPortableFamilyIdentityDenial::InvalidPortableIdentity)
    );
    assert_eq!(
        WorthQueryDeltaThreshold::from_untrusted_parts(WorthQueryPortableDeltaThresholdParts {
            value: AspectValue::UInt64(7),
            unit: portable_family("worth.tests.unit.count"),
            value_family: WorthQueryQuantityValueFamily::Float64,
            comparison_domain: WorthQueryDeltaComparisonDomain::RelativeRatio,
            boundary: WorthQueryThresholdBoundary::Exclusive,
        }),
        Err(WorthQueryPortableDeltaThresholdDenial::ValueFamilyMismatch)
    );
    assert_eq!(
        WorthQueryDeltaThreshold::from_untrusted_parts(WorthQueryPortableDeltaThresholdParts {
            value: AspectValue::Int64(-1),
            unit: portable_family("worth.tests.unit.count"),
            value_family: WorthQueryQuantityValueFamily::Integer,
            comparison_domain: WorthQueryDeltaComparisonDomain::AbsoluteDifference,
            boundary: WorthQueryThresholdBoundary::Inclusive,
        }),
        Err(WorthQueryPortableDeltaThresholdDenial::InvalidNumericValue)
    );
}

fn portable_family(value: &str) -> WorthQueryTypedFamilyIdentity {
    WorthQueryTypedFamilyIdentity::from_untrusted_portable_identity(value.to_owned()).unwrap()
}

fn semantic_dependency() -> WorthQuerySemanticTruthDependency {
    WorthQuerySemanticTruthDependency::new(
        WorthQueryConditionalGraphReadRole::new("model").unwrap(),
        semantic_contract(),
        AspectMask::<ProjectionMask>::whole_aspect(),
        AspectBinding::EntityField {
            field: FieldKey::new("name").unwrap(),
        },
        WorthQuerySemanticLocality::SourceRecord,
        [AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap()
}

fn semantic_contract() -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new("name").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    AspectContract::struct_aspect(
        AspectKey::new("profile").unwrap(),
        AspectIdentity(1601),
        AspectContractRevision(1),
        StructAspectShape::new([field]).unwrap(),
    )
}
