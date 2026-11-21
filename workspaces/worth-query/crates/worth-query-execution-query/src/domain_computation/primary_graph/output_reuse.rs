pub use worth_query_execution_lineage::facade::primary_graph::output_reuse::verification_denial as verification_denial_mod;
pub use verification_denial_mod::verification_denial;
mod dependency;
pub use worth_query_execution_lineage::facade::primary_graph::output_reuse::restored_facts;
pub use worth_query_execution_lineage::facade::primary_graph::output_reuse::selection;
pub use worth_query_execution_lineage::facade::primary_graph::output_reuse::verification_extent;
pub use restored_facts::{evidence_answer, evidence_unavailable, restored_facts_hold};
pub use verification_extent::RetainedFactVerification;

pub use dependency::require_installed_output_dependencies;
pub use selection::{
    compare_retained_output_dependencies, compare_retained_output_witness, first_moved_fact,
    first_moved_fact_in, interruption_denial, memory_denial, read_denial,
    retained_output_settlement_is_verified, OutputDependencySelection, RetainedFactStop,
};

pub use selection::{
    owner_read, preflight_stop, retained_fact_denial,
};

mod historical_selection;
pub use historical_selection::HistoricalOutputRead;
