// Two compiled schema markers preserve native exact-declaration readmission.
pub use crate::document_retention_model::schema::{SetRetentionInput, SetRetentionInputBinding};
use worth_query_host::facade::{
    declaration::{application_schema::*, authentication::*},
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
};

pub trait BodySchemaBinding: ApplicationSchema {}
pub struct DocumentRetentionSchema<const DESTINATION: u64>;
impl<const DESTINATION: u64> BodySchemaBinding for DocumentRetentionSchema<DESTINATION> {}
impl<const DESTINATION: u64> ApplicationSchema for DocumentRetentionSchema<DESTINATION> {
    const OWNER: &'static str = "worth.query.body-court";
    const NAME: &'static str = "BodySchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        Ok(super::program::schema::<DESTINATION>())
    }
}
impl<const DESTINATION: u64> ApplicationSchemaComposition for DocumentRetentionSchema<DESTINATION> {
    type Contributions = (super::program::BodyContribution<DESTINATION>,);
}

worth_query_entity!(pub ExternalMapping for Schema: BodySchemaBinding);
worth_query_entity!(pub Principal for Schema: BodySchemaBinding);
worth_query_entity!(pub Document for Schema: BodySchemaBinding);
worth_query_aspect!(pub ExternalIdentity for Schema: BodySchemaBinding, ExternalMapping; identity = AspectIdentity(0x9175B001), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalFacts for Schema: BodySchemaBinding, Principal; identity = AspectIdentity(0x9175B002), revision = AspectContractRevision(1),);
worth_query_aspect!(pub DocumentFacts for Schema: BodySchemaBinding, Document; identity = AspectIdentity(0x9175B003), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentityField for Schema: BodySchemaBinding, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for Schema: BodySchemaBinding, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for Schema: BodySchemaBinding, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentIdentityField for Schema: BodySchemaBinding, Document, DocumentFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub DocumentRetentionField for Schema: BodySchemaBinding, Document, DocumentFacts: u64 => U64ApplicationValueBinding, read_write, equality);
pub struct MappingTarget;
impl MappingTarget {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, ExternalMapping, Principal> {
        ApplicationRelationRef::from_schema_identifiers(
            "MappingTarget",
            "ExternalMapping",
            "Principal",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub struct DocumentPrincipalBinding;
impl DocumentPrincipalBinding {
    pub fn reference<Schema: BodySchemaBinding>() -> ApplicationPrincipalBindingRef<
        Schema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "DocumentPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::reference(),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::reference(),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::reference(),
                ),
            },
        )
    }
}
worth_query_operation!(pub SetRetention for Schema: BodySchemaBinding, input SetRetentionInputBinding);
worth_query_operation_reads!(SetRetention => [DocumentIdentityField, DocumentRetentionField]);
worth_query_operation_writes!(SetRetention => [DocumentRetentionField]);
