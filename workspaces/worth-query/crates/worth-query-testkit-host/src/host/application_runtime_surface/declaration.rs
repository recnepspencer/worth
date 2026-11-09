//! Contribution membership is a tuple; Host keeps the contribution configuration.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct HostIdentitySchema;
impl Fragment for HostIdentitySchema {
    const NAME: &'static str = "HostIdentitySchema";
    type Bindings = NoBindings;
    type Members = (HostIdentitySurface,);
}
impl ApplicationSchema for HostIdentitySchema {
    const OWNER: &'static str = "host_identity_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
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
impl ApplicationSchemaComposition for HostIdentitySchema {
    type Contributions = <Self as Fragment>::Members;
}
impl HostIdentitySchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct HostIdentitySurface;
impl ApplicationSchemaContribution<HostIdentitySchema> for HostIdentitySurface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new(
            "worth.query.host.identity-inspection.surface.v1",
        );
    type Members = (
        IdentityModel,
        inspection_capability::InspectionGrants,
        IdentityBinding,
        (
            inspection_capability::InspectionContext,
            inspection_capability::InspectionProvenance,
            inspection_capability::InspectAccountOperation,
            inspection_capability::InspectAccountCapability,
        ),
        (
            inspection_model::AccountQuery,
            inspection_model::AccountReadBinding,
            comparison_model::GovernedAccountQuery,
            comparison_model::GovernedAccountReadBinding,
        ),
    );
}
impl HostIdentitySurface {
    pub const fn reference() -> ApplicationSchemaContributionRef<HostIdentitySchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        HostIdentitySchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "IdentityBinding",
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
impl Registration<HostIdentitySchema> for IdentityBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}
impl Registration<HostIdentitySchema> for inspection_model::AccountQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(inspection_model::definition()))
    }
}
impl Registration<HostIdentitySchema> for inspection_model::AccountReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder
            .application_query_binding::<Self>()
            .application_query_inspection(inspection_model::inspection_definition()))
    }
}
impl Registration<HostIdentitySchema> for comparison_model::GovernedAccountQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(comparison_model::definition()))
    }
}
impl Registration<HostIdentitySchema> for comparison_model::GovernedAccountReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HostIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder
            .application_query_binding::<Self>()
            .application_query_inspection(comparison_model::inspection_definition()))
    }
}
