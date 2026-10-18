//! Same pure scalar and outcome source as the consumer fixture.
#[path = "../../../../../../worth-query-certification/fixtures/consumer_values/src/planar_operation/outcome.rs"]
mod outcome;
#[path = "../../../../../../worth-query-certification/fixtures/consumer_values/src/positive_length.rs"]
mod positive_length;
pub(super) use outcome::{PlanarAdjustmentResult, PlanarMutationDenial};
pub(super) use positive_length::PositiveLength;
