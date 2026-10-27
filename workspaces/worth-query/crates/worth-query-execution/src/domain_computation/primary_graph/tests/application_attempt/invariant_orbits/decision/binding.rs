//! A field-writing operation whose decision depends on a persistent orbit.
use super::super::schema::*;
use worth_query_declaration::{
    facade::{application_operation::*, application_schema::*},
    *,
};
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct CheckOrbitInput {
    pub root: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckOrbitOutcome(pub u64);
pub struct CheckOrbitDecision {
    pub root: u64,
    pub count: u64,
}
worth_query_portable_type!(CheckOrbitInput => "worth.query.test.check-orbit-input.v1");
worth_query_structured_value_binding!(pub CheckOrbitInputBinding for CheckOrbitInput {identity:"worth.query.test.check-orbit-input.v1"});
worth_query_portable_type!(CheckOrbitOutcome => "worth.query.test.check-orbit-outcome.v1");
worth_query_structured_value_binding!(pub CheckOrbitOutcomeBinding for CheckOrbitOutcome {identity:"worth.query.test.check-orbit-outcome.v1"});
worth_query_operation!(pub CheckOrbit for S: OrbitModel,input CheckOrbitInputBinding);
worth_query_operation_reads!(CheckOrbit => [Node,NodeKey,Marked,Next]);
worth_query_operation_expects_fact!(CheckOrbit => [Marked]);
worth_query_operation_writes!(CheckOrbit => [Marked]);
worth_query_mutation_binding!(pub CheckOrbitBinding for CheckOrbitInput,
    schema generic S: OrbitModel, identity "worth.query.test.check-orbit-mutation.v1",
    input CheckOrbitInputBinding,operation CheckOrbit,result CheckOrbitOutcomeBinding,
    idempotency String,identity "worth.query.test.check-orbit-key.v1",
    decision CheckOrbitDecision,denial CheckOrbitOutcomeBinding,
    handler identity "worth.query.test.check-orbit-handler.v1",
    outputs NoApplicationMutationOutputs,source NoApplicationMutationSource,
    principal binding IdentityBinding::reference(),binding_type IdentityBinding,
    mapping ExternalMapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Node,NodeData,NodeKey,u64,ReadOnly,NoApplicationUnit,
    field NodeKey::reference(),value_field root,
);
pub(in super::super) fn declare<S: OrbitModel>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .operation(
            CheckOrbit::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_entity(CheckOrbit::reference(), Node::reference())
        .operation_read_field(CheckOrbit::reference(), NodeKey::reference())
        .operation_read_field(CheckOrbit::reference(), Marked::reference())
        .operation_read_relation(CheckOrbit::reference(), Next::reference())
        .operation_expected_fact(CheckOrbit::reference(), Marked::reference())
        .operation_write(CheckOrbit::reference(), Marked::reference())
        .application_mutation_binding::<CheckOrbitBinding<S>>()
}
