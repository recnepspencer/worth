//! A shipment whose access is two shared fragments and nothing inline; they
//! overlap on the bin's on-hand read and write.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_schema::UnitApplicationValueBinding;
use worth_query_decl::facade::WorthQueryAction;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey, WorthQueryInvariantMutationTarget,
    WorthQueryOperationBodyDenial,
};

use super::schema::*;

#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = StockSchema, context = Desk, namespace = STOCK, member,
    input = ShipInput, operation = ShipOp, binding = ShipBinding, delivery = Shipped,
    result = UnitApplicationValueBinding, decision = ShipDecision, program,
    access(include: [
        super::posting::stock_posting_access,
        super::posting::movement_record_access,
    ];)
)]
pub struct Ship {
    #[worth_query(scope = BinId)]
    pub bin: u64,
    pub quantity: u64,
    pub movement: u64,
}

pub struct ShipDecision {
    target: WorthQueryInvariantMutationTarget<StockSchema, Bin>,
    remaining: u64,
}

pub struct Shipper;

impl OperationHandler<StockSchema, ShipBinding> for Shipper {
    fn decide(
        &self,
        input: &Ship,
        reader: &mut DecisionReader<'_, '_, '_, StockSchema, ShipBinding>,
    ) -> HandlerResult<ShipDecision, WorthQueryOperationBodyDenial> {
        let observed = reader
            .resolve_entity(BinFacts::bin_id(), input.bin)
            .and_then(|bin| {
                let on_hand = reader.field(&bin, BinFacts::on_hand())?;
                Ok((reader.write_target(&bin)?, on_hand.unwrap_or_default()))
            });
        match observed {
            Ok((target, on_hand)) => HandlerResult::Completed(ShipDecision {
                target,
                remaining: on_hand.saturating_sub(input.quantity),
            }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &Ship,
        decision: ShipDecision,
        writer: &mut CandidateWriter<'_, StockSchema, ShipBinding>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        let key = format!("movement-{}", input.movement);
        let entity_key =
            WorthQueryApplicationEntityKey::new(key.clone()).expect("movement keys are valid");
        let written = writer.projected_entity(&decision.target).and_then(|bin| {
            writer.write_field(&bin, BinFacts::on_hand(), decision.remaining)?;
            let movement =
                writer.create_entity_in_context(&bin, Movement::reference(), entity_key)?;
            writer.initialize_field(&movement, MovementFacts::movement_id(), input.movement)?;
            writer.initialize_field(&movement, MovementFacts::moved(), input.quantity)?;
            writer.link(BinMovements::reference(), key, &bin, &movement)
        });
        match written {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
