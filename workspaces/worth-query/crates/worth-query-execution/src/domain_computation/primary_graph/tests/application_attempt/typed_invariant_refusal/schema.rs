//! An authorized account, two ordered commit-boundary rules over it, and a
//! ledger rule no account publication touches.
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::authentication::*;
use worth_query_declaration::*;

worth_query_application_schema! {
    pub schema RefusalSchema {
        owner: typed_refusal_test,
        version: (1, 0),
        members: |schema| {
            schema.entity(ExternalMapping::reference()).entity(Principal::reference())
                .entity(Account::reference()).entity(Ledger::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .aspect(Principal::reference(), PrincipalIdentity::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .aspect(Account::reference(), AccountPolicy::reference())
                .field(Account::reference(), AccountIdentity::reference())
                .field(Account::reference(), AccountStatus::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .relation(AccountOwner::reference(), Principal::reference(), Account::reference())
                .principal_binding(IdentityBinding::reference())
                .ability(EditAccount::reference()).policy(AccountAccessPolicy::reference())
                .ability_policy(EditAccount::reference(), AccountAccessPolicy::reference(), [
                    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                        .forward(AccountOwner::reference()).allow(Account::reference())
                ])
                .operation(ChangeStatus::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(ChangeStatus::reference(), 32)
                .operation_requires_ability(ChangeStatus::reference(), EditAccount::reference())
                .operation_read_entity(ChangeStatus::reference(), Account::reference())
                .operation_read_field(ChangeStatus::reference(), AccountStatus::reference())
                .operation_expected_fact(ChangeStatus::reference(), AccountStatus::reference())
                .operation_write(ChangeStatus::reference(), AccountStatus::reference())
                .invariant(super::rule::definition::<super::rule::First>())
                .invariant(super::rule::definition::<super::rule::Second>())
                .operation(CloseAccount::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(CloseAccount::reference(), 32)
                .operation_requires_ability(CloseAccount::reference(), EditAccount::reference())
                .operation_read_entity(CloseAccount::reference(), Account::reference())
                .operation_read_field(CloseAccount::reference(), AccountStatus::reference())
                .operation_delete(CloseAccount::reference(), Account::reference())
                .invariant(super::ledger_rule::definition())
        }
    }
}
worth_query_entity!(pub ExternalMapping for RefusalSchema);
worth_query_entity!(pub Principal for RefusalSchema);
worth_query_entity!(pub Account for RefusalSchema);
worth_query_entity!(pub Ledger for RefusalSchema);
worth_query_aspect!(pub ExternalIdentity for RefusalSchema, ExternalMapping; identity = AspectIdentity(0x9176_6b01), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ExternalIdentityField for RefusalSchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_aspect!(pub PrincipalIdentity for RefusalSchema, Principal; identity = AspectIdentity(0x9176_6b02), revision = AspectContractRevision(1),);
worth_query_field!(
    pub PrincipalIdentityField for RefusalSchema, Principal, PrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub MappingStatusField for RefusalSchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_relation!(
    pub MappingTarget in RefusalSchema,
    ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub IdentityBinding in RefusalSchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);
worth_query_aspect!(pub AccountPolicy for RefusalSchema, Account; identity = AspectIdentity(0x9176_6b03), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AccountIdentity for RefusalSchema, Account, AccountPolicy:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub AccountStatus for RefusalSchema, Account, AccountPolicy:
    String => StringApplicationValueBinding, read_write, equality
);

worth_query_relation!(pub AccountOwner in RefusalSchema, Principal => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_ability!(pub EditAccount scoped_to Account, in RefusalSchema);
worth_query_policy!(pub AccountAccessPolicy in RefusalSchema);
#[derive(Clone, Debug)]
pub struct ChangeStatusInput;
worth_query_portable_type!(ChangeStatusInput => "worth.query.test.typed-refusal-input.v1");
worth_query_structured_value_binding!(pub ChangeStatusInputBinding for ChangeStatusInput { identity: "worth.query.test.typed-refusal-input.v1" });
worth_query_operation!(pub ChangeStatus for RefusalSchema, input ChangeStatusInputBinding);
worth_query_operation_requires!(ChangeStatus => [EditAccount]);
worth_query_operation_reads!(ChangeStatus => [Account, AccountStatus]);
worth_query_operation_expects_fact!(ChangeStatus => [AccountStatus]);
worth_query_operation_writes!(ChangeStatus => [AccountStatus]);
#[derive(Clone, Debug)]
pub struct CloseAccountInput;
worth_query_portable_type!(CloseAccountInput => "worth.query.test.typed-refusal-close.v1");
worth_query_structured_value_binding!(pub CloseAccountInputBinding for CloseAccountInput { identity: "worth.query.test.typed-refusal-close.v1" });
worth_query_operation!(pub CloseAccount for RefusalSchema, input CloseAccountInputBinding);
worth_query_operation_requires!(CloseAccount => [EditAccount]);
worth_query_operation_reads!(CloseAccount => [Account, AccountStatus]);
worth_query_operation_deletes!(CloseAccount => [Account]);
