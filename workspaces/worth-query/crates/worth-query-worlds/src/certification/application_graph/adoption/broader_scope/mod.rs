//! Explicit branch-set adoption composes the existing branch-local authority.

pub mod coverage;
pub mod progression;
pub mod resume;

pub use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
pub use worth_query_host::facade::product::WorthQueryProductBranch;

pub use crate::facade::certification::document_retention_model::host::DocumentRetentionRuntime;
pub use crate::facade::certification::document_retention_model::programs::{
    RetentionProgramP0, RetentionProgramP1,
};

pub const P0_ONLY_RETENTION: u64 = 3;
pub const P1_ONLY_RETENTION: u64 = 15;

pub fn fork(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    source: WorthQueryProductBranch,
) -> WorthQueryProductBranch {
    host.runtime()
        .with_host_advancement(|host_phase| {
            host.runtime()
                .branches()
                .fork(source)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&host_phase)
                .expect("the covered branch must publish")
        })
        .expect("installed host call opens")
}

pub fn target_revision(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
) -> worth_query_host::facade::declaration::application_program::ApplicationProgramRevision {
    *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
}
