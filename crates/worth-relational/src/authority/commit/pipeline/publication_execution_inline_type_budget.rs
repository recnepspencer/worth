//! Unit-only layout evidence; constructs no runtime, proof or authority.

use worth_relational_contracts::internal_port::commit_strategies::data::lowering_meaning::StrategyLoweringErrorOwnerPort as _;
pub(crate) fn collect(census: &mut crate::inline_type_sizes::census::Census) {
    super::preparation::inline_type_budget::collect(census);
}
