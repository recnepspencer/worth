//! An adjustment whose access is only the shared posting fragment, included
//! from its sibling module. `probe` makes the handler also touch the bin's
//! `label`, which that fragment does not declare.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_schema::UnitApplicationValueBinding;
use worth_query_decl::facade::WorthQueryAction;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget, WorthQueryOperationBodyDenial,
};

use super::schema::*;

/// `probe` value: also read the undeclared `label` while deciding.
pub const READ_LABEL: u64 = 1;
/// `probe` value: also write the undeclared `label` while building.
pub const WRITE_LABEL: u64 = 2;

#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = StockSchema, context = Desk, namespace = STOCK, member,
    input = AdjustInput, operation = AdjustOp, binding = AdjustBinding, delivery = Adjusted,
    result = UnitApplicationValueBinding, decision = AdjustDecision, program,
    access(include: [super::posting::stock_posting_access];)
)]
pub struct Adjust {
    #[worth_query(scope = BinId)]
    pub bin: u64,
    pub add: u64,
    pub probe: u64,
}

pub struct AdjustDecision {
    target: WorthQueryInvariantMutationTarget<StockSchema, Bin>,
    next: u64,
}

pub struct Adjuster;

impl OperationHandler<StockSchema, AdjustBinding> for Adjuster {
    fn decide(
        &self,
        input: &Adjust,
        reader: &mut DecisionReader<'_, '_, '_, StockSchema, AdjustBinding>,
    ) -> HandlerResult<AdjustDecision, WorthQueryOperationBodyDenial> {
        let observed = reader
            .resolve_entity(BinFacts::bin_id(), input.bin)
            .and_then(|bin| {
                if input.probe == READ_LABEL {
                    reader.field(&bin, BinFacts::label())?;
                }
                let on_hand = reader.field(&bin, BinFacts::on_hand())?;
                Ok((reader.write_target(&bin)?, on_hand.unwrap_or_default()))
            });
        match observed {
            Ok((target, on_hand)) => HandlerResult::Completed(AdjustDecision {
                target,
                next: on_hand + input.add,
            }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &Adjust,
        decision: AdjustDecision,
        writer: &mut CandidateWriter<'_, StockSchema, AdjustBinding>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        let written = writer.projected_entity(&decision.target).and_then(|bin| {
            writer.write_field(&bin, BinFacts::on_hand(), decision.next)?;
            if input.probe == WRITE_LABEL {
                writer.write_field(&bin, BinFacts::label(), "relabelled".to_owned())?;
            }
            Ok(())
        });
        match written {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
