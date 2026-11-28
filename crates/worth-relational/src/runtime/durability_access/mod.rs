use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointOwnerPort as _;
use worth_relational_durability::internal_port::durability::data::recovery_plan::RecoveryPlanOwnerPort as _;
mod authority_continuity;
mod branch_transition_artifact_export;
mod canonical_commit_artifact_export;
mod canonical_source;
mod medium_recovery_plan;
mod native_checkpoint_recovery_plan;
mod recovery_plan_entrypoint;
mod retained_recovery_plan;
mod transition_history;

pub use recovery_plan_entrypoint::DurabilityAccess;

use crate::durability::envelope_version_selection::descriptor_semantics_version_for_envelopes;
pub(crate) use authority_continuity::authority_continuity_for_envelopes;
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
        .map_or(&[][..], |checkpoint| checkpoint.envelopes());
    let mut continuity = authority_continuity_for_envelopes(runtime, envelopes, plan.tail_log());
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
    pub fn durability(&self) -> DurabilityAccess<'_> {
        DurabilityAccess::new(self)
    }
}
