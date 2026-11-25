//! External-member checkpoint capture and expansion for contract fixtures.
use super::*;

pub(crate) fn capture_checkpoint_with_source(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    (crate::durability::data::RelationalNativeCheckpoint, Arc<dyn crate::durability::medium::RelationalDurableColdSource>),
    crate::durability::data::RelationalNativeCheckpointCaptureDenial,
> {
    let prior = runtime.durability.checkpoint_selection().snapshot();
    let mut current_members = Vec::new();
    let prior_source = if let Some(directory) = &prior {
        if let Ok(semantic) = directory.manifest.semantic_members(execution) {
            current_members.extend(semantic.iter().copied());
        }
        for root in directory.manifest.roots.iter() {
            for content in &root.content {
                current_members.push(content.member());
            }
        }
        Some(directory.source.clone())
    } else {
        None
    };
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            policy,
            crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &current_members,
                source: prior_source.clone(),
            },
        )?;
    let source = captured_members_source_with_fallback(&checkpoint, prior_source);
    Ok((checkpoint, source))
}


impl OwnedTestDurableMedium {
    /// Build a recovery plan with the medium's filed external-member checkpoint
    /// expanded into in-memory `DurableCheckpoint` fields so contract tests can
    /// mutate specific fields and verify `runtime_rebuild` validation.
    pub(crate) fn expanded_recovery_plan(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &RelationalRuntime,
        mode: crate::durability::data::RecoveryVerificationMode,
    ) -> Result<
        crate::durability::data::RecoveryPlan,
        crate::durability::data::RelationalRecoveryStop,
    > {
        let mut plan = self.recovery_plan(execution, target, mode)?;
        let filed = self.lock().checkpoint.clone();
        if let Some(filed) = filed {
            let expanded =
                crate::durability::log::native_checkpoint_codec::decode_expanded_checkpoint(
                    execution,
                    filed.checkpoint().bytes(),
                    self.clone(),
                )?
                .checkpoint;
            plan.checkpoint = Some(expanded);
        }
        Ok(plan)
    }
}
