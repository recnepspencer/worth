//! The noting handler records the trusted admission instant it reads from
//! its `DecisionReader`, never the caller's clock or `SystemTime::now()`.
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryOperationBodyDenial,
};

use super::schema::*;

pub struct NoteDeskHandler;

impl OperationHandler<DeskSchema, NoteDeskBinding> for NoteDeskHandler {
    fn decide(
        &self,
        input: &NoteDesk,
        reader: &mut DecisionReader<'_, '_, '_, DeskSchema, NoteDeskBinding>,
    ) -> HandlerResult<NoteDecision, WorthQueryOperationBodyDenial> {
        let noted_at = match reader.admitted_at() {
            Ok(noted_at) => noted_at,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        let target = reader
            .resolve_entity(DeskFacts::desk_id(), input.desk)
            .and_then(|desk| reader.write_target(&desk));
        match target {
            Ok(target) => HandlerResult::Completed(NoteDecision { target, noted_at }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        _input: &NoteDesk,
        decision: NoteDecision,
        writer: &mut CandidateWriter<'_, DeskSchema, NoteDeskBinding>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        let written = writer
            .projected_entity(&decision.target)
            .and_then(|desk| writer.write_field(&desk, DeskFacts::noted_at(), decision.noted_at));
        match written {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
