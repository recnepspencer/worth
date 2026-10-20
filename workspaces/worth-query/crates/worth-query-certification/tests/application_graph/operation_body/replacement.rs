//! An attempted host replacement must be refused before any method can run.
use super::{binding::BodyBinding, schema::*};
use worth_query_host::facade::{
    declaration::application_operation::ApplicationCandidateRequirements,
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerResult, OperationHandler,
        WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial,
    },
};

pub struct Replacement;
impl<const DESTINATION: u64>
    OperationHandler<DocumentRetentionSchema<DESTINATION>, BodyBinding<DESTINATION>>
    for Replacement
{
    fn decide(
        &self,
        _: &SetRetentionInput,
        _: &mut DecisionReader<
            '_,
            '_,
            '_,
            DocumentRetentionSchema<DESTINATION>,
            BodyBinding<DESTINATION>,
        >,
    ) -> HandlerResult<
        WorthQueryOperationBodyDecision<DocumentRetentionSchema<DESTINATION>, Document>,
        WorthQueryOperationBodyDenial,
    > {
        panic!("a host replacement must never execute")
    }
    fn candidate_requirements(
        &self,
        _: &SetRetentionInput,
        _: &WorthQueryOperationBodyDecision<DocumentRetentionSchema<DESTINATION>, Document>,
    ) -> ApplicationCandidateRequirements {
        panic!("a host replacement must never execute")
    }
    fn build_candidate(
        &self,
        _: &SetRetentionInput,
        _: WorthQueryOperationBodyDecision<DocumentRetentionSchema<DESTINATION>, Document>,
        _: &mut CandidateWriter<'_, DocumentRetentionSchema<DESTINATION>, BodyBinding<DESTINATION>>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        panic!("a host replacement must never execute")
    }
}
