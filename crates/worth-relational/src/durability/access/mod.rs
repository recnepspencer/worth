mod authority_continuity;
mod continuity_issue_mapping;
mod envelope_version_selection;
mod medium_recovery_plan;
mod medium_transition_tail;
mod native_checkpoint_recovery_plan;
mod recovery_plan_entrypoint;
mod retained_recovery_plan;
mod transition_history;

pub use recovery_plan_entrypoint::DurabilityAccess;

pub(crate) use authority_continuity::authority_continuity_for_envelopes;
pub(super) use envelope_version_selection::descriptor_semantics_version_for_envelopes;
pub(crate) use medium_recovery_plan::checkpoint_coverage;
pub(crate) use native_checkpoint_recovery_plan::native_checkpoint_recovery_plan;

/// Re-evaluate configuration-dependent evidence after the same Home receiver
/// installs its actual executable meaning. Source chronology remains unchanged.
pub(crate) fn rebind_home_receiver_plan(
    runtime: &RelationalRuntime,
    plan: &mut crate::durability::data::RecoveryPlan,
) {
    let envelopes = plan
        .checkpoint
        .as_ref()
        .map_or(&[][..], |checkpoint| checkpoint.envelopes.as_slice());
    let mut continuity = authority_continuity_for_envelopes(runtime, envelopes, &plan.tail_log);
    if let Some(checkpoint) = &plan.checkpoint {
        native_checkpoint_recovery_plan::bind_source_runtime_identity(
            runtime,
            checkpoint,
            &mut continuity,
        );
    }
    plan.config.schema.registry = runtime.config().schema.registry.clone();
    plan.authority_continuity = continuity;
}

use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub(crate) fn durability_access(&self) -> DurabilityAccess<'_> {
        DurabilityAccess::new(self)
    }
}
