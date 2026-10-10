//! Ordinary no-source input with a real one-field candidate and discovered tree.

use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_mutation_binding,
    worth_query_operation, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget;

use super::*;

mod connection;
mod handler;
pub use connection::PlanarInitialToOutputConnection;
pub use handler::PlanarInitialAdjustmentHandler;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PlanarInitialAdjustment {
    pub scope_key: String,
    pub replacement_y: PositiveLength,
}

worth_query_structured_value_binding!(pub PlanarInitialAdjustmentInputBinding for PlanarInitialAdjustment {
    identity: "worth.query.certification.planar-initial-adjustment-input.v1"
});
worth_query_operation!(pub AdjustInitialPlanar for Schema: TopologySchemaBinding, input PlanarInitialAdjustmentInputBinding);
worth_query_operation_reads!(AdjustInitialPlanar => [Body, BodyKey, PositionY]);
worth_query_operation_writes!(AdjustInitialPlanar => [PositionY]);

worth_query_mutation_binding! {
    pub PlanarInitialAdjustmentBinding for PlanarInitialAdjustment,
    schema generic Schema: TopologySchemaBinding,
    identity "worth.query.certification.planar-initial-adjustment.v1",
    input PlanarInitialAdjustmentInputBinding,
    operation AdjustInitialPlanar,
    result PlanarSourceAdjustmentResultBinding,
    idempotency u64, identity "worth.query.certification.planar-initial-adjustment-command.v1",
    decision WorthQueryInvariantMutationTarget<Schema, Body>, denial PlanarSourceAdjustmentDenialBinding,
    handler identity "worth.query.certification.planar-initial-adjustment-handler.v1",
    outputs NoApplicationMutationOutputs,
    source NoApplicationMutationSource,
    principal binding ConsumerPrincipalBinding::reference(),
        binding_type ConsumerPrincipalBinding, mapping ExternalPrincipalMapping,
        principal_entity Principal, principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope Body, PlanarPosition, BodyKey, String, ReadOnly, NoApplicationUnit,
    field BodyKey::reference(), value_field scope_key,
    candidates creates 0, deletes 0, links 0, unlinks 0, writes 1, emits 0,
    resources retained_representation_bytes 1024
}

pub(crate) fn declare_initial_adjustment<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = AdjustInitialPlanar::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 16)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, PositionY::reference())
        .operation_write(operation, PositionY::reference())
        .application_mutation_binding::<PlanarInitialAdjustmentBinding<Schema>>()
}
