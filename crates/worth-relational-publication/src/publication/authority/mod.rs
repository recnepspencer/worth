use worth_relational_branch::internal_port as branch;
pub mod bundle_publication;
pub mod diagnostic_emission;
pub mod pending_settlement;
pub mod performed_settlement;
pub mod settled_receipt;

use branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use branch::runtime::state::subsystems::visibility::snapshot_handles::identity::SnapshotIdentityAllocator;
use branch::runtime::state::subsystems::visibility::VisibilitySubsystem;

pub struct PublicationAuthority<'runtime> {
    visibility: &'runtime VisibilitySubsystem,
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
        visibility: &'runtime VisibilitySubsystem,
        instrumentation: &'runtime RuntimeInstrumentation,
    ) -> Self {
        Self {
            visibility,
            instrumentation,
        }
    }
}

pub use pending_settlement::SettlementPublication;
pub use performed_settlement::RelationalSettlementBinding;
