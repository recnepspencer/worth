mod dependency;
mod selection;

pub(super) use dependency::require_installed_output_dependencies;
pub(super) use selection::{
    compare_retained_output_dependencies, compare_retained_output_witness,
    retained_output_settlement_is_verified, OutputDependencySelection,
};
