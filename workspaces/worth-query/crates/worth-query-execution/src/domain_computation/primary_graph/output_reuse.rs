mod verification_denial;
pub(in crate::domain_computation::primary_graph) use verification_denial::verification_denial;
mod dependency;
mod selection;
mod verification_extent;
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
