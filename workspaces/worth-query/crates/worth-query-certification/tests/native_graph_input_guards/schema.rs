//! One real schema: a row reads its parent's optional company without copying it.
use worth_query_host::facade::{
    declaration::{application_schema::*, authentication::*},
    worth_query_aspect, worth_query_entity, worth_query_field,
};
pub struct Schema;
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.path-input";
    const NAME: &'static str = "PathInput";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        Ok(super::program::schema())
    }
}
impl ApplicationSchemaComposition for Schema {
    type Contributions = (super::program::Contribution,);
}
worth_query_entity!(pub Mapping for Schema);
worth_query_entity!(pub Principal for Schema);
worth_query_entity!(pub Parent for Schema);
worth_query_entity!(pub Row for Schema);
worth_query_entity!(pub Audit for Schema);
worth_query_aspect!(pub MappingFacts for Schema, Mapping; identity = AspectIdentity(0x9175D001), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalFacts for Schema, Principal; identity = AspectIdentity(0x9175D002), revision = AspectContractRevision(1),);
worth_query_aspect!(pub ParentFacts for Schema, Parent; identity = AspectIdentity(0x9175D003), revision = AspectContractRevision(1),);
worth_query_aspect!(pub RowFacts for Schema, Row; identity = AspectIdentity(0x9175D004), revision = AspectContractRevision(1),);
worth_query_aspect!(pub AuditFacts for Schema, Audit; identity = AspectIdentity(0x9175D005), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentity for Schema, Mapping, MappingFacts: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatus for Schema, Mapping, MappingFacts: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentity for Schema, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub ParentIdentity for Schema, Parent, ParentFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Company for Schema, Parent, ParentFacts: optional String => StringApplicationValueBinding, read_write, equality);
worth_query_field!(pub ParentRevision for Schema, Parent, ParentFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub RowIdentity for Schema, Row, RowFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub RowValue for Schema, Row, RowFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub AuditIdentity for Schema, Audit, AuditFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub AuditValue for Schema, Audit, AuditFacts: u64 => U64ApplicationValueBinding, read_only, equality);
pub struct MappingTarget;
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Mapping, Principal> {
        ApplicationRelationRef::from_schema_identifiers(
            "MappingTarget",
            "Mapping",
            "Principal",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub struct ParentRows;
impl ParentRows {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Parent, Row> {
        // Deliberately permits a malformed multi-parent world so the guard's ambiguity refusal is tested.
        ApplicationRelationRef::from_schema_identifiers(
            "ParentRows",
            "Parent",
            "Row",
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        )
    }
}
pub struct PrincipalBinding;
impl PrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        Schema,
        Self,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "PrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentity::reference(),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatus::reference(),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentity::reference(),
                ),
            },
        )
    }
}
