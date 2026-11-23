pub(crate) mod artifacts;
mod bundle_publication;
pub(crate) mod diagnostic_emission;
mod pending_settlement;
mod performed_settlement;
mod post_commit_consumer;
mod post_commit_consumption;
mod post_commit_diagnostics;
mod settlement_port;

use crate::diagnostics::data::RelationalDiagnosticsProfile;
use crate::runtime::state::subsystems::publication::PublicationSubsystem;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::runtime::state::subsystems::visibility::{
    SnapshotIdentityAllocator, VisibilitySubsystem,
};

pub struct PublicationAuthority<'runtime> {
    publication: &'runtime PublicationSubsystem,
    visibility: &'runtime VisibilitySubsystem,
    diagnostics_profile: &'runtime RelationalDiagnosticsProfile,
    instrumentation: &'runtime RuntimeInstrumentation,
}

pub(crate) struct PublicationPreparationAuthority<'runtime> {
    runtime_instance_id: u64,
    snapshot_ids: &'runtime SnapshotIdentityAllocator,
}

impl<'runtime> PublicationPreparationAuthority<'runtime> {
    pub(crate) fn new(
        runtime_instance_id: u64,
        snapshot_ids: &'runtime SnapshotIdentityAllocator,
    ) -> Self {
        Self {
            runtime_instance_id,
            snapshot_ids,
        }
    }
}

impl<'runtime> PublicationAuthority<'runtime> {
    pub(crate) fn new(
        publication: &'runtime PublicationSubsystem,
        visibility: &'runtime VisibilitySubsystem,
        diagnostics_profile: &'runtime RelationalDiagnosticsProfile,
        instrumentation: &'runtime RuntimeInstrumentation,
    ) -> Self {
        Self {
            publication,
            visibility,
            diagnostics_profile,
            instrumentation,
        }
    }
}

pub use settlement_port::RelationalSettlementPort;

pub(crate) use post_commit_consumer::production_post_commit_consumer;
pub use post_commit_consumer::{
    PostCommitConsumer, PostCommitConsumptionContext, PostCommitConsumptionFailure,
};
