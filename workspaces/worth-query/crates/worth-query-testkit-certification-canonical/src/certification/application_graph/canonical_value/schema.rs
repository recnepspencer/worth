//! One independently installed source field using the generated canonical binding.
#[path = "schema/records.rs"]
mod storage_model;
pub use super::{binding::NestedBinding, model::*, query};
pub use declaration::{application_schema::*, authentication::*};
pub use storage_model::*;
pub use worth_query_host::facade::{declaration, *};

use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct CanonicalSchema;
impl Fragment for CanonicalSchema {
    const NAME: &'static str = "CanonicalSchema";
    type Bindings = NoBindings;
    type Members = (CanonicalContribution,);
}
impl ApplicationSchema for CanonicalSchema {
    const OWNER: &'static str = "canonical_nested_value";
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
impl CanonicalSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for CanonicalSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct CanonicalContribution;
impl ApplicationSchemaContribution<CanonicalSchema> for CanonicalContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("canonical-nested.application.v1");
    type Members = (StorageModel, Contracts);
}
impl CanonicalContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<CanonicalSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<CanonicalSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<CanonicalSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<CanonicalSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .principal_binding(DocumentPrincipalBinding::reference())
            .application_query_binding::<query::ReadBinding>()
            .application_query(query::definition()))
    }
}

pub struct DocumentPrincipalBinding;
impl DocumentPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        CanonicalSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
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
