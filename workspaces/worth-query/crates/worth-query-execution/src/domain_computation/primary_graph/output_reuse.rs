mod dependency;
mod selection;

pub(super) use dependency::require_installed_output_dependencies;
pub(super) use selection::{compare_retained_output_dependencies, OutputDependencySelection};
