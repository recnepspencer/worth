use serde::{Deserialize, Serialize};
use worth_signal::facade::branch::AdmittedSignalInitialEmptyRoot;

use super::{admit_baseline, AdmittedRuntimeWorldCorrespondenceBasis};
use crate::facade::RuntimeBridge;

/// Portable meaning of the existing uninstalled Bridge baseline.
/// The registered source and concrete Home composition retain authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedRuntimeWorldBaseline {
    version: u16,
    baseline: String,
    adapter_semantic_identity: String,
    graph_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeWorldBaselineRefusal {
    UnsupportedVersion { observed: u16, supported: u16 },
    RegisteredSourceRequired,
    UninstalledBaselineRequired,
    RecordedMeaningMismatch,
    SignalInitialRootRequired,
    CurrentBasisRequired,
}

pub(crate) fn recorded_baseline(
    runtime: &RuntimeBridge,
) -> Result<RecordedRuntimeWorldBaseline, RuntimeWorldBaselineRefusal> {
    use RuntimeWorldBaselineRefusal as Refusal;
    if runtime.semantic_dependency_registry.authoritative_count() != 0 {
        return Err(Refusal::UninstalledBaselineRequired);
    }
    let profile = runtime
        .authoritative_source_profile
        .as_ref()
        .ok_or(Refusal::RegisteredSourceRequired)?;
    let current_profile = runtime
        .committed_patch_source
        .authoritative_source_profile()
        .ok_or(Refusal::RegisteredSourceRequired)?;
    let role = runtime
        .committed_patch_source
        .runtime_world_baseline_graph_role()
        .ok_or(Refusal::RegisteredSourceRequired)?;
    if profile != &current_profile || role.trim().is_empty() || role.trim() != role {
        return Err(Refusal::RegisteredSourceRequired);
    }
    Ok(RecordedRuntimeWorldBaseline {
        version: 1,
        baseline: "bridge-runtime-world-uninstalled-baseline-v1".into(),
        adapter_semantic_identity: profile.adapter_semantic_identity().to_owned(),
        graph_role: role.to_owned(),
    })
}

pub(crate) fn readmit_baseline(
    runtime: &RuntimeBridge,
    recorded: &RecordedRuntimeWorldBaseline,
    signal: &AdmittedSignalInitialEmptyRoot,
) -> Result<AdmittedRuntimeWorldCorrespondenceBasis, RuntimeWorldBaselineRefusal> {
    if recorded.version != 1 {
        return Err(RuntimeWorldBaselineRefusal::UnsupportedVersion {
            observed: recorded.version,
            supported: 1,
        });
    }
    if recorded != &recorded_baseline(runtime)? {
        return Err(RuntimeWorldBaselineRefusal::RecordedMeaningMismatch);
    }
    let graph = signal
        .basis()
        .observation()
        .target()
        .as_basis()
        .and_then(|basis| basis.graph_instance_id().parse::<u64>().ok())
        .ok_or(RuntimeWorldBaselineRefusal::SignalInitialRootRequired)?;
    Ok(admit_baseline(runtime, graph))
}
