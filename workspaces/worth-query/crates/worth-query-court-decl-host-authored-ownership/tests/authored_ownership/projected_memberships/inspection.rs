//! A normal authenticated operation inspects the real Native component answer.
use super::{super::*, schema::*};
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Input {
    pub id: u64,
}
worth_query_structured_value_binding!(pub InputBinding for Input { identity: "ownership.case-inspection.input.v1" });
worth_query_structured_value_binding!(pub CasesBinding for Vec<String> { identity: "ownership.case-inspection.result.v1" });
worth_query_operation!(pub Operation for S: Scenario, input InputBinding);
impl OperationReads<Operation> for Node {}
impl OperationReads<Operation> for Id {}
impl OperationReads<Operation> for CaseRoots {}
impl OperationReads<Operation> for CaseAdjacency {}
worth_query_mutation_binding!(
    pub Binding for Input, schema generic S: Scenario,
    identity "ownership.case-inspection.v1", input InputBinding, operation Operation,
    result CasesBinding, idempotency String, identity "ownership.case-inspection.key.v1",
    decision Vec<String>, denial query::UnitBinding, handler identity "ownership.case-inspection.handler.v1",
    outputs NoApplicationMutationOutputs, source NoApplicationMutationSource,
    principal binding actor(), binding_type Actor, mapping Mapping, principal_entity Node,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Node, Facts, Id, u64, ReadOnly, NoApplicationUnit,
    field Id::reference(), value_field id,
);
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let op = Operation::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_read_entity(op, Node::reference())
        .operation_read_field(op, Id::reference())
        .operation_read_derived_relation(op, CaseRoots::reference())
        .operation_read_derived_relation(op, CaseAdjacency::reference())
        .application_mutation_binding::<Binding<S>>()
}
pub struct Handler;
impl<S: Scenario> OperationHandler<S, Binding<S>> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, S, Binding<S>>,
    ) -> HandlerResult<Vec<String>, ()> {
        let result = (|| {
            let carrier = reader.resolve_entity(Id::reference(), input.id)?;
            let root = reader
                .entity_member(&carrier)
                .map_err(HandlerExecutionDenial::new)?;
            let cases = reader
                .derived_member_relations_from(CaseRoots::reference(), &root)
                .map_err(HandlerExecutionDenial::new)?;
            if cases.len() != 2 {
                return Err(HandlerExecutionDenial::new(std::io::Error::other(
                    "case source is incomplete",
                )));
            }
            let alpha = cases
                .iter()
                .find(|edge| edge.to().case() == Some("Alpha"))
                .ok_or_else(|| {
                    HandlerExecutionDenial::new(std::io::Error::other("Alpha case is absent"))
                })?
                .to();
            let component = reader
                .graph_component_of(
                    CaseComponents::reference(),
                    CaseAdjacency::reference(),
                    alpha,
                )
                .map_err(HandlerExecutionDenial::new)?;
            let mut names = Vec::new();
            for member in component.members() {
                if member.carrier_id() != root.carrier_id() {
                    return Err(HandlerExecutionDenial::new(std::io::Error::other(
                        "component contains another carrier",
                    )));
                }
                names.push(
                    member
                        .case()
                        .ok_or_else(|| {
                            HandlerExecutionDenial::new(std::io::Error::other(
                                "component contains a physical member",
                            ))
                        })?
                        .to_owned(),
                );
            }
            Ok(names)
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Input,
        decision: Vec<String>,
        _: &mut CandidateWriter<'_, S, Binding<S>>,
    ) -> HandlerResult<Vec<String>, ()> {
        HandlerResult::Completed(decision)
    }
}
pub fn inspect<S: Scenario>(
    host: &WorthQueryProgramApplicationRuntime<S, program::Program>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    id: u64,
    key: &str,
) {
    let outcome = request
        .advanced()
        .mutate(Input { id })
        .without_source()
        .idempotency(&key.to_owned())
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert_eq!(
        outcome.result(),
        Some(&vec!["Alpha".to_owned(), "Omega".to_owned()]),
        "{outcome:?}"
    );
}
