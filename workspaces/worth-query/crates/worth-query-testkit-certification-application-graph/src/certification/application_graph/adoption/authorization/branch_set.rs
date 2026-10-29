//! Fresh branch-local admission preserves completed prefix and pending custody.
pub use super::*;
pub use std::num::NonZeroUsize;
pub use worth_query_host::facade::application_entry::{
    WorthQueryBranchSetAdoptionAdvanceDenial, WorthQueryBranchSetAdoptionPreparationDenial,
    WorthQueryBranchSetAdoptionProgress,
};
pub use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;
pub fn fork(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
) -> worth_query_host::facade::product::WorthQueryProductBranch {
    host.with_host_advancement(|phase| {
        host.runtime()
            .branches()
            .fork(host.current_world())
            .components(|components| components.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
    })
    .unwrap()
}
