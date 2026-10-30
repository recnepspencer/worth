//! Ordinary grant withdrawal gives revocation tests a real canonical cause.
use super::super::super::{
    adoption::*,
    edit::{DenialBinding, Edited, MissingValue, ResultBinding},
    schema::*,
};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_program::*, application_schema::*},
    primary_graph::*,
    *,
};
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Withdraw {
    pub grant: String,
}
impl Withdraw {
    fn scope_key(input: &Self) -> String {
        input.grant.clone()
    }
}
worth_query_structured_value_binding!(pub InputBinding for Withdraw { identity: "inspection-grant.withdraw-input.v1" });
worth_query_operation!(pub WithdrawInspectionGrant for NumericSchema, input InputBinding);
worth_query_operation_reads!(WithdrawInspectionGrant => [GrantIdentity, GrantStatus]);
worth_query_operation_writes!(WithdrawInspectionGrant => [GrantStatus]);
worth_query_mutation_binding!(pub Binding for Withdraw, schema NumericSchema,
    identity "inspection-grant.withdraw-binding.v1", input InputBinding, operation WithdrawInspectionGrant, result ResultBinding,
    idempotency String, identity "inspection-grant.withdraw-key.v1",
    decision WorthQueryInvariantMutationTarget<NumericSchema,Grant>, denial DenialBinding,
    handler identity "inspection-grant.withdraw-handler.v1", outputs NoApplicationMutationOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Grant,GrantFacts,GrantIdentity,String,ReadOnly,NoApplicationUnit,
    field GrantIdentity::reference(), value Withdraw::scope_key,
);
impl ApplicationDeclaredOperation<NumericSchema> for WithdrawInspectionGrant {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
        schema
            .operation_read_field(Self::reference(), GrantIdentity::reference())
            .operation_read_field(Self::reference(), GrantStatus::reference())
            .operation_write(Self::reference(), GrantStatus::reference())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    let schema = schema
        .operation(
            WithdrawInspectionGrant::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(WithdrawInspectionGrant::reference(), 32);
    WithdrawInspectionGrant::declare_access(schema).application_mutation_binding::<Binding>()
}
pub struct Handler;
impl OperationHandler<NumericSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &Withdraw,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Grant>, MissingValue> {
        let result = (|| {
            let grant = reader.resolve_entity(GrantIdentity::reference(), input.grant.clone())?;
            if reader.field(&grant, GrantStatus::reference())?.as_deref() != Some("active") {
                return Ok(None);
            }
            reader.mutation_target(&grant).map(Some)
        })();
        match result {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(MissingValue),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Withdraw,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Grant>,
        writer: &mut CandidateWriter<'_, NumericSchema, Binding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let result = writer.projected_entity(&target).and_then(|grant| {
            writer.write_field(&grant, GrantStatus::reference(), "withdrawn".to_owned())
        });
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub struct Feature;
impl ApplicationFeature<NumericSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "inspection-grant.withdraw-feature.v1";
}
pub struct Completion;
impl ApplicationActionDelivery<NumericSchema, Binding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "inspection-grant.withdraw-completion.v1";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, Feature>()
        .action::<Binding, Completion>()
        .finish_action()
        .finish()
}
