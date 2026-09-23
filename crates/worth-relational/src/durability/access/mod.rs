mod authority_continuity;
mod continuity_issue_mapping;
mod envelope_version_selection;
mod in_memory_recovery_plan;
mod native_checkpoint_recovery_plan;
mod persisted_recovery_plan;
mod recovery_basis_mismatch;
mod recovery_plan_entrypoint;

pub use recovery_plan_entrypoint::DurabilityAccess;

pub(crate) use authority_continuity::authority_continuity_for_envelopes;
pub(super) use envelope_version_selection::descriptor_semantics_version_for_envelopes;
pub(crate) use native_checkpoint_recovery_plan::native_checkpoint_recovery_plan;
pub(super) use recovery_basis_mismatch::recovery_basis_mismatch;

use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub(crate) fn durability_access(&self) -> DurabilityAccess<'_> {
        DurabilityAccess::new(self)
    }
}
