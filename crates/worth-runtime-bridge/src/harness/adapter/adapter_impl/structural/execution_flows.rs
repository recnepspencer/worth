mod replay;
use super::*;
use crate::structural::{
    StructuralIdentityDeclarationIdentity, StructuralMatchCandidate, StructuralMatchCandidateKind,
};
use replay::{execute_branch_replay, execute_remap_replay};

pub(super) fn execute_structural_request(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    target: StructuralHarnessTarget,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StructuralHarnessExecution, BridgeHarnessError> {
    match target {
        StructuralHarnessTarget::RemapExact {
            declaration_identity,
        } => execute_exact_remap(
            runtime_bridge,
            fixture,
            &declaration_identity,
            resource_request,
        ),
        StructuralHarnessTarget::RemapAmbiguous {
            declaration_identity,
        } => execute_ambiguous_remap(
            runtime_bridge,
            fixture,
            &declaration_identity,
            resource_request,
        ),
        StructuralHarnessTarget::RemapNoSafeMatch {
            declaration_identity,
        } => execute_no_safe_match_remap(
            runtime_bridge,
            fixture,
            &declaration_identity,
            resource_request,
        ),
        StructuralHarnessTarget::RemapLineageDivergence {
            declaration_identity,
        } => execute_lineage_divergence_remap(
            runtime_bridge,
            fixture,
            &declaration_identity,
            resource_request,
        ),
        StructuralHarnessTarget::RemapIdentityConflict {
            declaration_identity,
        } => execute_identity_conflict_remap(
            runtime_bridge,
            fixture,
            &declaration_identity,
            resource_request,
        ),
        StructuralHarnessTarget::RemapReplay {
            declaration_identity,
        } => execute_remap_replay(
            runtime_bridge,
            fixture,
            &declaration_identity,
            resource_request,
        ),
        StructuralHarnessTarget::BranchCompare {
            declaration_identity,
        } => execute_branch_compare(
            runtime_bridge,
            fixture,
            &declaration_identity,
            resource_request,
        ),
        StructuralHarnessTarget::BranchReplay {
            declaration_identity,
        } => execute_branch_replay(
            runtime_bridge,
            fixture,
            &declaration_identity,
            resource_request,
        ),
    }
}

fn execute_exact_remap(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &StructuralIdentityDeclarationIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StructuralHarnessExecution, BridgeHarnessError> {
    let contract = admitted_contract(runtime_bridge, fixture, declaration_identity)?;
    let planned = runtime_bridge
        .plan_structural_match_packet_set_from_read_packets(
            &contract,
            remap_target_packet(),
            vec![remap_target_packet()],
            execution,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge structural remap planning failed: {error}"))
        })?;
    let reduced = runtime_bridge
        .reduce_structural_match_set(&planned)
        .map_err(|error| {
            BridgeHarnessError::new(format!("bridge structural remap reduction failed: {error}"))
        })?;
    let artifact = runtime_bridge
        .publish_structural_remap_artifact(&reduced)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural remap publication failed: {error}"
            ))
        })?;
    let record = runtime_bridge
        .canonicalize_structural_remap_record(&contract, &planned, &reduced, &artifact);
    Ok(StructuralHarnessExecution::Remap {
        contract,
        planned,
        reduced,
        artifact,
        record,
    })
}

fn execute_ambiguous_remap(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &StructuralIdentityDeclarationIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StructuralHarnessExecution, BridgeHarnessError> {
    let contract = admitted_contract(runtime_bridge, fixture, declaration_identity)?;
    let fingerprint = runtime_bridge
        .materialize_structural_fingerprint(&contract, remap_target_packet(), execution)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural ambiguity fingerprint materialization failed: {error}"
            ))
        })?;
    let planned = runtime_bridge
        .plan_structural_match_packet_set(
            &contract,
            vec![
                StructuralMatchCandidate::with_fingerprint(
                    crate::facade::StructuralCandidateIdentity::admit_bridge_owned(
                        "structural-candidate:ambiguous-a",
                    ),
                    StructuralMatchCandidateKind::ExactAdvisoryMatch,
                    Some(fingerprint.clone()),
                ),
                StructuralMatchCandidate::with_fingerprint(
                    crate::facade::StructuralCandidateIdentity::admit_bridge_owned(
                        "structural-candidate:ambiguous-b",
                    ),
                    StructuralMatchCandidateKind::AdvisoryReuseCandidate,
                    Some(fingerprint),
                ),
            ],
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural ambiguity planning failed: {error}"
            ))
        })?;
    let reduced = runtime_bridge
        .reduce_structural_match_set(&planned)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural ambiguity reduction failed: {error}"
            ))
        })?;
    Ok(StructuralHarnessExecution::Rejected {
        contract,
        planned,
        reduced,
    })
}

fn execute_no_safe_match_remap(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &StructuralIdentityDeclarationIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StructuralHarnessExecution, BridgeHarnessError> {
    let contract = admitted_contract(runtime_bridge, fixture, declaration_identity)?;
    let planned = runtime_bridge
        .plan_structural_match_packet_set_from_read_packets(
            &contract,
            remap_target_packet(),
            vec![no_safe_match_packet()],
            execution,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural no-safe-match planning failed: {error}"
            ))
        })?;
    let reduced = runtime_bridge
        .reduce_structural_match_set(&planned)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural no-safe-match reduction failed: {error}"
            ))
        })?;
    Ok(StructuralHarnessExecution::Rejected {
        contract,
        planned,
        reduced,
    })
}

fn execute_lineage_divergence_remap(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &StructuralIdentityDeclarationIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StructuralHarnessExecution, BridgeHarnessError> {
    let contract = admitted_contract(runtime_bridge, fixture, declaration_identity)?;
    let fingerprint = runtime_bridge
        .materialize_structural_fingerprint(&contract, remap_target_packet(), execution)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural lineage-divergence fingerprint materialization failed: {error}"
            ))
        })?;
    let planned = runtime_bridge
        .plan_structural_match_packet_set(
            &contract,
            vec![StructuralMatchCandidate::with_fingerprint(
                crate::facade::StructuralCandidateIdentity::admit_bridge_owned(
                    "structural-candidate:lineage-divergence",
                ),
                StructuralMatchCandidateKind::LineageStructuralDivergence,
                Some(fingerprint),
            )],
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural lineage-divergence planning failed: {error}"
            ))
        })?;
    let reduced = runtime_bridge
        .reduce_structural_match_set(&planned)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural lineage-divergence reduction failed: {error}"
            ))
        })?;
    Ok(StructuralHarnessExecution::Rejected {
        contract,
        planned,
        reduced,
    })
}

fn execute_identity_conflict_remap(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &StructuralIdentityDeclarationIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StructuralHarnessExecution, BridgeHarnessError> {
    let contract = admitted_contract(runtime_bridge, fixture, declaration_identity)?;
    let planned = runtime_bridge
        .plan_structural_match_packet_set_from_read_packets(
            &contract,
            remap_target_packet(),
            vec![identity_conflict_packet()],
            execution,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural identity-separation planning failed: {error}"
            ))
        })?;
    let reduced = runtime_bridge
        .reduce_structural_match_set(&planned)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural identity-separation reduction failed: {error}"
            ))
        })?;
    Ok(StructuralHarnessExecution::Rejected {
        contract,
        planned,
        reduced,
    })
}

fn execute_branch_compare(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    declaration_identity: &StructuralIdentityDeclarationIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<StructuralHarnessExecution, BridgeHarnessError> {
    let contract = admitted_contract(runtime_bridge, fixture, declaration_identity)?;
    let planned = runtime_bridge
        .plan_structural_branch_comparison_from_read_packet(&contract, branch_packet(), execution)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural branch comparison planning failed: {error}"
            ))
        })?;
    let reduced = runtime_bridge
        .reduce_structural_match_set(&planned)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural branch comparison reduction failed: {error}"
            ))
        })?;
    let artifact = runtime_bridge
        .publish_branch_comparison_artifact(&reduced)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "bridge structural branch comparison publication failed: {error}"
            ))
        })?;
    let record = runtime_bridge
        .canonicalize_structural_branch_comparison_record(&contract, &planned, &reduced, &artifact);
    Ok(StructuralHarnessExecution::Branch {
        contract,
        planned,
        reduced,
        artifact,
        record,
    })
}

fn remap_target_packet() -> crate::facade::SnapshotReadPacket {
    crate::facade::SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
        "entity-1",
        crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile")
                .expect("valid snapshot aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
    )])
}

fn identity_conflict_packet() -> crate::facade::SnapshotReadPacket {
    crate::facade::SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
        "entity-2",
        crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile")
                .expect("valid snapshot aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
    )])
}

fn no_safe_match_packet() -> crate::facade::SnapshotReadPacket {
    crate::facade::SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
        "entity-3",
        crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile")
                .expect("valid snapshot aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
    )])
}

fn branch_packet() -> crate::facade::SnapshotReadPacket {
    crate::facade::SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
        "entity-1",
        crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile")
                .expect("valid snapshot aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
    )])
}
