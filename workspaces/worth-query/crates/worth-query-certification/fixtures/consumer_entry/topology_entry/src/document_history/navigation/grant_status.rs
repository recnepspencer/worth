//! Ordinary fixture administration changes a live grant through its real owner.
use super::*;
use worth_query_decl::facade::{
    application_operation::*, worth_query_mutation_binding, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct SetPlanarNavigationStatus {
    pub grant: String,
    pub active: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarNavigationStatusResult {
    pub active: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub enum PlanarNavigationStatusDenial {}
worth_query_structured_value_binding!(pub PlanarNavigationStatusResultBinding for PlanarNavigationStatusResult {
    identity: "worth.query.certification.planar-navigation-status-result.v1"
});
worth_query_structured_value_binding!(pub PlanarNavigationStatusDenialBinding for PlanarNavigationStatusDenial {
    identity: "worth.query.certification.planar-navigation-status-denial.v1"
});
worth_query_structured_value_binding!(pub SetPlanarNavigationStatusInput for SetPlanarNavigationStatus {
    identity: "worth.query.certification.planar-navigation-status-input.v1"
});
worth_query_operation!(pub SetPlanarNavigationStatusOperation for Schema: TopologySchemaBinding,
    input SetPlanarNavigationStatusInput);
worth_query_operation_reads!(SetPlanarNavigationStatusOperation => [PlanarNavigationGrant, PlanarNavigationGrantIdentity, PlanarNavigationStatus]);
worth_query_operation_writes!(SetPlanarNavigationStatusOperation => [PlanarNavigationStatus]);
worth_query_mutation_binding! {
    pub SetPlanarNavigationStatusBinding for SetPlanarNavigationStatus,
    schema generic Schema: TopologySchemaBinding,
    identity "worth.query.certification.planar-navigation-status.v1",
    input SetPlanarNavigationStatusInput,
    operation SetPlanarNavigationStatusOperation,
    result PlanarNavigationStatusResultBinding,
    idempotency u64, identity "worth.query.certification.planar-navigation-status-command.v1",
    decision WorthQueryInvariantMutationTarget<Schema, PlanarNavigationGrant>,
    denial PlanarNavigationStatusDenialBinding,
    handler identity "worth.query.certification.planar-navigation-status-handler.v1",
    outputs NoApplicationMutationOutputs,
    source NoApplicationMutationSource,
    principal binding ConsumerPrincipalBinding::reference(),
        binding_type ConsumerPrincipalBinding, mapping ExternalPrincipalMapping,
        principal_entity Principal, principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope PlanarNavigationGrant, PlanarNavigationGrantFacts, PlanarNavigationGrantIdentity,
        String, ReadWrite, NoApplicationUnit,
    field PlanarNavigationGrantIdentity::reference(), value_field grant,
}

pub(crate) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let operation = SetPlanarNavigationStatusOperation::reference::<S>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 16)
        .operation_read_entity(operation, PlanarNavigationGrant::reference())
        .operation_read_field(operation, PlanarNavigationGrantIdentity::reference())
        .operation_read_field(operation, PlanarNavigationStatus::reference())
        .operation_write(operation, PlanarNavigationStatus::reference())
        .application_mutation_binding::<SetPlanarNavigationStatusBinding<S>>()
}

pub(crate) struct SetPlanarNavigationStatusHandler;
impl<S: TopologySchemaBinding> OperationHandler<S, SetPlanarNavigationStatusBinding<S>>
    for SetPlanarNavigationStatusHandler
{
    fn decide(
        &self,
        input: &SetPlanarNavigationStatus,
        reader: &mut DecisionReader<'_, '_, '_, S, SetPlanarNavigationStatusBinding<S>>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<S, PlanarNavigationGrant>,
        PlanarNavigationStatusDenial,
    > {
        let grant = match reader.resolve_entity(
            PlanarNavigationGrantIdentity::reference(),
            input.grant.clone(),
        ) {
            Ok(grant) => grant,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        // The existing field's observed revision guards this ordinary write.
        if let Err(cause) = reader.field(&grant, PlanarNavigationStatus::reference()) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&grant) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &SetPlanarNavigationStatus,
        target: WorthQueryInvariantMutationTarget<S, PlanarNavigationGrant>,
        writer: &mut CandidateWriter<'_, S, SetPlanarNavigationStatusBinding<S>>,
    ) -> HandlerResult<PlanarNavigationStatusResult, PlanarNavigationStatusDenial> {
        let grant = match writer.projected_entity(&target) {
            Ok(grant) => grant,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        let status = if input.active { "active" } else { "revoked" };
        match writer.write_field(
            &grant,
            PlanarNavigationStatus::reference(),
            status.to_owned(),
        ) {
            Ok(()) => HandlerResult::Completed(PlanarNavigationStatusResult {
                active: input.active,
            }),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
