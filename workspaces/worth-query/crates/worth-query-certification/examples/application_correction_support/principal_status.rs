//! Ordinary principal revocation supplies the fresh-authority correction court.
use super::{
    authentication::{operator_identity, reviewer_identity},
    edit::{DenialBinding, Edited, MissingValue, ResultBinding},
    schema::*,
};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*, authentication::*},
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct DisablePrincipal {
    pub identity: u64,
}
worth_query_structured_value_binding!(pub InputBinding for DisablePrincipal { identity: "numeric-correction.disable-principal.input.v1" });
worth_query_operation!(pub Disable for NumericSchema, input InputBinding);
worth_query_operation_reads!(Disable => [ExternalIdentityField, MappingStatusField]);
worth_query_operation_writes!(Disable => [MappingStatusField]);
fn identity(input: &DisablePrincipal) -> WorthQueryExternalPrincipalIdentity {
    if input.identity == 1 {
        operator_identity()
    } else {
        reviewer_identity()
    }
}
worth_query_mutation_binding!(
    pub Binding for DisablePrincipal, schema NumericSchema,
    identity "numeric-correction.disable-principal.binding.v1", input InputBinding,
    operation Disable, result ResultBinding,
    idempotency String, identity "numeric-correction.disable-principal.key.v1",
    decision WorthQueryInvariantMutationTarget<NumericSchema, ExternalMapping>, denial DenialBinding,
    handler identity "numeric-correction.disable-principal.handler.v1", outputs NoApplicationMutationOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ExternalMapping, ExternalIdentity, ExternalIdentityField, WorthQueryExternalPrincipalIdentity, ReadOnly, NoApplicationUnit,
    field ExternalIdentityField::reference(), value identity,
);
pub struct Handler;
impl OperationHandler<NumericSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &DisablePrincipal,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<NumericSchema, ExternalMapping>,
        MissingValue,
    > {
        let mapping =
            match reader.resolve_entity(ExternalIdentityField::reference(), identity(input)) {
                Ok(mapping) => mapping,
                Err(cause) => return HandlerResult::ExecutionDenied(cause),
            };
        match reader.field(&mapping, MappingStatusField::reference()) {
            Ok(Some(WorthQueryPrincipalMappingStatus::Enabled)) => {}
            Ok(_) => return HandlerResult::DomainDenied(MissingValue),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
        match reader.mutation_target(&mapping) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &DisablePrincipal,
        target: WorthQueryInvariantMutationTarget<NumericSchema, ExternalMapping>,
        writer: &mut CandidateWriter<'_, NumericSchema, Binding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let target = match writer.projected_entity(&target) {
            Ok(target) => target,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        match writer.write_field(
            &target,
            MappingStatusField::reference(),
            WorthQueryPrincipalMappingStatus::Disabled,
        ) {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
