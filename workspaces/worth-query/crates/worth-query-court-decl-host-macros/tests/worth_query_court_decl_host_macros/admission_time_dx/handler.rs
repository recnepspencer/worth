//! The stamping handler: it records the caller's dates and the trusted
//! admission instant it reads from its `DecisionReader`, never the caller's
//! clock or `SystemTime::now()`.
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryOperationBodyDenial,
};

use super::schema::*;

pub struct StampLotHandler;

type Denial = WorthQueryOperationBodyDenial;

impl OperationHandler<DatedSchema, StampLotBinding> for StampLotHandler {
    fn decide(
        &self,
        input: &StampLot,
        reader: &mut DecisionReader<'_, '_, '_, DatedSchema, StampLotBinding>,
    ) -> HandlerResult<StampDecision, Denial> {
        let stamped_at = match (reader.admitted_at(), reader.admitted_at()) {
            (Ok(first), Ok(again)) if first == again => first,
            (Ok(first), Ok(again)) => {
                return HandlerResult::domain_refusal(
                    "admission_time_moved",
                    &format!("one decision read {first:?} and then {again:?}"),
                )
            }
            (Err(cause), _) | (_, Err(cause)) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        let target = reader
            .resolve_entity(LotFacts::lot_id(), input.lot)
            .and_then(|lot| reader.write_target(&lot));
        match target {
            Ok(target) => HandlerResult::Completed(StampDecision { target, stamped_at }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &StampLot,
        decision: StampDecision,
        writer: &mut CandidateWriter<'_, DatedSchema, StampLotBinding>,
    ) -> HandlerResult<(), Denial> {
        let written = writer.projected_entity(&decision.target).and_then(|lot| {
            writer.write_field(&lot, LotFacts::effective_on(), input.effective_on)?;
            writer.write_field(&lot, LotFacts::expires_on(), input.expires_on)?;
            writer.write_field(&lot, LotFacts::stamped_at(), decision.stamped_at)
        });
        match written {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
