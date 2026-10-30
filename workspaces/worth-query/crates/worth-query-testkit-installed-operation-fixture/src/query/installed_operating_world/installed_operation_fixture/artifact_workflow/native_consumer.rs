pub use std::cell::Cell;

pub use worth_foundational::facade::AspectValue;
pub use worth_query::facade::domain;

pub use super::contract::{
    candidate_id, candidate_layout, candidate_score, candidate_token, foreign_candidate_layout,
};
pub use super::integrated_evidence::integrated_evidence;
pub use super::native_observation::{
    ArtifactNativeCandidate, ArtifactNativeDenial, ArtifactNativeLane, ArtifactNativeObservation,
    ArtifactNativeSuccess, ArtifactNativeValues,
};
pub use super::native_provider::CANDIDATE_ROWS;
pub use super::provider::ArtifactProbe;

pub type NativeResult = Result<ArtifactNativeSuccess, domain::WorthQueryArtifactNativeAccessDenial>;

pub fn execute_native_consumer(
    mode: &str,
    transferred: domain::WorthQueryTransferredArtifactHandle,
    workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
    probe: &ArtifactProbe,
) -> Result<domain::WorthQueryWorkflowStageMaterial, domain::WorthQueryWorkflowStageExecutorFailure>
{
    let reader = workspace
        .artifact_reader(&transferred)
        .map_err(|denial| record_denial(probe, denial))?;
    let result = match mode {
        "native-bulk" | "native-integrated" | "native-provider-panic" => consume_bulk_rows(reader),
        "native-field" => consume_field_slice(reader),
        "native-short-chunks" => consume_chunks(reader),
        "native-scalar" => consume_scalar_fallback(reader),
        "native-projection-small" => consume_summary_projection(reader, 4),
        "native-projection-wide" => consume_summary_projection(reader, CANDIDATE_ROWS),
        "native-provenance" => consume_provenance_projection(reader),
        "native-denied-direct" => deny_direct_opaque_access(reader),
        "native-denied-field" => deny_opaque_field_slice(reader),
        "native-wrong-layout" => deny_wrong_layout(reader),
        "native-chunk-too-wide" => deny_chunk_bound(reader),
        "native-scalar-amplification" => deny_scalar_amplification(reader),
        "native-provider-alignment" | "native-session-mismatch" => consume_bulk_rows(reader),
        "native-zero-progress" => deny_zero_progress(reader),
        "native-zero-projection-progress" => deny_zero_projection_progress(reader),
        _ => panic!("unknown native artifact scenario: {mode}"),
    };
    match result {
        Ok(success) => {
            let evidence = (mode == "native-integrated").then(|| integrated_evidence(&success));
            probe.observe_native_access(ArtifactNativeObservation::Success(success));
            let material = ready_material();
            Ok(match evidence {
                Some(evidence) => material.with_domain_evidence(evidence),
                None => material,
            })
        }
        Err(denial) => Err(record_denial(probe, denial)),
    }
}

pub fn consume_bulk_rows(reader: domain::WorthQueryStageArtifactReader<'_>) -> NativeResult {
    let outcome = reader.with_rows(
        row_request(candidate_layout(), CANDIDATE_ROWS),
        collect_rows,
    )?;
    Ok(success(
        ArtifactNativeLane::BulkRows,
        ArtifactNativeValues::Candidates(outcome.value().clone()),
        Vec::new(),
        outcome.evidence().clone(),
    ))
}

pub fn consume_field_slice(reader: domain::WorthQueryStageArtifactReader<'_>) -> NativeResult {
    let outcome = reader.with_field_slice(
        domain::WorthQueryArtifactFieldSliceRequest::new(
            candidate_layout(),
            candidate_id(),
            CANDIDATE_ROWS,
        ),
        |values| {
            (0..values.len())
                .map(|row| {
                    values
                        .value(row)
                        .and_then(|value| value.as_u64())
                        .expect("installed candidate id slice is UInt64")
                })
                .collect::<Vec<_>>()
        },
    )?;
    Ok(success(
        ArtifactNativeLane::FieldSlice,
        ArtifactNativeValues::CandidateIds(outcome.value().clone()),
        Vec::new(),
        outcome.evidence().clone(),
    ))
}

pub fn consume_chunks(reader: domain::WorthQueryStageArtifactReader<'_>) -> NativeResult {
    let mut cursor = reader.chunks(domain::WorthQueryArtifactChunkRequest::new(
        candidate_layout(),
        [candidate_id(), candidate_score()],
        8,
    ))?;
    let mut rows = Vec::new();
    while cursor
        .next(|batch| rows.extend(collect_rows(batch)))?
        .is_some()
    {}
    Ok(success(
        ArtifactNativeLane::ChunkedRows,
        ArtifactNativeValues::Candidates(rows),
        Vec::new(),
        cursor.evidence(),
    ))
}

pub fn consume_scalar_fallback(reader: domain::WorthQueryStageArtifactReader<'_>) -> NativeResult {
    let id = candidate_id();
    let score = candidate_score();
    let mut session =
        reader.scalar_fallback(domain::WorthQueryArtifactScalarFallbackRequest::new(
            candidate_layout(),
            [id.clone(), score.clone()],
        ))?;
    let mut rows = Vec::with_capacity(CANDIDATE_ROWS);
    for row in 0..CANDIDATE_ROWS {
        let id = session.with_value(row, &id, |value| {
            value.as_u64().expect("candidate id scalar is UInt64")
        })?;
        let score = session.with_value(row, &score, |value| {
            value.as_f64().expect("candidate score scalar is Float64")
        })?;
        rows.push(ArtifactNativeCandidate::new(id, score));
    }
    Ok(success(
        ArtifactNativeLane::ScalarFallback,
        ArtifactNativeValues::Candidates(rows),
        Vec::new(),
        session.evidence(),
    ))
}

pub fn consume_summary_projection(
    reader: domain::WorthQueryStageArtifactReader<'_>,
    chunk_rows: usize,
) -> NativeResult {
    let mut cursor =
        reader.projected_chunks(domain::WorthQueryArtifactProjectedChunkRequest::new(
            candidate_layout(),
            "candidate-summary-v1",
            chunk_rows,
        ))?;
    let mut rows = Vec::new();
    let mut capacities = Vec::new();
    while cursor
        .next(|chunk| {
            capacities.push(chunk.allocated_capacity_bytes());
            for row in 0..chunk.row_count() {
                let Some([AspectValue::UInt64(id), AspectValue::Float64(score)]) = chunk.row(row)
                else {
                    panic!("installed summary projection has id and score columns");
                };
                rows.push(ArtifactNativeCandidate::new(*id, score.as_f64()));
            }
        })?
        .is_some()
    {}
    Ok(success(
        ArtifactNativeLane::SummaryProjection,
        ArtifactNativeValues::Candidates(rows),
        capacities,
        cursor.evidence(),
    ))
}

pub fn consume_provenance_projection(
    reader: domain::WorthQueryStageArtifactReader<'_>,
) -> NativeResult {
    let mut cursor =
        reader.projected_chunks(domain::WorthQueryArtifactProjectedChunkRequest::new(
            candidate_layout(),
            "candidate-provenance-v1",
            8,
        ))?;
    let mut signatures = Vec::new();
    let mut capacities = Vec::new();
    while cursor
        .next(|chunk| {
            capacities.push(chunk.allocated_capacity_bytes());
            for row in 0..chunk.row_count() {
                let Some([AspectValue::UInt64(signature)]) = chunk.row(row) else {
                    panic!("installed provenance projection has one signature column");
                };
                signatures.push(*signature);
            }
        })?
        .is_some()
    {}
    Ok(success(
        ArtifactNativeLane::ProvenanceProjection,
        ArtifactNativeValues::Signatures(signatures),
        capacities,
        cursor.evidence(),
    ))
}

pub fn deny_direct_opaque_access(
    reader: domain::WorthQueryStageArtifactReader<'_>,
) -> NativeResult {
    expected_denial(reader.with_rows(
        domain::WorthQueryArtifactRowBatchRequest::new(
            candidate_layout(),
            [candidate_token()],
            CANDIDATE_ROWS,
        ),
        |_| (),
    ))
}

pub fn deny_opaque_field_slice(reader: domain::WorthQueryStageArtifactReader<'_>) -> NativeResult {
    expected_denial(reader.with_field_slice(
        domain::WorthQueryArtifactFieldSliceRequest::new(
            candidate_layout(),
            candidate_token(),
            CANDIDATE_ROWS,
        ),
        |_| (),
    ))
}

pub fn deny_wrong_layout(reader: domain::WorthQueryStageArtifactReader<'_>) -> NativeResult {
    expected_denial(reader.with_rows(
        row_request(foreign_candidate_layout(), CANDIDATE_ROWS),
        |_| (),
    ))
}

pub fn deny_chunk_bound(reader: domain::WorthQueryStageArtifactReader<'_>) -> NativeResult {
    match reader.chunks(domain::WorthQueryArtifactChunkRequest::new(
        candidate_layout(),
        [candidate_id(), candidate_score()],
        65,
    )) {
        Err(denial) => Err(denial),
        Ok(_) => panic!("chunk request exceeded the installed maximum"),
    }
}

pub fn deny_scalar_amplification(
    reader: domain::WorthQueryStageArtifactReader<'_>,
) -> NativeResult {
    let id = candidate_id();
    let mut session = reader.scalar_fallback(
        domain::WorthQueryArtifactScalarFallbackRequest::new(candidate_layout(), [id.clone()]),
    )?;
    for row in 0..CANDIDATE_ROWS {
        session.with_value(row, &id, |_| ())?;
    }
    match session.with_value(0, &id, |_| ()) {
        Err(denial) => Err(denial),
        Ok(_) => panic!("scalar fallback exceeded its admitted amplification"),
    }
}

pub fn deny_zero_progress(reader: domain::WorthQueryStageArtifactReader<'_>) -> NativeResult {
    let mut cursor = reader.chunks(domain::WorthQueryArtifactChunkRequest::new(
        candidate_layout(),
        [candidate_id(), candidate_score()],
        8,
    ))?;
    let callback_invoked = Cell::new(false);
    match cursor.next(|_| callback_invoked.set(true)) {
        Err(denial) => {
            assert!(
                !callback_invoked.get(),
                "invalid nonterminal chunk reached the consumer callback"
            );
            Err(denial)
        }
        Ok(_) => panic!("zero-progress provider returned a usable native chunk"),
    }
}

pub fn deny_zero_projection_progress(
    reader: domain::WorthQueryStageArtifactReader<'_>,
) -> NativeResult {
    let mut cursor =
        reader.projected_chunks(domain::WorthQueryArtifactProjectedChunkRequest::new(
            candidate_layout(),
            "candidate-summary-v1",
            8,
        ))?;
    let callback_invoked = Cell::new(false);
    match cursor.next(|_| callback_invoked.set(true)) {
        Err(denial) => {
            assert!(
                !callback_invoked.get(),
                "invalid nonterminal projection reached the consumer callback"
            );
            Err(denial)
        }
        Ok(_) => panic!("zero-progress provider returned a usable projected chunk"),
    }
}

pub fn row_request(
    layout: domain::WorthQueryArtifactNativeLayoutReference,
    max_rows: usize,
) -> domain::WorthQueryArtifactRowBatchRequest {
    domain::WorthQueryArtifactRowBatchRequest::new(
        layout,
        [candidate_id(), candidate_score()],
        max_rows,
    )
}

pub fn collect_rows(
    batch: domain::WorthQueryArtifactBorrowedRowBatch<'_>,
) -> Vec<ArtifactNativeCandidate> {
    let id = candidate_id();
    let score = candidate_score();
    batch
        .rows()
        .map(|row| {
            ArtifactNativeCandidate::new(
                row.field(&id)
                    .and_then(|value| value.as_u64())
                    .expect("candidate row id is UInt64"),
                row.field(&score)
                    .and_then(|value| value.as_f64())
                    .expect("candidate row score is Float64"),
            )
        })
        .collect()
}

pub fn expected_denial<T>(
    outcome: Result<
        domain::WorthQueryArtifactNativeAccessOutcome<T>,
        domain::WorthQueryArtifactNativeAccessDenial,
    >,
) -> NativeResult {
    match outcome {
        Err(denial) => Err(denial),
        Ok(_) => panic!("hostile native access scenario unexpectedly succeeded"),
    }
}

pub fn success(
    lane: ArtifactNativeLane,
    values: ArtifactNativeValues,
    capacities: Vec<usize>,
    evidence: domain::WorthQueryArtifactNativeAccessEvidence,
) -> ArtifactNativeSuccess {
    ArtifactNativeSuccess::new(lane, values, capacities, evidence)
}

pub fn record_denial(
    probe: &ArtifactProbe,
    denial: domain::WorthQueryArtifactNativeAccessDenial,
) -> domain::WorthQueryWorkflowStageExecutorFailure {
    probe.observe_native_access(ArtifactNativeObservation::Denied(
        ArtifactNativeDenial::new(&denial),
    ));
    domain::WorthQueryWorkflowStageExecutorFailure::new(
        domain::WorthQueryOperationFailureClass::Dependency,
        format!("{:?}: {}", denial.kind(), denial.detail()),
    )
}

pub fn ready_material() -> domain::WorthQueryWorkflowStageMaterial {
    domain::WorthQueryWorkflowStageMaterial::new(domain::WorthQueryWorkflowValue::Text(
        "native-artifact-consumed".into(),
    ))
    .with_result_state(domain::WorthQueryOperationResultState::Ready)
}
