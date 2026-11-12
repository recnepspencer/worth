#[path = "schema/records.rs"]
mod storage_model;
use super::*;
pub use storage_model::Record;
pub use storage_model::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
#[derive(Clone, Copy)]
pub struct Schema;
impl Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = NoBindings;
    type Members = (Contribution,);
}
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "content-door";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}
impl Schema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for Schema {
    type Contributions = <Self as Fragment>::Members;
}
pub struct Contribution;
impl ApplicationSchemaContribution<Schema> for Contribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("content-door.contribution.v1");
    type Members = (
        ContentModel,
        Contracts,
        revision::Restore,
        created_content::Capture,
    );
}

pub struct Contracts;
impl Registration<Schema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(source::declare(created_content::declare(
            revision::declare(operation::declare(declare(schema))),
        )))
    }
}

pub struct Actor;
impl Actor {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        Schema,
        Self,
        Mapping,
        Principal,
        <PrincipalId as RecordSelection>::Value,
        <PrincipalId as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "Actor",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalId::at("ExternalId"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    Status::at("Status"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalId::at("PrincipalId"),
                ),
            },
        )
    }
}

pub fn samples() -> ApplicationContentFieldRef<Values, codec::GridFormat> {
    content_field(Values::at("Values"), codec::GridFormat)
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .principal_binding(Actor::reference())
        .invariant(readers::invariant::definition())
}
