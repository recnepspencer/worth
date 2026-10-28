//! Explicit branch-set adoption composes the existing branch-local authority.

mod coverage;
mod progression;
mod resume;

use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::product::WorthQueryProductBranch;

use crate::document_retention_model::host::DocumentRetentionRuntime;
use crate::document_retention_model::programs::{RetentionProgramP0, RetentionProgramP1};

const P0_ONLY_RETENTION: u64 = 3;
const P1_ONLY_RETENTION: u64 = 15;

fn fork(
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

fn target_revision(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
) -> worth_query_host::facade::declaration::application_program::ApplicationProgramRevision {
    *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
}
