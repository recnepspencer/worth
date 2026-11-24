use std::sync::Arc;

use crate::authority::commit::preparation::packets::invariant::InvariantWorkPacket;
use crate::authority::commit::preparation::planning::context::PreparationPlanningContext;
use worth_relational_contracts::internal_port::transactions::data::preparation_strategy::PreparationStrategy;

#[derive(Clone)]
pub(crate) struct PreparedInvariantExecution<'runtime> {
    pub(crate) context: Arc<PreparationPlanningContext>,
    pub(crate) strategy: PreparationStrategy,
    pub(crate) packets: Vec<InvariantWorkPacket<'runtime>>,
}
