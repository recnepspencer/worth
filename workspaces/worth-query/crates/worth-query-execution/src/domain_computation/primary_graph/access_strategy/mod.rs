//! Managed candidate execution; legal narrowing remains a separate verified handoff.
pub(super) mod coverage;
pub(super) mod ordered_storage;
pub(super) use worth_query_execution_lineage::facade::primary_graph::access_strategy::realized_footprint;
