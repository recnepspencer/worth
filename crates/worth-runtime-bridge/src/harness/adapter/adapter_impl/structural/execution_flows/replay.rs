use super::{
    execute_branch_compare, execute_exact_remap, BridgeHarnessError, BridgeHarnessFixture,
    StructuralHarnessExecution, StructuralIdentityDeclarationIdentity,
};

pub(super) fn execute_remap_replay(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &StructuralIdentityDeclarationIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StructuralHarnessExecution, BridgeHarnessError> {
    let execution = execute_exact_remap(
        runtime_bridge,
        fixture,
        declaration_identity,
        resource_request,
    )?;
    let StructuralHarnessExecution::Remap {
        contract,
        planned,
        reduced,
        artifact,
        record,
    } = execution
    else {
        unreachable!("exact remap execution must produce a remap record");
    };
    let replayed = runtime_bridge
        .replay_canonical_structural_remap_record(&record, resource_request)
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge structural remap replay failed: {error}"))
        })?;
    Ok(StructuralHarnessExecution::RemapReplay {
        contract,
        planned,
        reduced,
        artifact,
        record,
        replayed,
    })
}

pub(super) fn execute_branch_replay(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &StructuralIdentityDeclarationIdentity,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StructuralHarnessExecution, BridgeHarnessError> {
    let execution = execute_branch_compare(
        runtime_bridge,
        fixture,
        declaration_identity,
        resource_request,
    )?;
    let StructuralHarnessExecution::Branch {
        contract,
        planned,
        reduced,
        artifact,
        record,
    } = execution
    else {
        unreachable!("branch execution must produce a branch record");
    };
    let replayed = runtime_bridge
        .replay_canonical_structural_branch_comparison_record(&record, resource_request)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural branch comparison replay failed: {error}"
            ))
        })?;
    Ok(StructuralHarnessExecution::BranchReplay {
        contract,
        planned,
        reduced,
        artifact,
        record,
        replayed,
    })
}
