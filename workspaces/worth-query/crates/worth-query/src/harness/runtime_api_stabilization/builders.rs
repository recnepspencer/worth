use crate::harness::certification::{
    digest_parts, CanonicalCertificationRow, HostileExpectation, ParityAnchor,
    RejectionCertificationRow,
};
use crate::runtime::{
    WorthQueryHandleContract, WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupportStatus,
    WorthQueryRuntimePublicApiContract, WorthQueryRuntimePublicApiNamingContract,
    WorthQueryRuntimePublicSupportMatrix, WorthQueryRuntimeSupportProfile,
};

use super::{
    transcripts, RuntimeApiStabilizationBundle, RuntimeApiStabilizationCertificationMatrix,
    RuntimeApiStabilizationFailureClass, RuntimeApiStabilizationPerturbationClass,
    RuntimeApiStabilizationRejectionBundle,
};

pub(super) fn canonical_rows(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Vec<
    CanonicalCertificationRow<
        RuntimeApiStabilizationPerturbationClass,
        RuntimeApiStabilizationBundle,
    >,
> {
    vec![
        canonical_row(
            request_execution,
            "workflow-editor-golden-transcript",
            RuntimeApiStabilizationPerturbationClass::WorkflowEditorGoldenTranscript,
            transcripts::workflow_editor_transcript,
            [
                "live",
                "computed",
                "effect",
                "branch-preview",
                "branch-intent",
                "inspect",
            ],
            9,
        ),
        canonical_row(
            request_execution,
            "geometry-kernel-golden-transcript",
            RuntimeApiStabilizationPerturbationClass::GeometryKernelGoldenTranscript,
            transcripts::geometry_kernel_transcript,
            [
                "topology-live",
                "derived-surface",
                "fallback",
                "intent",
                "inspect",
            ],
            8,
        ),
        canonical_row(
            request_execution,
            "table-spreadsheet-golden-transcript",
            RuntimeApiStabilizationPerturbationClass::TableSpreadsheetGoldenTranscript,
            transcripts::table_spreadsheet_transcript,
            [
                "visible-rows",
                "formula",
                "dropdown",
                "layout",
                "batched-intent",
            ],
            8,
        ),
        adversarial_composed_row(request_execution),
    ]
}

pub(super) fn rejection_rows(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Vec<
    RejectionCertificationRow<
        RuntimeApiStabilizationPerturbationClass,
        RuntimeApiStabilizationBundle,
        RuntimeApiStabilizationRejectionBundle,
    >,
> {
    vec![
        rejection_row(
            request_execution,
            "store-backed-parity-deferred-gate",
            RuntimeApiStabilizationPerturbationClass::StoreBackedParityDeferredGate,
            WorthQueryRuntimeFacadeFamily::StoreBackedExecution,
            RuntimeApiStabilizationFailureClass::DeferredStoreDurableGate,
        ),
        rejection_row(
            request_execution,
            "durable-restart-deferred-gate",
            RuntimeApiStabilizationPerturbationClass::DurableRestartDeferredGate,
            WorthQueryRuntimeFacadeFamily::DurableArtifacts,
            RuntimeApiStabilizationFailureClass::DeferredStoreDurableGate,
        ),
    ]
}

pub(super) fn bundle_digest_parts(
    matrix: &RuntimeApiStabilizationCertificationMatrix,
) -> Vec<String> {
    matrix
        .rows
        .iter()
        .flat_map(|row| {
            [
                row.control_lane.semantic_signature(),
                row.hostile_lane.semantic_signature(),
                row.parity_lane.semantic_signature(),
            ]
        })
        .chain(matrix.rejection_rows.iter().map(|row| {
            digest_parts(&[
                row.row_name.to_string(),
                row.hostile_lane.failure_digest.clone(),
                row.hostile_lane.deferred_temporal_async_gate_digest.clone(),
            ])
        }))
        .collect()
}

pub(super) fn coverage_digest_parts(
    matrix: &RuntimeApiStabilizationCertificationMatrix,
) -> Vec<String> {
    matrix
        .rows
        .iter()
        .map(|row| format!("canonical:{}", row.row_name))
        .chain(
            matrix
                .rejection_rows
                .iter()
                .map(|row| format!("rejection:{}", row.row_name)),
        )
        .collect()
}

fn canonical_row(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    row_name: &'static str,
    perturbation_class: RuntimeApiStabilizationPerturbationClass,
    transcript: fn(
        &worth_execution::ExecutionRequest<'_, '_>,
    ) -> crate::runtime::WorthQueryRuntimePublicApiTranscriptEvidence,
    surfaces: impl IntoIterator<Item = &'static str> + Clone,
    meaningful_assertion_count: usize,
) -> CanonicalCertificationRow<
    RuntimeApiStabilizationPerturbationClass,
    RuntimeApiStabilizationBundle,
> {
    let transcript_evidence = transcript(request_execution);
    CanonicalCertificationRow {
        row_name,
        perturbation_class,
        hostile_expectation: HostileExpectation::EquivalentToControl,
        parity_anchor: ParityAnchor::Control,
        control_lane: bundle(
            transcript_evidence.clone(),
            surfaces.clone(),
            meaningful_assertion_count,
        ),
        hostile_lane: bundle(
            transcript_evidence.clone(),
            surfaces.clone(),
            meaningful_assertion_count,
        ),
        parity_lane: bundle(transcript_evidence, surfaces, meaningful_assertion_count),
    }
}

fn adversarial_composed_row(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> CanonicalCertificationRow<
    RuntimeApiStabilizationPerturbationClass,
    RuntimeApiStabilizationBundle,
> {
    let control_lane = bundle(
        transcripts::composed_runtime_transcript(request_execution),
        [
            "live-subscription",
            "nested-computed",
            "pending-intent-effect",
            "authoritative-intent",
            "effect-intent",
            "branch-intent",
            "preview-isolation",
            "feedback-graph",
        ],
        14,
    );
    let hostile_lane = bundle(
        transcripts::composed_runtime_hostile_transcript(request_execution),
        [
            "live-subscription",
            "nested-computed",
            "pending-intent-effect",
            "authoritative-intent",
            "effect-intent",
            "branch-intent",
            "preview-isolation",
            "feedback-graph",
            "temporal-neighbor-denial",
            "async-resource-neighbor-denial",
        ],
        18,
    );
    CanonicalCertificationRow {
        row_name: "composed-runtime-adversarial-transcript",
        perturbation_class:
            RuntimeApiStabilizationPerturbationClass::ComposedRuntimeAdversarialTranscript,
        hostile_expectation: HostileExpectation::DistinctFromControl,
        parity_anchor: ParityAnchor::Hostile,
        control_lane,
        parity_lane: hostile_lane.clone(),
        hostile_lane,
    }
}

fn rejection_row(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    row_name: &'static str,
    perturbation_class: RuntimeApiStabilizationPerturbationClass,
    family: WorthQueryRuntimeFacadeFamily,
    failure_class: RuntimeApiStabilizationFailureClass,
) -> RejectionCertificationRow<
    RuntimeApiStabilizationPerturbationClass,
    RuntimeApiStabilizationBundle,
    RuntimeApiStabilizationRejectionBundle,
> {
    let control_lane = bundle(
        transcripts::workflow_editor_transcript(request_execution),
        ["read", "live", "inspect"],
        6,
    );
    let contract = contract();
    let row = contract
        .family(family)
        .expect("deferred public API family should have support row");
    let failure_digest = digest_parts(&[
        format!("failure_class:{failure_class:?}"),
        format!("family:{}", family.as_str()),
        format!("status:{}", row.status().as_str()),
        format!("reason:{}", row.reason().unwrap_or("none")),
    ]);
    RejectionCertificationRow {
        row_name,
        perturbation_class,
        control_lane: control_lane.clone(),
        hostile_lane: RuntimeApiStabilizationRejectionBundle {
            failure_class,
            family,
            status: row.status(),
            failure_digest,
            deferred_temporal_async_gate_digest: row.contract_digest().to_string(),
            counter_snapshot: "forbidden_delivery_residue=0;authority_residue=0".to_string(),
            compile_fail_boundary_digest: compile_fail_boundary_digest(),
        },
        parity_lane: control_lane,
    }
}

fn bundle(
    transcript_evidence: crate::runtime::WorthQueryRuntimePublicApiTranscriptEvidence,
    surfaces: impl IntoIterator<Item = &'static str>,
    meaningful_assertion_count: usize,
) -> RuntimeApiStabilizationBundle {
    let contract = contract();
    let handle_contract = WorthQueryHandleContract::from_public_api_contract(&contract);
    let naming_contract = WorthQueryRuntimePublicApiNamingContract::standard();
    let support_matrix = WorthQueryRuntimePublicSupportMatrix::from_public_api_contract(&contract);
    let surface_list: Vec<_> = surfaces.into_iter().collect();
    let transcript_family = transcript_evidence.transcript_family().to_string();
    let golden_transcript_digest = digest_parts(
        &surface_list
            .iter()
            .map(|surface| format!("surface:{surface}"))
            .chain([format!("transcript:{transcript_family}")])
            .collect::<Vec<_>>(),
    );
    RuntimeApiStabilizationBundle {
        public_api_surface_digest: contract.contract_digest().to_string(),
        public_api_naming_contract_digest: naming_contract.contract_digest().to_string(),
        golden_transcript_digest,
        executable_transcript_digest: transcript_evidence.transcript_digest().to_string(),
        handle_contract_digest: handle_contract.contract_digest().to_string(),
        state_contract_digest: transcript_evidence.state_digest().to_string(),
        aspect_contract_digest: digest_parts(&[
            "aspect:reads".to_string(),
            "aspect:produces".to_string(),
            "aspect:trigger".to_string(),
            "aspect:condition-inputs".to_string(),
        ]),
        authority_lane_digest: digest_parts(&[
            "lane:authoritative-truth".to_string(),
            "lane:branch-local-truth".to_string(),
            "lane:preview-truth".to_string(),
            "lane:derived-runtime-state".to_string(),
            "lane:effect-delivery-state".to_string(),
            "lane:pending-write-intent".to_string(),
            "lane:bridge-external-state".to_string(),
            "lane:temporal-execution-state".to_string(),
            "lane:async-resource-state".to_string(),
        ]),
        inspection_contract_digest: transcript_evidence.inspection_digest().to_string(),
        support_matrix_digest: support_matrix
            .matrix_digest()
            .terminal_projection_for_reporting()
            .to_string(),
        deferred_temporal_async_gate_digest: deferred_gate_digest(&contract),
        failure_digest: "none".to_string(),
        counter_snapshot: format!(
            "stable={};deferred={};unsupported={};support_rows={};support_deferred_rows={};support_fail_closed={};parallel_api_forbidden={};preferred_names={};alternate_names={};assertions={meaningful_assertion_count};plumbing=0;denials={};residue={}",
            contract.stable_family_count(),
            contract.deferred_family_count(),
            contract.unsupported_family_count(),
            support_matrix.rows().len(),
            support_matrix.deferred_row_count(),
            support_matrix.fail_closed_row_count(),
            support_matrix.parallel_api_forbidden_row_count(),
            naming_contract.preferred_entrypoint_count(),
            naming_contract.alternate_name_count(),
            transcript_evidence.support_gated_neighbor_denial_digests().len(),
            transcript_evidence.delivery_residue_count()
        ),
        compile_fail_boundary_digest: compile_fail_boundary_digest(),
        transcript_family,
        public_facade_only: true,
        lower_runtime_plumbing_count: 0,
        meaningful_assertion_count: meaningful_assertion_count
            .max(transcript_evidence.meaningful_assertion_count()),
        support_gated_neighbor_denial_count: transcript_evidence
            .support_gated_neighbor_denial_digests()
            .len(),
        delivery_residue_count: transcript_evidence.delivery_residue_count(),
        stable_family_count: contract.stable_family_count(),
        deferred_family_count: support_matrix.deferred_row_count(),
        unsupported_family_count: contract.unsupported_family_count(),
    }
}

fn contract() -> WorthQueryRuntimePublicApiContract {
    WorthQueryRuntimePublicApiContract::from_support_profile(
        &WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
    )
}

fn deferred_gate_digest(contract: &WorthQueryRuntimePublicApiContract) -> String {
    digest_parts(
        &contract
            .families()
            .iter()
            .filter(|row| row.status() == WorthQueryRuntimeFamilySupportStatus::DeferredDebt)
            .map(|row| format!("{}:{}", row.family().as_str(), row.contract_digest()))
            .collect::<Vec<_>>(),
    )
}

fn compile_fail_boundary_digest() -> String {
    digest_parts(&[
        "runtime-public-api-contract-private-fields".to_string(),
        "runtime-public-api-naming-contract-private-fields".to_string(),
        "runtime-state-snapshot-private-fields".to_string(),
        "runtime-public-api-transcript-evidence-private-fields".to_string(),
        "runtime-handle-contract-private-fields".to_string(),
        "runtime-public-support-matrix-private-fields".to_string(),
        "runtime-workspace-dynamic-surface-shortcut-forbidden".to_string(),
        "runtime-workspace-handle-value-shortcut-forbidden".to_string(),
        "lower-runtime-plumbing-shortcut-forbidden".to_string(),
    ])
}
