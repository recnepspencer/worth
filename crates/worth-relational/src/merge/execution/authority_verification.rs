use crate::merge::data::{
    MergeExecutionCompilationError, MergeExecutionError, PreparedMergeExecution, RuntimeInstanceId,
};

use super::super::planning_artifact::merge_schema_snapshot_for_execution_ready;

pub(super) fn verify_prepared_merge_execution(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    prepared: &PreparedMergeExecution,
) -> Result<(), MergeExecutionError> {
    runtime
        .performance_access()
        .count_merge_execution_verification_request();
    let binding = &prepared.bound_executable_plan().authority_binding;
    verify_authority_binding_matches_request(binding, prepared)?;
    verify_runtime_instance(runtime, binding)?;
    verify_branch_heads(runtime, binding)?;
    verify_merge_base(runtime, binding)?;
    verify_execution_ready_proof(binding, prepared)?;
    verify_schema_snapshot(execution, runtime, binding, prepared)?;
    verify_compiled_plan_digest(runtime, binding, prepared)?;
    Ok(())
}

fn verify_authority_binding_matches_request(
    binding: &crate::merge::data::MergeExecutionAuthorityBinding,
    prepared: &PreparedMergeExecution,
) -> Result<(), MergeExecutionError> {
    if binding.request != *prepared.request() {
        return authority_binding_mismatch("binding request does not match prepared request");
    }
    Ok(())
}

fn verify_runtime_instance(
    runtime: &crate::runtime::RelationalRuntime,
    binding: &crate::merge::data::MergeExecutionAuthorityBinding,
) -> Result<(), MergeExecutionError> {
    let current_runtime_instance_id = RuntimeInstanceId(runtime.runtime_instance_id());
    if binding.runtime_instance_id != current_runtime_instance_id {
        return Err(MergeExecutionError::RuntimeInstanceMismatch {
            planned: binding.runtime_instance_id,
            current: current_runtime_instance_id,
        });
    }
    Ok(())
}

fn verify_branch_heads(
    runtime: &crate::runtime::RelationalRuntime,
    binding: &crate::merge::data::MergeExecutionAuthorityBinding,
) -> Result<(), MergeExecutionError> {
    let target_state = runtime
        .history()
        .branch_reference_state(binding.request.target_branch());
    let target_head = runtime
        .history()
        .branch_head(binding.request.target_branch());
    runtime
        .performance_access()
        .count_merge_execution_branch_head_checks(1);
    if let Some((current_reference, current_truth_version)) = &target_state {
        if (current_reference != &binding.target_reference
            || current_truth_version != &binding.target_truth_version)
            && target_head.as_ref().map(|head| head.commit_id)
                == Some(binding.target_head_commit_id)
        {
            return Err(MergeExecutionError::StaleBranchReference {
                branch: binding.request.target_branch().clone(),
                planned_generation: binding.target_reference.generation().get(),
                current_generation: Some(current_reference.generation().get()),
                planned_truth_version: binding.target_truth_version.as_u64(),
                current_truth_version: Some(current_truth_version.as_u64()),
            });
        }
    } else if target_head.is_none() {
        return Err(MergeExecutionError::StaleBranchReference {
            branch: binding.request.target_branch().clone(),
            planned_generation: binding.target_reference.generation().get(),
            current_generation: None,
            planned_truth_version: binding.target_truth_version.as_u64(),
            current_truth_version: None,
        });
    }
    if target_head.as_ref().map(|head| head.commit_id) != Some(binding.target_head_commit_id) {
        return Err(MergeExecutionError::StaleBranchHead {
            branch: binding.request.target_branch().clone(),
            planned: binding.target_head_commit_id,
            current: target_head.map(|head| head.commit_id),
        });
    }

    let source_state = runtime
        .history()
        .branch_reference_state(binding.request.source_branch());
    let source_head = runtime
        .history()
        .branch_head(binding.request.source_branch());
    runtime
        .performance_access()
        .count_merge_execution_branch_head_checks(1);
    if let Some((current_reference, current_truth_version)) = &source_state {
        if (current_reference != &binding.source_reference
            || current_truth_version != &binding.source_truth_version)
            && source_head.as_ref().map(|head| head.commit_id)
                == Some(binding.source_head_commit_id)
        {
            return Err(MergeExecutionError::StaleBranchReference {
                branch: binding.request.source_branch().clone(),
                planned_generation: binding.source_reference.generation().get(),
                current_generation: Some(current_reference.generation().get()),
                planned_truth_version: binding.source_truth_version.as_u64(),
                current_truth_version: Some(current_truth_version.as_u64()),
            });
        }
    } else if source_head.is_none() {
        return Err(MergeExecutionError::StaleBranchReference {
            branch: binding.request.source_branch().clone(),
            planned_generation: binding.source_reference.generation().get(),
            current_generation: None,
            planned_truth_version: binding.source_truth_version.as_u64(),
            current_truth_version: None,
        });
    }
    if source_head.as_ref().map(|head| head.commit_id) != Some(binding.source_head_commit_id) {
        return Err(MergeExecutionError::StaleBranchHead {
            branch: binding.request.source_branch().clone(),
            planned: binding.source_head_commit_id,
            current: source_head.map(|head| head.commit_id),
        });
    }
    Ok(())
}

fn verify_merge_base(
    runtime: &crate::runtime::RelationalRuntime,
    binding: &crate::merge::data::MergeExecutionAuthorityBinding,
) -> Result<(), MergeExecutionError> {
    let merge_base = runtime
        .branch_identity(binding.request.target_branch())
        .ok()
        .and_then(|target_identity| {
            runtime
                .transaction_validation_input_for(&target_identity)
                .ok()
        })
        .zip(
            runtime
                .branch_identity(binding.request.source_branch())
                .ok()
                .and_then(|source_identity| {
                    runtime
                        .transaction_validation_input_for(&source_identity)
                        .ok()
                }),
        )
        .and_then(|(target_options, source_options)| {
            runtime.history().latest_common_ancestor_between_bindings(
                target_options.basis(),
                source_options.basis(),
            )
        });
    runtime
        .performance_access()
        .count_merge_execution_merge_base_checks(1);
    if merge_base != Some(binding.merge_base_commit_id) {
        return Err(MergeExecutionError::MergeBaseDrift {
            planned: binding.merge_base_commit_id,
            current: merge_base,
        });
    }
    Ok(())
}

fn verify_execution_ready_proof(
    binding: &crate::merge::data::MergeExecutionAuthorityBinding,
    prepared: &PreparedMergeExecution,
) -> Result<(), MergeExecutionError> {
    let execution_ready = prepared.execution_ready_plan();
    if binding.target_head_commit_id != execution_ready.basis.target_head.commit_id {
        return authority_binding_mismatch(
            "binding target head does not match execution-ready proof",
        );
    }
    if binding.source_head_commit_id != execution_ready.basis.source_head.commit_id {
        return authority_binding_mismatch(
            "binding source head does not match execution-ready proof",
        );
    }
    if binding.merge_base_commit_id != execution_ready.basis.merge_base.commit.commit_id {
        return authority_binding_mismatch(
            "binding merge base does not match execution-ready proof",
        );
    }
    let prepared_schema_digest =
        crate::merge::data::schema_snapshot_digest(&execution_ready.schema_snapshot);
    if binding.schema_snapshot_digest != prepared_schema_digest {
        return authority_binding_mismatch(
            "binding schema digest does not match execution-ready proof",
        );
    }
    Ok(())
}

fn verify_schema_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    binding: &crate::merge::data::MergeExecutionAuthorityBinding,
    prepared: &PreparedMergeExecution,
) -> Result<(), MergeExecutionError> {
    let execution_ready = prepared.execution_ready_plan();
    let current_schema_snapshot = merge_schema_snapshot_for_execution_ready(
        execution,
        runtime,
        execution_ready.basis.target_head.version_id,
        execution_ready.source_records.as_ref(),
        execution_ready.target_touched_records.as_ref(),
    )
    .map_err(MergeExecutionError::Planning)?;
    runtime
        .performance_access()
        .count_merge_execution_schema_snapshot_kinds(current_schema_snapshot.touched_kinds.len());
    let current_digest = crate::merge::data::schema_snapshot_digest(&current_schema_snapshot);
    if current_digest != binding.schema_snapshot_digest {
        return Err(MergeExecutionError::SchemaSemanticDrift {
            planned_digest: binding.schema_snapshot_digest.clone(),
            current_digest,
        });
    }
    Ok(())
}

fn verify_compiled_plan_digest(
    runtime: &crate::runtime::RelationalRuntime,
    binding: &crate::merge::data::MergeExecutionAuthorityBinding,
    prepared: &PreparedMergeExecution,
) -> Result<(), MergeExecutionError> {
    let compiled = prepared.bound_executable_plan();
    let current_compiled_digest = crate::merge::data::compiled_executable_plan_digest(
        &binding.request,
        compiled.parent_order.as_ref(),
        compiled.record_plans.as_ref(),
    );
    runtime
        .performance_access()
        .count_merge_execution_compiled_plan_digest_checks(1);
    if current_compiled_digest != binding.executable_plan_digest {
        return authority_binding_mismatch(
            "compiled executable plan digest does not match binding certification",
        );
    }
    Ok(())
}

fn authority_binding_mismatch<T>(detail: &'static str) -> Result<T, MergeExecutionError> {
    Err(MergeExecutionError::Compilation(
        MergeExecutionCompilationError::PreparedAuthorityBindingMismatch { detail },
    ))
}
