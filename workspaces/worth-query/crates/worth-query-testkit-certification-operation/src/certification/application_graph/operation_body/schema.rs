#[path = "schema/records.rs"]
pub(super) mod storage_model;
pub use storage_model::*;
// Two compiled schema markers preserve native exact-declaration readmission.
pub use super::input::{BodyInputBinding, SetRetentionInput};
pub use worth_query_decl::facade::{
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_unlinks,
};
pub use worth_query_host::facade::{
    declaration::{application_schema::*, authentication::*},
    worth_query_operation_reads, worth_query_operation_writes,
};

pub trait BodySchemaBinding: ApplicationSchema {
    type InputBinding: ApplicationStructuredValueBinding<Value = SetRetentionInput>;
}
#[derive(Clone, Copy)]
pub struct DocumentRetentionSchema<const DESTINATION: u64>;

impl<const DESTINATION: u64> BodySchemaBinding for DocumentRetentionSchema<DESTINATION> {
    type InputBinding = BodyInputBinding<DESTINATION>;
}
impl<const DESTINATION: u64> ApplicationSchema for DocumentRetentionSchema<DESTINATION> {
    const OWNER: &'static str = "worth.query.body-court";
    const NAME: &'static str = "BodySchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
}
impl<const DESTINATION: u64> ApplicationSchemaComposition for DocumentRetentionSchema<DESTINATION> {
    type Contributions =
        <Self as worth_query_decl::facade::application_schema::fragment::Fragment>::Members;
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
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}
pub struct SetRetention;
impl<Schema: BodySchemaBinding> ApplicationOperationMarkerIdentity<Schema> for SetRetention {
    type InputBinding = Schema::InputBinding;
    const IDENTIFIER: &'static str = "SetRetention";
}
impl SetRetention {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationOperationRef<Schema, Self, SetRetentionInput> {
        ApplicationOperationRef::from_declaration()
    }
}
pub struct SetAuditSource;
impl<Schema: BodySchemaBinding> ApplicationOperationMarkerIdentity<Schema> for SetAuditSource {
    type InputBinding = Schema::InputBinding;
    const IDENTIFIER: &'static str = "SetAuditSource";
}
impl SetAuditSource {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationOperationRef<Schema, Self, SetRetentionInput> {
        ApplicationOperationRef::from_declaration()
    }
}
worth_query_operation_reads!(SetRetention => [Document, Audit, DocumentMembers]);

worth_query_operation_creates!(SetRetention => [Audit]);
worth_query_operation_links!(SetRetention => [DocumentMembers]);

pub struct SetDraftNote;
pub struct ClearDraftNote;
pub struct RequireDraftNote;
pub struct UnlinkMember;
impl<Schema: BodySchemaBinding> ApplicationOperationMarkerIdentity<Schema> for SetDraftNote {
    type InputBinding = super::draft::DraftInputBinding;
    const IDENTIFIER: &'static str = "SetDraftNote";
}
impl<Schema: BodySchemaBinding> ApplicationOperationMarkerIdentity<Schema> for ClearDraftNote {
    type InputBinding = super::draft::DraftInputBinding;
    const IDENTIFIER: &'static str = "ClearDraftNote";
}
impl<Schema: BodySchemaBinding> ApplicationOperationMarkerIdentity<Schema> for RequireDraftNote {
    type InputBinding = super::draft::DraftInputBinding;
    const IDENTIFIER: &'static str = "RequireDraftNote";
}
impl<Schema: BodySchemaBinding> ApplicationOperationMarkerIdentity<Schema> for UnlinkMember {
    type InputBinding = super::draft::DraftInputBinding;
    const IDENTIFIER: &'static str = "UnlinkMember";
}
impl UnlinkMember {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationOperationRef<Schema, Self, super::draft::DraftInput> {
        ApplicationOperationRef::from_declaration()
    }
}

impl SetDraftNote {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationOperationRef<Schema, Self, super::draft::DraftInput> {
        ApplicationOperationRef::from_declaration()
    }
}
impl ClearDraftNote {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationOperationRef<Schema, Self, super::draft::DraftInput> {
        ApplicationOperationRef::from_declaration()
    }
}
impl RequireDraftNote {
    pub fn reference<Schema: BodySchemaBinding>(
    ) -> ApplicationOperationRef<Schema, Self, super::draft::DraftInput> {
        ApplicationOperationRef::from_declaration()
    }
}

worth_query_operation_reads!(RequireDraftNote => [DocumentMembers]);

worth_query_operation_reads!(UnlinkMember => [DocumentMembers]);
worth_query_operation_unlinks!(UnlinkMember => [DocumentMembers]);

impl<const DESTINATION: u64> worth_query_decl::facade::application_schema::fragment::Fragment
    for DocumentRetentionSchema<DESTINATION>
{
    const NAME: &'static str = "BodySchema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (super::program::BodyContribution<DESTINATION>,);
}
