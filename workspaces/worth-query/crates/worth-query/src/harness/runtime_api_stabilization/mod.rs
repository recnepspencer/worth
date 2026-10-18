mod builders;
#[cfg(test)]
mod tests;
mod transcript_maintainer;
mod transcript_runtime;
mod transcript_session_proofs;
mod transcripts;

use crate::harness::certification::{digest_parts, CertificationMatrix};
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupportStatus,
};
use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RuntimeApiStabilizationPerturbationClass {
    WorkflowEditorGoldenTranscript,
    GeometryKernelGoldenTranscript,
    TableSpreadsheetGoldenTranscript,
    ComposedRuntimeAdversarialTranscript,
    StoreBackedParityDeferredGate,
    DurableRestartDeferredGate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeApiStabilizationFailureClass {
    DeferredStoreDurableGate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeApiStabilizationBundle {
    pub public_api_surface_digest: String,
    pub public_api_naming_contract_digest: String,
    pub golden_transcript_digest: String,
    pub executable_transcript_digest: String,
    pub handle_contract_digest: String,
    pub state_contract_digest: String,
    pub aspect_contract_digest: String,
    pub authority_lane_digest: String,
    pub inspection_contract_digest: String,
    pub support_matrix_digest: String,
    pub deferred_temporal_async_gate_digest: String,
    pub failure_digest: String,
    pub counter_snapshot: String,
    pub compile_fail_boundary_digest: String,
    pub transcript_family: String,
    pub public_facade_only: bool,
    pub lower_runtime_plumbing_count: usize,
    pub meaningful_assertion_count: usize,
    pub support_gated_neighbor_denial_count: usize,
    pub delivery_residue_count: usize,
    pub stable_family_count: usize,
    pub deferred_family_count: usize,
    pub unsupported_family_count: usize,
}

impl RuntimeApiStabilizationBundle {
    pub(super) fn semantic_signature(&self) -> String {
        digest_parts(&[
            format!("surface:{}", self.public_api_surface_digest),
            format!("naming:{}", self.public_api_naming_contract_digest),
            format!("transcript:{}", self.golden_transcript_digest),
            format!("executable:{}", self.executable_transcript_digest),
            format!("handle:{}", self.handle_contract_digest),
            format!("state:{}", self.state_contract_digest),
            format!("aspect:{}", self.aspect_contract_digest),
            format!("lane:{}", self.authority_lane_digest),
            format!("inspection:{}", self.inspection_contract_digest),
            format!("support:{}", self.support_matrix_digest),
            format!("gate:{}", self.deferred_temporal_async_gate_digest),
            format!("family:{}", self.transcript_family),
            format!("facade_only:{}", self.public_facade_only),
            format!("plumbing:{}", self.lower_runtime_plumbing_count),
            format!("denials:{}", self.support_gated_neighbor_denial_count),
            format!("residue:{}", self.delivery_residue_count),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeApiStabilizationRejectionBundle {
    pub failure_class: RuntimeApiStabilizationFailureClass,
    pub family: WorthQueryRuntimeFacadeFamily,
    pub status: WorthQueryRuntimeFamilySupportStatus,
    pub failure_digest: String,
    pub deferred_temporal_async_gate_digest: String,
    pub counter_snapshot: String,
    pub compile_fail_boundary_digest: String,
}

pub type RuntimeApiStabilizationCertificationMatrix = CertificationMatrix<
    RuntimeApiStabilizationPerturbationClass,
    RuntimeApiStabilizationBundle,
    RuntimeApiStabilizationRejectionBundle,
>;

pub struct RuntimeApiStabilizationAdapter;

impl RuntimeApiStabilizationAdapter {
    pub fn runtime_api_golden_dx_and_async_safe_facade_test(
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> RuntimeApiStabilizationCertificationMatrix {
        RuntimeApiStabilizationCertificationMatrix {
            suite_name: "Runtime API Golden DX And Async-Safe Facade Test",
            rows: builders::canonical_rows(request_execution),
            rejection_rows: builders::rejection_rows(request_execution),
        }
    }

    pub(crate) fn composed_runtime_hostile_transcript_evidence(
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> crate::runtime::WorthQueryRuntimePublicApiTranscriptEvidence {
        transcripts::composed_runtime_hostile_transcript(request_execution)
    }
}

pub(super) fn transcript_aspect_touch(
    authored_touch_text: impl AsRef<str>,
) -> WorthQueryAspectTouch {
    let authored_touch_text = authored_touch_text.as_ref();
    let mut segments = authored_touch_text.split('.');
    let aspect_label = segments
        .next()
        .expect("runtime transcript authored touch should include an aspect");
    let aspect_key =
        AspectKey::new(aspect_label).expect("runtime transcript aspect key should admit");
    let fields = segments
        .map(|field_label| {
            FieldKey::new(field_label).expect("runtime transcript field key should admit")
        })
        .collect::<Vec<_>>();
    let field_path =
        CanonicalFieldPath::new(fields).expect("runtime transcript field path should admit");
    WorthQueryAspectTouch::aspect_field_path(aspect_key, field_path)
}
