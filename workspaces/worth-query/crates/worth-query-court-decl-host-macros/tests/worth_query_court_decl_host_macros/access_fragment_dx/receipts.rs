//! A receipt that includes the shared posting fragment and adds its own
//! inline access: the lot it books, a movement it creates and links, and a
//! repeat of the fragment's on-hand read and write.
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
    input = ReceiveInput, operation = ReceiveOp, binding = ReceiveBinding, delivery = Received,
    result = UnitApplicationValueBinding, decision = ReceiveDecision, program,
    access(
        include: [super::posting::stock_posting_access];
        read_entities: [Lot];
        read_fields: [LotFacts::lot_id(), LotFacts::lot_quantity(), BinFacts::on_hand()];
        writes: [BinFacts::on_hand(), MovementFacts::movement_id(), MovementFacts::moved()];
        creates: [Movement];
        links: [BinMovements];
    )
)]
pub struct Receive {
    #[worth_query(scope = BinId)]
    pub bin: u64,
    pub lot: u64,
    pub movement: u64,
}

pub struct ReceiveDecision {
    target: WorthQueryInvariantMutationTarget<StockSchema, Bin>,
    on_hand: u64,
    quantity: u64,
}

pub struct Receiver;

impl OperationHandler<StockSchema, ReceiveBinding> for Receiver {
    fn decide(
        &self,
        input: &Receive,
        reader: &mut DecisionReader<'_, '_, '_, StockSchema, ReceiveBinding>,
    ) -> HandlerResult<ReceiveDecision, WorthQueryOperationBodyDenial> {
        let observed = reader
            .resolve_entity(BinFacts::bin_id(), input.bin)
            .and_then(|bin| {
                let lot = reader.resolve_entity(LotFacts::lot_id(), input.lot)?;
                let quantity = reader.field(&lot, LotFacts::lot_quantity())?;
                let on_hand = reader.field(&bin, BinFacts::on_hand())?;
                Ok(ReceiveDecision {
                    target: reader.write_target(&bin)?,
                    on_hand: on_hand.unwrap_or_default(),
                    quantity: quantity.unwrap_or_default(),
                })
            });
        match observed {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &Receive,
        decision: ReceiveDecision,
        writer: &mut CandidateWriter<'_, StockSchema, ReceiveBinding>,
    ) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
        let key = format!("movement-{}", input.movement);
        let entity_key =
            WorthQueryApplicationEntityKey::new(key.clone()).expect("movement keys are valid");
        let written = writer.projected_entity(&decision.target).and_then(|bin| {
            let total = decision.on_hand + decision.quantity;
            writer.write_field(&bin, BinFacts::on_hand(), total)?;
            let movement =
                writer.create_entity_in_context(&bin, Movement::reference(), entity_key)?;
            writer.initialize_field(&movement, MovementFacts::movement_id(), input.movement)?;
            writer.initialize_field(&movement, MovementFacts::moved(), decision.quantity)?;
            writer.link(BinMovements::reference(), key, &bin, &movement)
        });
        match written {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
