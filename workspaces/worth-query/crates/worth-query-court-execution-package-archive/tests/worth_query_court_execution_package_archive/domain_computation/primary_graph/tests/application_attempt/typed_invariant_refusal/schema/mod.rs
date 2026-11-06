//! An authorized account, two ordered commit-boundary rules over it, and a
//! ledger rule no account publication touches.
#[path = "records.rs"]
mod storage_model;

use worth_query_declaration::facade::application_schema::fragment::{self};
#[derive(Clone, Copy)]
pub struct RefusalSchema;
impl Fragment for RefusalSchema {
    const NAME: &'static str = "RefusalSchema";
    type Bindings = NoBindings;
    type Members = (RefusalSchemaRecords, Contracts);
}
impl ApplicationSchema for RefusalSchema {
    const OWNER: &'static str = "typed_refusal_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl RefusalSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Contracts;
impl Registration<RefusalSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<RefusalSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<RefusalSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
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
            .invariant(super::rule::definition::<super::rule::First>())
            .invariant(super::rule::definition::<super::rule::Second>())
            .operation(
                CloseAccount::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(CloseAccount::reference(), 32)
            .operation_requires_ability(CloseAccount::reference(), EditAccount::reference())
            .operation_read_entity(CloseAccount::reference(), Account::reference())
            .operation_read_field(
                CloseAccount::reference(),
                AccountStatus::at("AccountStatus"),
            )
            .operation_delete(CloseAccount::reference(), Account::reference())
            .invariant(super::ledger_rule::definition()))
    }
}

pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        RefusalSchema,
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

worth_query_ability!(pub EditAccount scoped_to Account, in RefusalSchema);
worth_query_policy!(pub AccountAccessPolicy in RefusalSchema);
#[derive(Clone, Debug)]
pub struct ChangeStatusInput;
worth_query_portable_type!(ChangeStatusInput => "worth.query.test.typed-refusal-input.v1");
worth_query_structured_value_binding!(pub ChangeStatusInputBinding for ChangeStatusInput { identity: "worth.query.test.typed-refusal-input.v1" });
worth_query_operation!(pub ChangeStatus for RefusalSchema, input ChangeStatusInputBinding);
worth_query_operation_requires!(ChangeStatus => [EditAccount]);
worth_query_operation_reads!(ChangeStatus => [Account]);

#[derive(Clone, Debug)]
pub struct CloseAccountInput;
worth_query_portable_type!(CloseAccountInput => "worth.query.test.typed-refusal-close.v1");
worth_query_structured_value_binding!(pub CloseAccountInputBinding for CloseAccountInput { identity: "worth.query.test.typed-refusal-close.v1" });
worth_query_operation!(pub CloseAccount for RefusalSchema, input CloseAccountInputBinding);
worth_query_operation_requires!(CloseAccount => [EditAccount]);
worth_query_operation_reads!(CloseAccount => [Account]);
worth_query_operation_deletes!(CloseAccount => [Account]);

pub use storage_model::*;
use worth_query_declaration::facade::application_schema::fragment::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_declaration::*;
