//! One real schema: a row reads its parent's optional company without copying it.
#[path = "schema/records.rs"]
pub(in crate::facade::certification) mod storage_model;
pub use storage_model::{
    Audit, AuditFacts, AuditIdentity, AuditValue, Company, ExternalIdentity, InputModel, Mapping,
    MappingFacts, MappingStatus, MappingTarget, Parent, ParentFacts, ParentIdentity,
    ParentRevision, ParentRows, Principal, PrincipalFacts, PrincipalIdentity, Row, RowFacts,
    RowIdentity, RowValue,
};
pub use worth_query_host::facade::declaration::{application_schema::*, authentication::*};
#[derive(Clone, Copy)]
pub struct Schema;
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.path-input";
    const NAME: &'static str = "PathInput";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
}
impl worth_query_decl::facade::application_schema::fragment::Fragment for Schema {
    const NAME: &'static str = "PathInput";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (super::program::Contribution,);
}
impl ApplicationSchemaComposition for Schema {
    type Contributions =
        <Self as worth_query_decl::facade::application_schema::fragment::Fragment>::Members;
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
                    ExternalIdentity::at("ExternalIdentity"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatus::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentity::at("PrincipalIdentity"),
                ),
            },
        )
    }
}

pub use super::reuse_note_codec::{Note, NoteCodec};
