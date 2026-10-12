//! A fixed schema for the application-rule lease proof.
use crate::{Count, Parameter};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;
use worth_query_decl::facade::*;
impl crate::ParameterSchemaBinding for LeaseSchema {}

worth_query_application! {
    pub LeaseSchema {
        owner: "parameter_lease_test",
        version: (1, 0),
        contributions: [crate::ParameterContribution, LeaseAuthorizationContribution],
    }
}
worth_query_application_contribution! {
    pub contribution LeaseAuthorizationContribution in LeaseSchema {
        identity: "worth.query.test.parameter-lease-authorization.v1",
        members: |schema| {
            schema.entity(ExternalMapping::reference()).entity(Principal::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .aspect(Principal::reference(), PrincipalIdentity::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .relation(ParameterOwner::reference(), Principal::reference(), Parameter::reference())
                .principal_binding(IdentityBinding::reference())
                .ability(EditParameter::reference()).policy(ParameterAccessPolicy::reference())
                .ability_policy(EditParameter::reference(), ParameterAccessPolicy::reference(), [
                    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                        .forward(ParameterOwner::reference()).allow(Parameter::reference())
                ])
                .operation(SetCount::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(SetCount::reference(), 32)
                .operation_requires_ability(SetCount::reference(), EditParameter::reference())
                .operation_read_entity(SetCount::reference(), Parameter::reference())
                .operation_read_field(SetCount::reference(), Count::reference())
                .operation_expected_fact(SetCount::reference(), Count::reference())
                .operation_write(SetCount::reference(), Count::reference())
        }
    }
}
worth_query_entity!(pub ExternalMapping for LeaseSchema);
worth_query_entity!(pub Principal for LeaseSchema);
worth_query_aspect!(pub ExternalIdentity for LeaseSchema, ExternalMapping; identity = AspectIdentity(0x9176_6c01), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ExternalIdentityField for LeaseSchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_aspect!(pub PrincipalIdentity for LeaseSchema, Principal; identity = AspectIdentity(0x9176_6c02), revision = AspectContractRevision(1),);
worth_query_field!(
    pub PrincipalIdentityField for LeaseSchema, Principal, PrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub MappingStatusField for LeaseSchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_relation!(
    pub MappingTarget in LeaseSchema,
    ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub IdentityBinding in LeaseSchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);
worth_query_relation!(pub ParameterOwner in LeaseSchema, Principal => Parameter; integrity = same_context_unbounded_retain_dangling);
worth_query_ability!(pub EditParameter scoped_to Parameter, in LeaseSchema);
worth_query_policy!(pub ParameterAccessPolicy in LeaseSchema);
#[derive(Clone, Debug)]
pub struct SetCountInput;
worth_query_portable_type!(SetCountInput => "worth.query.test.checked-hook-input.v1");
worth_query_structured_value_binding!(pub SetCountInputBinding for SetCountInput { identity: "worth.query.test.checked-hook-input.v1" });
worth_query_operation!(pub SetCount for LeaseSchema, input SetCountInputBinding);
worth_query_operation_requires!(SetCount => [EditParameter]);
worth_query_operation_reads!(SetCount => [Parameter, Count]);
worth_query_operation_expects_fact!(SetCount => [Count]);
worth_query_operation_writes!(SetCount => [Count]);

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
