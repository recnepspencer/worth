//! A fixed schema for the application-rule lease proof.
#[path = "schema/records.rs"]
mod storage_model;
use crate::{Count, Parameter};
pub use storage_model::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::*;
impl crate::ParameterSchemaBinding for LeaseSchema {}

use worth_query_decl::facade::application_schema::fragment::{self, *};
#[derive(Clone, Copy)]
pub struct LeaseSchema;
impl Fragment for LeaseSchema {
    const NAME: &'static str = "LeaseSchema";
    type Bindings = NoBindings;
    type Members = (crate::ParameterContribution, LeaseAuthorizationContribution);
}
impl ApplicationSchema for LeaseSchema {
    const OWNER: &'static str = "parameter_lease_test";
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

impl ApplicationSchemaComposition for LeaseSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct LeaseAuthorizationContribution;
impl ApplicationSchemaContribution<LeaseSchema> for LeaseAuthorizationContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new(
            "worth.query.test.parameter-lease-authorization.v1",
        );
    type Members = (LeaseRecords, Contracts);
}
impl LeaseAuthorizationContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<LeaseSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<LeaseSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<LeaseSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<LeaseSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(schema
            .principal_binding(IdentityBinding::reference())
            .ability(EditParameter::reference())
            .policy(ParameterAccessPolicy::reference())
            .ability_policy(
                EditParameter::reference(),
                ParameterAccessPolicy::reference(),
                [
                    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                        .forward(ParameterOwner::reference())
                        .allow(Parameter::reference()),
                ],
            )
            .operation(
                SetCount::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(SetCount::reference(), 32)
            .operation_requires_ability(SetCount::reference(), EditParameter::reference())
            .operation_read_entity(SetCount::reference(), Parameter::reference())
            .operation_read_field(SetCount::reference(), Count::at("value"))
            .operation_expected_fact(SetCount::reference(), Count::at("value"))
            .operation_write(SetCount::reference(), Count::at("value")))
    }
}

pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        LeaseSchema,
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

worth_query_ability!(pub EditParameter scoped_to Parameter, in LeaseSchema);
worth_query_policy!(pub ParameterAccessPolicy in LeaseSchema);
#[derive(Clone, Debug)]
pub struct SetCountInput;
worth_query_portable_type!(SetCountInput => "worth.query.test.checked-hook-input.v1");
worth_query_structured_value_binding!(pub SetCountInputBinding for SetCountInput { identity: "worth.query.test.checked-hook-input.v1" });
worth_query_operation!(pub SetCount for LeaseSchema, input SetCountInputBinding);
worth_query_operation_requires!(SetCount => [EditParameter]);
worth_query_operation_reads!(SetCount => [Parameter]);

impl
    worth_query_host::facade::application_contribution::WorthQueryApplicationContribution<
        LeaseSchema,
    > for LeaseAuthorizationContribution
{
    type Configuration = ();
    fn configure(
        _: (),
        _: &mut worth_query_host::facade::application_contribution::WorthQueryApplicationContributionSetup<'_,LeaseSchema>,
    ) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial>
    {
        Ok(())
    }
}
