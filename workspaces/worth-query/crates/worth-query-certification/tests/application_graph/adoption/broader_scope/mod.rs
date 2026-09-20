//! Explicit branch-set adoption composes the existing branch-local authority.

mod coverage;
mod progression;
mod recovery;
mod resume;

use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::product::WorthQueryProductBranch;

use crate::bounded_dimension_model::host::BoundedDimensionRuntime;
use crate::bounded_dimension_model::programs::{DimensionProgramP0, DimensionProgramP1};

const P0_ONLY_DIMENSION: u64 = 3;
const P1_ONLY_DIMENSION: u64 = 15;

fn fork(
    host: &BoundedDimensionRuntime<DimensionProgramP0>,
    source: WorthQueryProductBranch,
) -> WorthQueryProductBranch {
    host.runtime()
        .branches()
        .fork(source)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the covered branch must publish")
}

fn target_revision(
    host: &BoundedDimensionRuntime<DimensionProgramP0>,
) -> worth_query_host::facade::declaration::application_program::ApplicationProgramRevision {
    host.supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone()
}
