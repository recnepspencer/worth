use worth_relational_branch::internal_port as branch;
pub mod artifacts;
pub mod bundle_publication;
pub mod diagnostic_emission;
pub mod pending_settlement;
pub mod performed_settlement;
pub mod post_commit_consumer;
pub mod post_commit_consumption;
pub mod post_commit_diagnostics;
pub mod settled_receipt;

use crate::runtime::state::subsystems::publication::PublicationSubsystem;
use branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use branch::runtime::state::subsystems::visibility::snapshot_handles::identity::SnapshotIdentityAllocator;
use branch::runtime::state::subsystems::visibility::VisibilitySubsystem;
use worth_relational_contracts::internal_port::diagnostics::data::profiles::RelationalDiagnosticsProfile;

pub struct PublicationAuthority<'runtime> {
    publication: &'runtime PublicationSubsystem,
    visibility: &'runtime VisibilitySubsystem,
    diagnostics_profile: &'runtime RelationalDiagnosticsProfile,
    instrumentation: &'runtime RuntimeInstrumentation,
}

pub struct PublicationPreparationAuthority<'runtime> {
    runtime_instance_id: u64,
    snapshot_ids: &'runtime SnapshotIdentityAllocator,
}

impl<'runtime> PublicationPreparationAuthority<'runtime> {
    pub fn new(
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
    pub fn new(
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

pub use pending_settlement::SettlementPublication;
pub use performed_settlement::RelationalSettlementBinding;

pub(crate) use post_commit_consumer::production_post_commit_consumer;
pub use post_commit_consumer::{
    PostCommitConsumer, PostCommitConsumptionContext, PostCommitConsumptionFailure,
};
