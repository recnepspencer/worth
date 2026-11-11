use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_signal::facade::branch::AdmittedSignalBranchBasis;

use super::{admit_baseline, AdmittedRuntimeWorldCorrespondenceBasis};
use crate::facade::RuntimeBridge;

/// Portable identity of the World correspondence baseline.
/// Conditional declarations remain authored in Signal's Bridge transition section;
/// this description neither snapshots nor replaces their fresh branch projections.
/// The registered source and concrete Home composition retain authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecordedRuntimeWorldBaseline {
    meaning: Arc<RecordedBaselineMeaning>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedBaselineMeaning {
    version: u16,
    baseline: String,
    adapter_semantic_identity: String,
    graph_role: String,
    #[serde(skip)]
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl PartialEq for RecordedBaselineMeaning {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.baseline == other.baseline
            && self.adapter_semantic_identity == other.adapter_semantic_identity
            && self.graph_role == other.graph_role
    }
}
impl Eq for RecordedBaselineMeaning {}
mod json;
const BASELINE: &str = "bridge-runtime-world-baseline-v2";
impl RecordedRuntimeWorldBaseline {
    /// Fund decoded descriptive data without admitting a Bridge basis.
    pub fn from_json_in(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bytes: &[u8],
    ) -> Result<Self, RuntimeWorldBaselineRefusal> {
        let recorded = json::decode(execution, bytes)?;
        recorded.require_version()?;
        Ok(recorded)
    }
    fn require_version(&self) -> Result<(), RuntimeWorldBaselineRefusal> {
        if self.meaning.version != 2 {
            return Err(RuntimeWorldBaselineRefusal::UnsupportedVersion {
                observed: self.meaning.version,
                supported: 2,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeWorldBaselineRefusal {
    Request(worth_execution::WorkCeilingDenial),
    Description,
    UnsupportedVersion { observed: u16, supported: u16 },
    RegisteredSourceRequired,
    BaselineCorrespondenceRequired,
    RecordedMeaningMismatch,
    SignalGraphBasisRequired,
    CurrentBasisRequired,
}

pub(crate) fn recorded_baseline(
    runtime: &RuntimeBridge,
) -> Result<RecordedRuntimeWorldBaseline, RuntimeWorldBaselineRefusal> {
    let (profile, role) = baseline_values(runtime)?;
    Ok(make_baseline(profile, role, None))
}
pub(crate) fn recorded_baseline_in(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RuntimeBridge,
) -> Result<RecordedRuntimeWorldBaseline, RuntimeWorldBaselineRefusal> {
    let (profile, role) = baseline_values(runtime)?;
    let bytes = json::fixed_bytes()?
        .checked_add(BASELINE.len())
        .and_then(|bytes| bytes.checked_add(profile.adapter_semantic_identity().len()))
        .and_then(|bytes| bytes.checked_add(role.len()))
        .ok_or(RuntimeWorldBaselineRefusal::Description)?;
    let funding = json::reserve(execution, bytes)?;
    Ok(make_baseline(profile, role, funding))
}
fn make_baseline(
    profile: &crate::facade::BridgeAuthoritativeSourceProfile,
    role: &str,
    funding: Option<worth_execution::ExecutionMemoryReservation>,
) -> RecordedRuntimeWorldBaseline {
    RecordedRuntimeWorldBaseline {
        meaning: Arc::new(RecordedBaselineMeaning {
            version: 2,
            baseline: BASELINE.into(),
            adapter_semantic_identity: profile.adapter_semantic_identity().to_owned(),
            graph_role: role.to_owned(),
            _funding: funding,
        }),
    }
}
fn baseline_values(
    runtime: &RuntimeBridge,
) -> Result<(&crate::facade::BridgeAuthoritativeSourceProfile, &str), RuntimeWorldBaselineRefusal> {
    use RuntimeWorldBaselineRefusal as Refusal;
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
    Ok((profile, role))
}

pub(crate) fn readmit_baseline(
    runtime: &RuntimeBridge,
    recorded: &RecordedRuntimeWorldBaseline,
    signal: &AdmittedSignalBranchBasis,
) -> Result<AdmittedRuntimeWorldCorrespondenceBasis, RuntimeWorldBaselineRefusal> {
    admit_recorded_meaning(runtime, recorded)?;
    let graph = signal
        .observation()
        .target()
        .as_basis()
        .and_then(|basis| basis.graph_instance_id().parse::<u64>().ok())
        .ok_or(RuntimeWorldBaselineRefusal::SignalGraphBasisRequired)?;
    Ok(admit_baseline(runtime, graph).bind_signal_branch(signal))
}

fn admit_recorded_meaning(
    runtime: &RuntimeBridge,
    recorded: &RecordedRuntimeWorldBaseline,
) -> Result<(), RuntimeWorldBaselineRefusal> {
    recorded.require_version()?;
    let (profile, role) = baseline_values(runtime)?;
    if recorded.meaning.baseline != BASELINE
        || recorded.meaning.adapter_semantic_identity != profile.adapter_semantic_identity()
        || recorded.meaning.graph_role != role
    {
        return Err(RuntimeWorldBaselineRefusal::RecordedMeaningMismatch);
    }
    Ok(())
}
