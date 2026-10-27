//! Count is gathered under the actual admitted reader, computed and completed normally.
use super::super::operation::GraphDenial;
use super::operation::*;
use super::*;
pub(super) struct Handler<L: LawPolicy, Owner>(
    pub(super)  WorthQueryInstalledPartitionedComputation<
        GraphSchema<L>,
        GraphFeature,
        collection::CountComputation,
        Owner,
    >,
)
where
    Owner: WorthQueryPartitionedComputationOwner<
        GraphSchema<L>,
        GraphFeature,
        collection::CountComputation,
        Operation = CountTwoSteps,
        Output = u64,
        Stopped = GraphDenial,
    >;
impl<L: LawPolicy, Owner> OperationHandler<GraphSchema<L>, CountBinding<L>> for Handler<L, Owner>
where
    Owner: WorthQueryPartitionedComputationOwner<
        GraphSchema<L>,
        GraphFeature,
        collection::CountComputation,
        Operation = CountTwoSteps,
        Output = u64,
        Stopped = GraphDenial,
    >,
{
    fn decide(
        &self,
        _: &Input,
        _: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, CountBinding<L>>,
    ) -> HandlerResult<u64, GraphDenial> {
        HandlerResult::DomainDenied(GraphDenial)
    }
    fn prepare_decision(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, CountBinding<L>>,
    ) -> WorthQueryDecisionPlan<u64, GraphDenial> {
        let root = match reader.resolve_entity(NodeKey::reference(), input.root) {
            Ok(root) => root,
            Err(cause) => {
                return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(cause))
            }
        };
        if input.bad_reference {
            let hostile=ApplicationDerivedRelationRef::<GraphSchema<L>,TwoSteps,Node,Node>::from_schema_identifiers(
                "TwoSteps","Leaf","Node");
            assert!(reader.derived_relations_from(hostile, &root).is_err());
            // Deliberately swallow the actual failed source; seal must still veto it.
            return WorthQueryDecisionPlan::immediate(HandlerResult::Completed(0));
        }
        match self.0.prepare(reader, &root) {
            Ok(prepared) => prepared.into_decision_plan(|computed| {
                match computed.and_then(|run| run.complete()) {
                    Ok(count) => HandlerResult::Completed(count),
                    Err(cause) => {
                        HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
                    }
                }
            }),
            Err(cause) => WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(
                HandlerExecutionDenial::new(cause),
            )),
        }
    }
    fn build_candidate(
        &self,
        _: &Input,
        count: u64,
        _: &mut CandidateWriter<'_, GraphSchema<L>, CountBinding<L>>,
    ) -> HandlerResult<Counted, GraphDenial> {
        HandlerResult::Completed(Counted { count })
    }
}
