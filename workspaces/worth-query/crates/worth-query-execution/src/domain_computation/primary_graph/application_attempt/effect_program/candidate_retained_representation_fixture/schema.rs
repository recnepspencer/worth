//! The actual schema identity of the two retained-representation contracts.

use super::bindings::{ReservedDeletionBinding, ReservedEmissionBinding};
use crate::domain_computation::primary_graph::tests::fixture::MutationFreeNoticeBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationPathBuilder, StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_declaration::*;

worth_query_entity!(pub ExternalMapping for ReservationSchema);
worth_query_entity!(pub Principal for ReservationSchema);
worth_query_entity!(pub Account for ReservationSchema);
worth_query_aspect!(pub ExternalIdentity for ReservationSchema, ExternalMapping; identity = AspectIdentity(0x91611070), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalIdentity for ReservationSchema, Principal; identity = AspectIdentity(0x91611071), revision = AspectContractRevision(1),);
worth_query_aspect!(pub AccountPolicy for ReservationSchema, Account; identity = AspectIdentity(0x91611072), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentityField for ReservationSchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for ReservationSchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for ReservationSchema, Principal, PrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub AccountStatus for ReservationSchema, Account, AccountPolicy:
    String => StringApplicationValueBinding, read_write, equality);
worth_query_relation!(pub MappingTarget in ReservationSchema,
    ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub AccountOwner in ReservationSchema,
    Principal => Account; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub IdentityBinding in ReservationSchema,
    mapping ExternalMapping { identity: ExternalIdentityField, status: MappingStatusField,
        target: MappingTarget => Principal, principal_identity: PrincipalIdentityField });
worth_query_ability!(pub ViewAccount scoped_to Account, in ReservationSchema);
worth_query_policy!(pub AccountAccessPolicy in ReservationSchema);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct TouchAccountInput;
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct MutationFreeEmitInput;
worth_query_portable_type!(TouchAccountInput => "worth.query.test.reserved-deletion-input.v1");
worth_query_portable_type!(MutationFreeEmitInput => "worth.query.test.reserved-emission-input.v1");
worth_query_structured_value_binding!(pub TouchAccountInputBinding for TouchAccountInput {
    identity: "worth.query.test.reserved-deletion-input.v1" });
worth_query_structured_value_binding!(pub MutationFreeEmitInputBinding for MutationFreeEmitInput {
    identity: "worth.query.test.reserved-emission-input.v1" });
worth_query_operation!(pub TouchAccountOperation for ReservationSchema, input TouchAccountInputBinding);
worth_query_operation_requires!(TouchAccountOperation => [ViewAccount]);
worth_query_operation_reads!(TouchAccountOperation => [Account, AccountStatus]);
worth_query_operation_expects_fact!(TouchAccountOperation => [AccountStatus]);
worth_query_operation_deletes!(TouchAccountOperation => [Account]);
worth_query_operation!(pub MutationFreeEmitOperation for ReservationSchema, input MutationFreeEmitInputBinding);
worth_query_effect!(pub MutationFreeExternalEffect for ReservationSchema, payload MutationFreeNoticeBinding);
worth_query_operation_emits!(MutationFreeEmitOperation => [MutationFreeExternalEffect]);

worth_query_application_schema! {
    pub schema ReservationSchema {
        owner: "candidate_retained_representation_test",
        version: (1, 0),
        members: |schema| {
            schema.entity(ExternalMapping::reference())
                .entity(Principal::reference()).entity(Account::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalIdentity::reference())
                .aspect(Account::reference(), AccountPolicy::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .field(Account::reference(), AccountStatus::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .relation(AccountOwner::reference(), Principal::reference(), Account::reference())
                .principal_binding(IdentityBinding::reference())
                .ability(ViewAccount::reference()).policy(AccountAccessPolicy::reference())
                .ability_policy(ViewAccount::reference(), AccountAccessPolicy::reference(), [
                    ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
                        .forward(AccountOwner::reference()).allow(Account::reference()),
                ])
                .effect(MutationFreeExternalEffect::reference())
                .operation(TouchAccountOperation::reference().definition()
                    .no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(TouchAccountOperation::reference(), 32)
                .operation_read_entity(TouchAccountOperation::reference(), Account::reference())
                .operation_read_field(TouchAccountOperation::reference(), AccountStatus::reference())
                .operation_expected_fact(TouchAccountOperation::reference(), AccountStatus::reference())
                .operation_requires_ability(TouchAccountOperation::reference(), ViewAccount::reference())
                .operation_delete(TouchAccountOperation::reference(), Account::reference())
                .operation(MutationFreeEmitOperation::reference().definition()
                    .external_effect(MutationFreeExternalEffect::reference(),
                        worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new("test-mutation-free-rail").unwrap())
                    .no_aftermath().finish())
                .operation_projection_work_budget(MutationFreeEmitOperation::reference(), 8)
                .operation_emit(MutationFreeEmitOperation::reference(), MutationFreeExternalEffect::reference())
                .application_mutation_binding::<ReservedEmissionBinding>()
                .application_mutation_binding::<ReservedDeletionBinding>()
        }
    }
}
