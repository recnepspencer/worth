//! Same pure scalar and adjustment result source as the consumer fixture, with
//! the oracle's own planar denial vocabulary.

pub use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::fixture_values::outcome::PlanarAdjustmentResult;
pub use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::fixture_values::positive_length::PositiveLength;

/// Why a planar mutation of this oracle cannot produce a candidate: its region
/// output's scope has no Length. The consumer fixture's full planar denial
/// vocabulary belongs to its planar handlers, which this oracle never runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanarMutationDenial {
    MissingCoordinate,
}

#[path = "../../../../../../../worth-query-certification/fixtures/consumer_values/src/planar_operation/outcome.rs"]
pub mod outcome;
#[path = "../../../../../../../worth-query-certification/fixtures/consumer_values/src/positive_length.rs"]
pub mod positive_length;
