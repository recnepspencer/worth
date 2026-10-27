//! Ordinary authored links between rings, independent of their successor order.
use super::*;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_mutation_binding,
    worth_query_operation, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_unlinks, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget;
mod handler;
pub(crate) use handler::PlanarIslandCouplingHandler;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarIslandCoupling {
    pub scope_key: String,
    pub target_key: String,
    pub linked: bool,
}
#[derive(Debug, Eq, PartialEq)]
pub struct PlanarIslandCouplingResult;
#[derive(Debug, Eq, PartialEq)]
pub enum PlanarIslandCouplingDenial {
    SameVertex,
    AlreadyInRequestedState,
}
worth_query_structured_value_binding!(pub PlanarIslandCouplingInputBinding for PlanarIslandCoupling {
    identity: "worth.query.certification.planar-island-coupling-input.v1"
});
worth_query_structured_value_binding!(pub PlanarIslandCouplingResultBinding for PlanarIslandCouplingResult {
    identity: "worth.query.certification.planar-island-coupling-result.v1"
});
worth_query_structured_value_binding!(pub PlanarIslandCouplingDenialBinding for PlanarIslandCouplingDenial {
    identity: "worth.query.certification.planar-island-coupling-denial.v1"
});
worth_query_operation!(pub EditPlanarIslandCoupling for S: TopologySchemaBinding, input PlanarIslandCouplingInputBinding);
worth_query_operation_reads!(EditPlanarIslandCoupling => [Body, BodyKey, PlanarCoupling]);
worth_query_operation_links!(EditPlanarIslandCoupling => [PlanarCoupling]);
worth_query_operation_unlinks!(EditPlanarIslandCoupling => [PlanarCoupling]);
worth_query_mutation_binding! {
    pub PlanarIslandCouplingBinding for PlanarIslandCoupling,
    schema generic S: TopologySchemaBinding,
    identity "worth.query.certification.planar-island-coupling.v1",
    input PlanarIslandCouplingInputBinding, operation EditPlanarIslandCoupling,
    result PlanarIslandCouplingResultBinding,
    idempotency u64, identity "worth.query.certification.planar-island-coupling-command.v1",
    decision [WorthQueryInvariantMutationTarget<S, Body>; 2], denial PlanarIslandCouplingDenialBinding,
    handler identity "worth.query.certification.planar-island-coupling-handler.v1",
    outputs NoApplicationMutationOutputs, source NoApplicationMutationSource,
    principal binding ConsumerPrincipalBinding::reference(),
        binding_type ConsumerPrincipalBinding, mapping ExternalPrincipalMapping,
        principal_entity Principal, principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope Body, PlanarPosition, BodyKey, String, ReadOnly, NoApplicationUnit,
    field BodyKey::reference(), value_field scope_key,
}

pub(super) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let operation = EditPlanarIslandCoupling::reference::<S>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 128)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_relation(operation, PlanarCoupling::reference())
        .operation_link(operation, PlanarCoupling::reference())
        .operation_unlink(operation, PlanarCoupling::reference())
        .application_mutation_binding::<PlanarIslandCouplingBinding<S>>()
}
