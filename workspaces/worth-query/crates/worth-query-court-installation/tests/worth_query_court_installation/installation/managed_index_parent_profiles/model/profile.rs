//! Each profile supplies handles from its actual membership, including the content row.
use super::*;
pub trait ParentModel: Fragment + Default {
    type Mapping: Entity;
    type Principal: Entity;
    type Target: Relation<To = Self::Principal>;
    type Identity: RecordSelection<Entity = Self::Principal, Value = u64, Binding = U64ApplicationValueBinding>
        + OperationReads<super::super::decision_read::DecisionOperation>;
    type MappingIdentity: RecordSelection<
        Entity = Self::Mapping,
        Value = WorthQueryExternalPrincipalIdentity,
        Binding = WorthQueryExternalPrincipalIdentityBinding,
    >;
    type MappingStatus: RecordSelection<
        Entity = Self::Mapping,
        Value = WorthQueryPrincipalMappingStatus,
        Binding = WorthQueryPrincipalMappingStatusBinding,
    >;
    type Content: RecordSelection<
        Entity = Self::Principal,
        Value = worth_foundational::facade::ContentRefId,
        Binding = ContentRefApplicationValueBinding,
    >;
    fn storage<S: ApplicationSchema>(
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial>;
    fn identity() -> Self::Identity;
    fn mapping_identity() -> Self::MappingIdentity;
    fn mapping_status() -> Self::MappingStatus;
    fn content() -> Option<Self::Content>;
}
pub type Principal<S> = <<S as ModelSchema>::Model as ParentModel>::Principal;
pub type Mapping<S> = <<S as ModelSchema>::Model as ParentModel>::Mapping;
pub type PrincipalIdentity<S> = <<S as ModelSchema>::Model as ParentModel>::Identity;
pub type ContentSamples<S> = <<S as ModelSchema>::Model as ParentModel>::Content;
pub fn principal<S: ModelSchema>() -> ApplicationEntityRef<S, Principal<S>> {
    ApplicationEntityRef::from_schema_identifier(<Principal<S> as Entity>::NAME)
}
pub fn principal_identity<S: ModelSchema>() -> PrincipalIdentity<S> {
    S::Model::identity()
}
pub fn mapping_target<S: ModelSchema>(
) -> ApplicationRelationRef<S, <S::Model as ParentModel>::Target, Mapping<S>, Principal<S>> {
    ApplicationRelationRef::from_schema_identifiers(
        "MappingTarget",
        "Mapping",
        "Principal",
        relation_integrity::<<S::Model as ParentModel>::Target>(),
    )
}
impl ParentModel for ParentRecords<false> {
    type Mapping = MappingRecord<false>;
    type Principal = PrincipalRecord<false>;
    type Target = MappingTargetRecord<false>;
    type Identity = FieldSelection<Self, PrincipalRecord<false>, PrincipalFacts<false>, u64>;
    type MappingIdentity = FieldSelection<
        Self,
        MappingRecord<false>,
        ExternalIdentity,
        WorthQueryExternalPrincipalIdentity,
    >;
    type MappingStatus = FieldSelection<
        Self,
        MappingRecord<false>,
        ExternalIdentity,
        WorthQueryPrincipalMappingStatus,
    >;
    type Content = FieldSelection<
        Self,
        PrincipalRecord<false>,
        PrincipalFacts<false>,
        worth_foundational::facade::ContentRefId,
    >;
    fn storage<S: ApplicationSchema>(
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::from_fragment(Self)
    }
    fn identity() -> Self::Identity {
        Self::Identity::at("PrincipalIdentity")
    }
    fn mapping_identity() -> Self::MappingIdentity {
        Self::MappingIdentity::at("ExternalIdentityField")
    }
    fn mapping_status() -> Self::MappingStatus {
        Self::MappingStatus::at("MappingStatus")
    }
    fn content() -> Option<Self::Content> {
        None
    }
}
impl OperationReads<super::super::decision_read::DecisionOperation>
    for FieldSelection<ParentRecords<false>, PrincipalRecord<false>, PrincipalFacts<false>, u64>
{
}
impl ParentModel for ParentRecords<true> {
    type Mapping = MappingRecord<true>;
    type Principal = PrincipalRecord<true>;
    type Target = MappingTargetRecord<true>;
    type Identity = FieldSelection<Self, PrincipalRecord<true>, PrincipalFacts<true>, u64>;
    type MappingIdentity = FieldSelection<
        Self,
        MappingRecord<true>,
        ExternalIdentity,
        WorthQueryExternalPrincipalIdentity,
    >;
    type MappingStatus = FieldSelection<
        Self,
        MappingRecord<true>,
        ExternalIdentity,
        WorthQueryPrincipalMappingStatus,
    >;
    type Content = FieldSelection<
        Self,
        PrincipalRecord<true>,
        PrincipalFacts<true>,
        worth_foundational::facade::ContentRefId,
    >;
    fn storage<S: ApplicationSchema>(
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::from_fragment(Self)
    }
    fn identity() -> Self::Identity {
        Self::Identity::at("PrincipalIdentity")
    }
    fn mapping_identity() -> Self::MappingIdentity {
        Self::MappingIdentity::at("ExternalIdentityField")
    }
    fn mapping_status() -> Self::MappingStatus {
        Self::MappingStatus::at("MappingStatus")
    }
    fn content() -> Option<Self::Content> {
        Some(Self::Content::at("ContentSamples"))
    }
}
impl OperationReads<super::super::decision_read::DecisionOperation>
    for FieldSelection<ParentRecords<true>, PrincipalRecord<true>, PrincipalFacts<true>, u64>
{
}
