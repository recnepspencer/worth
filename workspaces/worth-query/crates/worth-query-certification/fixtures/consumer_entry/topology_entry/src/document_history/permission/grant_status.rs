//! Ordinary fixture administration changes a live grant through its real owner.
use super::*;
use worth_query_decl::facade::{
    application_operation::*, worth_query_mutation_binding, worth_query_operation,
    worth_query_operation_reads, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct SetPlanarHistoryStatus {
    pub grant: String,
    pub active: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarHistoryStatusResult {
    pub active: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub enum PlanarHistoryStatusDenial {}
worth_query_structured_value_binding!(pub PlanarHistoryStatusResultBinding for PlanarHistoryStatusResult {
    identity: "worth.query.certification.planar-navigation-status-result.v1"
});
worth_query_structured_value_binding!(pub PlanarHistoryStatusDenialBinding for PlanarHistoryStatusDenial {
    identity: "worth.query.certification.planar-navigation-status-denial.v1"
});
worth_query_structured_value_binding!(pub SetPlanarHistoryStatusInput for SetPlanarHistoryStatus {
    identity: "worth.query.certification.planar-navigation-status-input.v1"
});
worth_query_operation!(pub SetPlanarHistoryStatusOperation for Schema: TopologySchemaBinding,
    input SetPlanarHistoryStatusInput);
worth_query_operation_reads!(SetPlanarHistoryStatusOperation => [PlanarHistoryGrant]);

worth_query_mutation_binding! {
    pub SetPlanarHistoryStatusBinding for SetPlanarHistoryStatus,
    schema generic Schema: TopologySchemaBinding,
    identity "worth.query.certification.planar-navigation-status.v1",
    input SetPlanarHistoryStatusInput,
    operation SetPlanarHistoryStatusOperation,
    result PlanarHistoryStatusResultBinding,
    idempotency u64, identity "worth.query.certification.planar-navigation-status-command.v1",
    decision WorthQueryInvariantMutationTarget<Schema, PlanarHistoryGrant>,
    denial PlanarHistoryStatusDenialBinding,
    handler identity "worth.query.certification.planar-navigation-status-handler.v1",
    outputs NoApplicationMutationOutputs,
    source NoApplicationMutationSource,
    principal binding ConsumerPrincipalBinding::reference(),
        binding_type ConsumerPrincipalBinding, mapping ExternalPrincipalMapping,
        principal_entity Principal, principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope PlanarHistoryGrantIdentity,
    field PlanarHistoryGrantIdentity::at("PlanarHistoryGrantIdentity"), value_field grant,
}

pub(crate) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let operation = SetPlanarHistoryStatusOperation::reference::<S>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 16)
        .operation_read_entity(operation, PlanarHistoryGrant::reference())
        .operation_read_field(
            operation,
            PlanarHistoryGrantIdentity::at("PlanarHistoryGrantIdentity"),
        )
        .operation_read_field(operation, PlanarHistoryStatus::at("PlanarHistoryStatus"))
        .operation_write(operation, PlanarHistoryStatus::at("PlanarHistoryStatus"))
        .application_mutation_binding::<SetPlanarHistoryStatusBinding<S>>()
}

pub struct SetPlanarHistoryStatusHandler;
impl<S: TopologySchemaBinding> OperationHandler<S, SetPlanarHistoryStatusBinding<S>>
    for SetPlanarHistoryStatusHandler
{
    fn decide(
        &self,
        input: &SetPlanarHistoryStatus,
        reader: &mut DecisionReader<'_, '_, '_, S, SetPlanarHistoryStatusBinding<S>>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<S, PlanarHistoryGrant>,
        PlanarHistoryStatusDenial,
    > {
        let grant = match reader.resolve_entity(
            PlanarHistoryGrantIdentity::at("PlanarHistoryGrantIdentity"),
            input.grant.clone(),
        ) {
            Ok(grant) => grant,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        // The existing field's observed revision guards this ordinary write.
        if let Err(cause) = reader.field(&grant, PlanarHistoryStatus::at("PlanarHistoryStatus")) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&grant) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &SetPlanarHistoryStatus,
        target: WorthQueryInvariantMutationTarget<S, PlanarHistoryGrant>,
        writer: &mut CandidateWriter<'_, S, SetPlanarHistoryStatusBinding<S>>,
    ) -> HandlerResult<PlanarHistoryStatusResult, PlanarHistoryStatusDenial> {
        let grant = match writer.projected_entity(&target) {
            Ok(grant) => grant,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        let status = if input.active { "active" } else { "revoked" };
        match writer.write_field(
            &grant,
            PlanarHistoryStatus::at("PlanarHistoryStatus"),
            status.to_owned(),
        ) {
            Ok(()) => HandlerResult::Completed(PlanarHistoryStatusResult {
                active: input.active,
            }),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
