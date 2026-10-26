//! One ordinary field-writing handler whose decision reads a complete path.
use super::super::schema::*;
use worth_query_declaration::{
    facade::{application_operation::NoApplicationMutationOutputs, application_schema::*},
    *,
};
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct CheckWalkInput {
    pub root: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckWalkOutcome(pub u64);
pub struct CheckWalkDecision {
    pub root: u64,
    pub visited: u64,
}
worth_query_portable_type!(CheckWalkInput => "worth.query.test.check-walk-input.v1");
worth_query_structured_value_binding!(pub CheckWalkInputBinding for CheckWalkInput {identity:"worth.query.test.check-walk-input.v1"});
worth_query_portable_type!(CheckWalkOutcome => "worth.query.test.check-walk-outcome.v1");
worth_query_structured_value_binding!(pub CheckWalkOutcomeBinding for CheckWalkOutcome {identity:"worth.query.test.check-walk-outcome.v1"});
worth_query_operation!(pub CheckWalk for WalkSchema,input CheckWalkInputBinding);
worth_query_operation_requires!(CheckWalk => [EditNode]);
worth_query_operation_reads!(CheckWalk => [Node,NodeKey,Active,Next]);
worth_query_operation_expects_fact!(CheckWalk => [Active]);
worth_query_operation_writes!(CheckWalk => [Active]);
worth_query_mutation_binding!(pub CheckWalkBinding for CheckWalkInput,
    schema WalkSchema,identity "worth.query.test.check-walk-mutation.v1",
    input CheckWalkInputBinding,operation CheckWalk,result CheckWalkOutcomeBinding,
    idempotency String,identity "worth.query.test.check-walk-key.v1",
    decision CheckWalkDecision,denial CheckWalkOutcomeBinding,
    handler identity "worth.query.test.check-walk-handler.v1",
    outputs NoApplicationMutationOutputs,
    principal IdentityBinding,mapping ExternalMapping,principal_entity Principal,
    principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Node,NodeData,NodeKey,u64,ReadOnly,NoApplicationUnit,
    field NodeKey::reference(),value scope_key,
);
fn scope_key(input: &CheckWalkInput) -> u64 {
    input.root
}
pub(in super::super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<WalkSchema>,
) -> ApplicationSchemaDeclarationBuilder<WalkSchema> {
    schema
        .operation(
            CheckWalk::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_requires_ability(CheckWalk::reference(), EditNode::reference())
        .operation_read_entity(CheckWalk::reference(), Node::reference())
        .operation_read_field(CheckWalk::reference(), NodeKey::reference())
        .operation_read_field(CheckWalk::reference(), Active::reference())
        .operation_read_relation(CheckWalk::reference(), Next::reference())
        .operation_expected_fact(CheckWalk::reference(), Active::reference())
        .operation_write(CheckWalk::reference(), Active::reference())
        .application_mutation_binding::<CheckWalkBinding>()
}
