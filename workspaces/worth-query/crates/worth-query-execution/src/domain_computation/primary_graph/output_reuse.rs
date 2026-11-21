pub use worth_query_execution_lineage::facade::primary_graph::output_reuse::verification_denial as verification_denial_mod;
pub(in crate::domain_computation::primary_graph) use verification_denial_mod::verification_denial;
mod dependency;
pub use worth_query_execution_lineage::facade::primary_graph::output_reuse::restored_facts;
pub(crate) use worth_query_execution_lineage::facade::primary_graph::output_reuse::selection;
pub use worth_query_execution_lineage::facade::primary_graph::output_reuse::verification_extent;
pub(super) use restored_facts::{evidence_answer, evidence_unavailable, restored_facts_hold};
pub(in crate::domain_computation::primary_graph) use verification_extent::RetainedFactVerification;

pub(super) use dependency::require_installed_output_dependencies;
pub(super) use selection::{
    compare_retained_output_dependencies, compare_retained_output_witness, first_moved_fact,
    first_moved_fact_in, interruption_denial, memory_denial, read_denial,
    retained_output_settlement_is_verified, OutputDependencySelection, RetainedFactStop,
};

pub(in crate::domain_computation::primary_graph) use selection::{
    owner_read, preflight_stop, retained_fact_denial,
};

mod historical_selection;
pub(in crate::domain_computation::primary_graph) use historical_selection::HistoricalOutputRead;
