pub use worth_foundational::facade::{
    AbsenceLaw, AspectBinding, AspectContract, AspectContractRevision, AspectEvolutionPolicy,
    AspectIdentity, AspectKey, AspectMask, AspectValue, AuthoritativeAspectChangeKind,
    FieldDeclaration, FieldKey, FieldRequirement, ProjectionMask, ScalarAspectType,
    StructAspectShape,
};
pub use worth_query_declaration::facade::application_query::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryBasisSupport,
    ApplicationQueryCardinality, ApplicationQueryDisclosureContract,
    ApplicationQueryLaneEligibility, ApplicationQueryResultShape, ErasedApplicationQueryDefinition,
    WorthQueryPortableApplicationQueryParts, WorthQueryPortableApplicationQueryResultShapeParts,
};
pub use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaMember, WorthQueryPortableApplicationSchemaParts,
    WorthQueryPortableApplicationSchemaRecord,
};
pub use worth_query_declaration::facade::identity::{
    CanonicalQueryDigest, CanonicalResultShapeDigest,
};
pub use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
pub use worth_query_installation::facade::{
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

pub fn assert_public_artifact_parts_type(_: Option<WorthQueryPortableArtifactContractParts>) {}

pub fn portable_family(value: &str) -> WorthQueryTypedFamilyIdentity {
    WorthQueryTypedFamilyIdentity::from_untrusted_portable_identity(value.to_owned()).unwrap()
}

pub fn semantic_dependency() -> WorthQuerySemanticTruthDependency {
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

pub fn semantic_contract() -> AspectContract {
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
