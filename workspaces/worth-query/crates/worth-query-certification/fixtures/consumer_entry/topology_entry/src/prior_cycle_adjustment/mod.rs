mod binding;
mod declaration;
mod handler;
mod identity;

pub use binding::PriorCycleAdjustmentBinding;
pub(crate) use declaration::declare_prior_cycle_adjustment;
pub use handler::PriorCycleAdjustmentHandler;

use super::{Body, BodyKey, PositionY, TopologySchemaBinding};
use worth_query_consumer_values::PositiveLength;
use worth_query_decl::facade::{
    worth_query_operation, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriorCycleAdjustment {
    pub scope_key: String,
    pub offset_y: PositiveLength,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriorCycleAdjustmentResult {
    pub adjusted_roles: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PriorCycleAdjustmentDenial {
    MissingCoordinate,
    NoPriorCycle,
}

worth_query_structured_value_binding!(pub PriorCycleAdjustmentInputBinding for PriorCycleAdjustment {
    identity: "worth.query.certification.prior-cycle-adjustment-input.v1"
});
worth_query_structured_value_binding!(pub PriorCycleAdjustmentResultBinding for PriorCycleAdjustmentResult {
    identity: "worth.query.certification.prior-cycle-adjustment-result.v1"
});
worth_query_structured_value_binding!(pub PriorCycleAdjustmentDenialBinding for PriorCycleAdjustmentDenial {
    identity: "worth.query.certification.prior-cycle-adjustment-denial.v1"
});
worth_query_operation!(pub AdjustPriorCycle for Schema: TopologySchemaBinding, input PriorCycleAdjustmentInputBinding);
worth_query_operation_reads!(AdjustPriorCycle => [Body, BodyKey, PositionY]);
worth_query_operation_writes!(AdjustPriorCycle => [PositionY]);
