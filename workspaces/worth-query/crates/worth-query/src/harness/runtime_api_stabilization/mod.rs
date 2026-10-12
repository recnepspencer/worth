mod builders;
mod closeout;
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

use closeout::RuntimeApiStabilizationCloseout;

pub const RUNTIME_API_STABILIZATION_REQUIRED_CANONICAL_ROW_NAMES: &[&str] = &[
    "workflow-editor-golden-transcript",
    "geometry-kernel-golden-transcript",
    "table-spreadsheet-golden-transcript",
    "composed-runtime-adversarial-transcript",
];

pub const RUNTIME_API_STABILIZATION_REQUIRED_REJECTION_ROW_NAMES: &[&str] = &[
    "store-backed-parity-deferred-gate",
    "durable-restart-deferred-gate",
];

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
    pub(super) fn has_required_outputs(&self) -> bool {
        !self.public_api_surface_digest.is_empty()
            && !self.public_api_naming_contract_digest.is_empty()
            && !self.golden_transcript_digest.is_empty()
            && !self.executable_transcript_digest.is_empty()
            && !self.handle_contract_digest.is_empty()
            && !self.state_contract_digest.is_empty()
            && !self.aspect_contract_digest.is_empty()
            && !self.authority_lane_digest.is_empty()
            && !self.inspection_contract_digest.is_empty()
            && !self.support_matrix_digest.is_empty()
            && !self.deferred_temporal_async_gate_digest.is_empty()
            && !self.failure_digest.is_empty()
            && !self.counter_snapshot.is_empty()
            && !self.compile_fail_boundary_digest.is_empty()
            && !self.transcript_family.is_empty()
            && self.support_gated_neighbor_denial_count >= 1
    }

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeApiStabilizationArtifact {
    pub suite_name: &'static str,
    pub certification_bundle_digest: String,
    pub coverage_matrix_digest: String,
    pub closeout: RuntimeApiStabilizationCloseout,
    pub matrix: RuntimeApiStabilizationCertificationMatrix,
}

impl RuntimeApiStabilizationCertificationMatrix {
    pub fn into_runtime_api_stabilization_artifact(self) -> RuntimeApiStabilizationArtifact {
        let closeout = RuntimeApiStabilizationCloseout::from_matrix(&self);
        RuntimeApiStabilizationArtifact {
            suite_name: self.suite_name,
            certification_bundle_digest: digest_parts(&builders::bundle_digest_parts(&self)),
            coverage_matrix_digest: digest_parts(&builders::coverage_digest_parts(&self)),
            closeout,
            matrix: self,
        }
    }
}

pub struct RuntimeApiStabilizationAdapter;

impl RuntimeApiStabilizationAdapter {
    pub fn runtime_api_golden_dx_and_async_safe_facade_artifact(
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> RuntimeApiStabilizationArtifact {
        Self::runtime_api_golden_dx_and_async_safe_facade_test(request_execution)
            .into_runtime_api_stabilization_artifact()
    }

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
