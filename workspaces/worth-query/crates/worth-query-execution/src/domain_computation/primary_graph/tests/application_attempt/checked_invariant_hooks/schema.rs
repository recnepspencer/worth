//! A fixed schema for the application-rule lease proof.
#[path = "schema/records.rs"]
mod storage_model;

use worth_query_declaration::facade::application_schema::fragment::{self};
#[derive(Clone, Copy)]
pub struct HookSchema;
impl Fragment for HookSchema {
    const NAME: &'static str = "HookSchema";
    type Bindings = NoBindings;
    type Members = (HookSchemaRecords, Contracts);
}
impl ApplicationSchema for HookSchema {
    const OWNER: &'static str = "hook_accounting_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl HookSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Contracts;
impl Registration<HookSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<HookSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<HookSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(schema
            .principal_binding(IdentityBinding::reference())
            .ability(EditAccount::reference())
            .policy(AccountAccessPolicy::reference())
            .ability_policy(
                EditAccount::reference(),
                AccountAccessPolicy::reference(),
                [
                    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                        .forward(AccountOwner::reference())
                        .allow(Account::reference()),
                ],
            )
            .operation(
                ChangeStatus::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(ChangeStatus::reference(), 32)
            .operation_requires_ability(ChangeStatus::reference(), EditAccount::reference())
            .operation_read_entity(ChangeStatus::reference(), Account::reference())
            .operation_read_field(
                ChangeStatus::reference(),
                AccountStatus::at("AccountStatus"),
            )
            .operation_expected_fact(
                ChangeStatus::reference(),
                AccountStatus::at("AccountStatus"),
            )
            .operation_write(
                ChangeStatus::reference(),
                AccountStatus::at("AccountStatus"),
            )
            .invariant(super::accounting_rule::definition()))
    }
}

pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        HookSchema,
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

worth_query_ability!(pub EditAccount scoped_to Account, in HookSchema);
worth_query_policy!(pub AccountAccessPolicy in HookSchema);
#[derive(Clone, Debug)]
pub struct ChangeStatusInput;
worth_query_portable_type!(ChangeStatusInput => "worth.query.test.checked-hook-input.v1");
worth_query_structured_value_binding!(pub ChangeStatusInputBinding for ChangeStatusInput { identity: "worth.query.test.checked-hook-input.v1" });
worth_query_operation!(pub ChangeStatus for HookSchema, input ChangeStatusInputBinding);
worth_query_operation_requires!(ChangeStatus => [EditAccount]);
worth_query_operation_reads!(ChangeStatus => [Account]);

pub use storage_model::*;
use worth_query_declaration::facade::application_schema::fragment::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_declaration::*;
