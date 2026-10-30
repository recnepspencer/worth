//! Decision tokens retain the genuine source scope and persistent component answer.
pub use super::super::operation::GraphDenial;
pub use super::operation::*;
pub use super::*;
pub struct Handler;
impl<L: LawPolicy> OperationHandler<GraphSchema<L>, CheckBinding<L>> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, CheckBinding<L>>,
    ) -> HandlerResult<Checked, GraphDenial> {
        match gather(input, reader) {
            Ok(result) => HandlerResult::Completed(result),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Input,
        result: Checked,
        _: &mut CandidateWriter<'_, GraphSchema<L>, CheckBinding<L>>,
    ) -> HandlerResult<Checked, GraphDenial> {
        HandlerResult::Completed(result)
    }
}
pub fn gather<L: LawPolicy>(
    input: &Input,
    reader: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, CheckBinding<L>>,
) -> Result<Checked, HandlerExecutionDenial> {
    let entity = reader.resolve_entity(NodeKey::reference(), input.root)?;
    let root = reader
        .entity_member(&entity)
        .map_err(HandlerExecutionDenial::new)?;
    let cases = reader
        .derived_member_relations_from(RootUses::reference(), &root)
        .map_err(HandlerExecutionDenial::new)?;
    assert_eq!(cases.len(), 2);
    let negative = cases
        .iter()
        .find(|edge| edge.to().case() == Some("Negative"))
        .unwrap()
        .to();
    assert!(negative.as_entity().is_none());
    let walk = reader
        .derived_walk(
            UseSuccessor::reference(),
            RelationDirection::Forward,
            negative,
        )
        .map_err(HandlerExecutionDenial::new)?;
    let component = reader
        .graph_component_of(
            UseComponents::reference(),
            UseSuccessor::reference(),
            negative,
        )
        .map_err(HandlerExecutionDenial::new)?;
    let members = component
        .members()
        .iter()
        .map(|member| (member.carrier_id(), member.case().unwrap().to_owned()))
        .collect();
    Ok(Checked {
        members,
        distances: walk.steps().iter().map(|step| step.distance()).collect(),
        identity: *component.identity().bytes(),
    })
}
