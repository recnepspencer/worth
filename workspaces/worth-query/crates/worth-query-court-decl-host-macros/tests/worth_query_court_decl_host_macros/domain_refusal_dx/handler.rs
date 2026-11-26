//! The shipping handler: it refuses an expired lot with a domain reason
//! instead of a generic rule refusal, so no caller precheck is needed.
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryOperationBodyDenial,
};

use super::schema::*;

pub const LOT_EXPIRED: &str = "lot_expired";
/// A defective handler's reason: longer than the 256-byte message bound.
pub fn mislabeled_message() -> String {
    "This lot's label does not match its contents. ".repeat(8)
}

pub fn expired_message(lot: u64) -> String {
    format!("Lot {lot} has expired and can't be shipped.")
}

pub struct ShipLotHandler;

type Denial = WorthQueryOperationBodyDenial;

impl OperationHandler<StockSchema, ShipLotBinding> for ShipLotHandler {
    fn decide(
        &self,
        input: &ShipLot,
        reader: &mut DecisionReader<'_, '_, '_, StockSchema, ShipLotBinding>,
    ) -> HandlerResult<ShipDecision, Denial> {
        let observed = reader
            .resolve_entity(LotFacts::lot_id(), input.lot)
            .and_then(|lot| {
                let target = reader.write_target(&lot)?;
                let state = reader.field(&lot, LotFacts::state())?;
                let on_hand = reader.field(&lot, LotFacts::quantity())?;
                Ok((
                    target,
                    state.unwrap_or_default(),
                    on_hand.unwrap_or_default(),
                ))
            });
        let (target, state, on_hand) = match observed {
            Ok(observed) => observed,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        match state.as_str() {
            "EXPIRED" => HandlerResult::domain_refusal(LOT_EXPIRED, &expired_message(input.lot)),
            "MISLABELED" => HandlerResult::domain_refusal("lot_mislabeled", &mislabeled_message()),
            _ => match on_hand.checked_sub(input.quantity) {
                Some(remaining) => HandlerResult::Completed(ShipDecision { target, remaining }),
                None => HandlerResult::domain_refusal(
                    "insufficient_stock",
                    &format!("Only {on_hand} units are on hand."),
                ),
            },
        }
    }

    fn build_candidate(
        &self,
        _input: &ShipLot,
        decision: ShipDecision,
        writer: &mut CandidateWriter<'_, StockSchema, ShipLotBinding>,
    ) -> HandlerResult<(), Denial> {
        let written = writer
            .projected_entity(&decision.target)
            .and_then(|lot| writer.write_field(&lot, LotFacts::quantity(), decision.remaining));
        match written {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
