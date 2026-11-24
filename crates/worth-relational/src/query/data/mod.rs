pub(crate) mod index_parity_outcome;
pub(crate) mod query_outcome;
pub(crate) mod snapshot_pinned_plan;
pub use worth_relational_contracts::internal_port::query::data::*;

#[cfg(test)]
pub(crate) use worth_relational_contracts::internal_port::query::data::reduction_oracle::reduce_query_fragments_for_test;
