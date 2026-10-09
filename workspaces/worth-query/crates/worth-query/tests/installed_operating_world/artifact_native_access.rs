use worth_foundational::facade::CanonicalF64;
use worth_proof::TransitionOutcome;
use worth_query::facade::domain;

use super::installed_operation_fixture::{
    artifact_move_workspace, bind_artifact_workflow, move_intent, ArtifactNativeDenial,
    ArtifactNativeLane, ArtifactNativeObservation, ArtifactNativeSuccess, ArtifactNativeValues,
    ArtifactProbe,
};

const ROWS: usize = 32;

#[test]
fn bulk_and_scalar_lanes_preserve_semantics_basis_and_distinct_physical_work() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-native-parity").unwrap();
    bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(move_intent("native-bulk"), &mut workspace)
        .unwrap();
    bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(move_intent("native-scalar"), &mut workspace)
        .unwrap();
    let observations = successes(&probe, 2);
    let bulk = &observations[0];
    let scalar = &observations[1];

    assert_eq!(bulk.lane(), ArtifactNativeLane::BulkRows);
    assert_eq!(scalar.lane(), ArtifactNativeLane::ScalarFallback);
    assert_candidates(bulk.values());
    assert_candidates(scalar.values());
    assert_eq!(
        bulk.evidence().basis_identity(),
        scalar.evidence().basis_identity()
    );
    assert_eq!(bulk.evidence().layout(), scalar.evidence().layout());
    assert_eq!(
        bulk.evidence().access_bound(),
        &domain::WorthQueryArtifactNativeAccessBound::RowBatch {
            start_row: 0,
            max_rows: ROWS,
        }
    );
    assert_eq!(
        scalar.evidence().access_bound(),
        &domain::WorthQueryArtifactNativeAccessBound::ScalarFallback {
            max_calls_per_admission: 64,
            max_call_amplification: 32,
        }
    );
    let bulk_work = bulk.evidence().counters();
    assert_eq!(bulk_work.provider_contacts, 1);
    assert_eq!(bulk_work.row_batch_contacts, 1);
    assert_eq!(bulk_work.scalar_calls, 0);
    assert_eq!(bulk_work.source_bytes, ROWS * 16);
    assert_eq!(bulk_work.generic_row_clones, 0);
    let scalar_work = scalar.evidence().counters();
    assert_eq!(scalar_work.provider_contacts, ROWS * 2);
    assert_eq!(scalar_work.row_batch_contacts, 0);
    assert_eq!(scalar_work.scalar_calls, ROWS * 2);
    assert_eq!(scalar_work.source_bytes, ROWS * 16);
    assert_eq!(scalar_work.generic_row_clones, 0);
    assert_eq!(probe.native_row_batches(), 1);
    assert_eq!(probe.native_scalars(), ROWS * 2);
    assert_eq!(probe.borrow_observations(), 0);
    assert_eq!(probe.disposals(), 2);

    let retained = b"canonical-candidates".len()
        + ROWS * (4 * std::mem::size_of::<u64>() + std::mem::size_of::<CanonicalF64>());
    assert!(probe
        .lifecycle_snapshots()
        .iter()
        .all(|snapshot| snapshot.counters().retained_bytes == retained));
}

#[test]
fn field_slice_and_short_chunks_expose_all_rows_without_continuation_skips() {
    let (field_probe, field) = run_success("native-field");
    assert_eq!(field.lane(), ArtifactNativeLane::FieldSlice);
    let ArtifactNativeValues::CandidateIds(ids) = field.values() else {
        panic!("field lane did not retain candidate ids");
    };
    assert_eq!(
        ids,
        &(0..ROWS).map(|row| 1_000 + row as u64).collect::<Vec<_>>()
    );
    let field_work = field.evidence().counters();
    assert_eq!(
        field.evidence().access_bound(),
        &domain::WorthQueryArtifactNativeAccessBound::FieldSlice {
            start_row: 0,
            max_rows: ROWS,
        }
    );
    assert_eq!(field_work.provider_contacts, 1);
    assert_eq!(field_work.field_slice_contacts, 1);
    assert_eq!(field_work.source_bytes, ROWS * 8);
    assert_eq!(field_probe.native_field_slices(), 1);

    let (chunk_probe, chunked) = run_success("native-short-chunks");
    assert_eq!(chunked.lane(), ArtifactNativeLane::ChunkedRows);
    assert_candidates(chunked.values());
    let chunk_work = chunked.evidence().counters();
    assert_eq!(
        chunked.evidence().access_bound(),
        &domain::WorthQueryArtifactNativeAccessBound::Chunk { chunk_rows: 8 }
    );
    assert_eq!(chunk_work.provider_contacts, 12);
    assert_eq!(chunk_work.row_batch_contacts, 11);
    assert_eq!(chunk_work.chunk_contacts, 11);
    assert_eq!(chunk_work.rows_exposed, ROWS);
    assert_eq!(chunk_work.source_bytes, ROWS * 16);
    assert_eq!(chunk_probe.native_row_counts(), 1);
    assert_eq!(chunk_probe.native_row_batches(), 11);
}

#[test]
fn projection_chunk_width_controls_actual_allocated_capacity() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-native-memory").unwrap();
    bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(move_intent("native-projection-small"), &mut workspace)
        .unwrap();
    bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(move_intent("native-projection-wide"), &mut workspace)
        .unwrap();
    let observations = successes(&probe, 2);
    let small = &observations[0];
    let wide = &observations[1];

    assert_eq!(small.lane(), ArtifactNativeLane::SummaryProjection);
    assert_eq!(wide.lane(), ArtifactNativeLane::SummaryProjection);
    assert_candidates(small.values());
    assert_candidates(wide.values());
    assert_eq!(
        small.evidence().access_bound(),
        &domain::WorthQueryArtifactNativeAccessBound::Projection {
            projection_identity: "candidate-summary-v1".into(),
            chunk_rows: 4,
        }
    );
    assert_eq!(
        wide.evidence().access_bound(),
        &domain::WorthQueryArtifactNativeAccessBound::Projection {
            projection_identity: "candidate-summary-v1".into(),
            chunk_rows: ROWS,
        }
    );
    assert_eq!(small.chunk_capacity_bytes().len(), 8);
    assert_eq!(wide.chunk_capacity_bytes().len(), 1);
    let small_peak = *small.chunk_capacity_bytes().iter().max().unwrap();
    let wide_peak = *wide.chunk_capacity_bytes().iter().max().unwrap();
    assert!(small_peak < wide_peak);
    assert_eq!(
        small.evidence().counters().peak_result_capacity_bytes,
        small_peak
    );
    assert_eq!(
        wide.evidence().counters().peak_result_capacity_bytes,
        wide_peak
    );
    assert_eq!(small.evidence().counters().projection_contacts, 8);
    assert_eq!(wide.evidence().counters().projection_contacts, 1);
    assert_eq!(small.evidence().counters().generic_row_clones, 0);
    assert_eq!(wide.evidence().counters().generic_row_clones, 0);
    assert_eq!(probe.native_projections(), 9);
}

#[test]
fn opaque_reference_and_content_fields_require_the_declared_native_projection() {
    let (probe, projected) = run_success("native-provenance");
    assert_eq!(projected.lane(), ArtifactNativeLane::ProvenanceProjection);
    let ArtifactNativeValues::Signatures(signatures) = projected.values() else {
        panic!("provenance lane did not retain signatures");
    };
    let expected = (0..ROWS)
        .map(|row| (0xA500 + row as u64) ^ (0xB600 + row as u64 * 3) ^ (0xC700 + row as u64 * 5))
        .collect::<Vec<_>>();
    assert_eq!(signatures, &expected);
    assert_eq!(projected.evidence().requested_fields().len(), 3);
    assert_eq!(projected.evidence().counters().projection_contacts, 4);
    assert_eq!(probe.native_projections(), 4);

    for (mode, kind) in [
        (
            "native-denied-direct",
            domain::WorthQueryArtifactNativeAccessDenialKind::ProviderNativeProjectionRequired,
        ),
        (
            "native-denied-field",
            domain::WorthQueryArtifactNativeAccessDenialKind::FieldSliceDenied,
        ),
    ] {
        let (probe, denial) = run_denial(mode);
        assert_eq!(denial.kind(), kind);
        assert_eq!(denial.counters().provider_contacts, 0);
        assert_eq!(probe.native_row_batches(), 0);
        assert_eq!(probe.native_field_slices(), 0);
    }
}

#[test]
fn layout_bounds_session_and_progress_denials_stop_at_the_responsible_boundary() {
    for (mode, kind, provider_contacts, row_counts, row_batches, scalar_calls) in [
        (
            "native-wrong-layout",
            domain::WorthQueryArtifactNativeAccessDenialKind::LayoutMismatch,
            0,
            0,
            0,
            0,
        ),
        (
            "native-chunk-too-wide",
            domain::WorthQueryArtifactNativeAccessDenialKind::BoundsExceeded,
            0,
            0,
            0,
            0,
        ),
        (
            "native-scalar-amplification",
            domain::WorthQueryArtifactNativeAccessDenialKind::BoundsExceeded,
            ROWS,
            0,
            0,
            ROWS,
        ),
        (
            "native-provider-alignment",
            domain::WorthQueryArtifactNativeAccessDenialKind::AlignmentMismatch,
            1,
            0,
            0,
            0,
        ),
        (
            "native-session-mismatch",
            domain::WorthQueryArtifactNativeAccessDenialKind::ProviderSessionMismatch,
            1,
            0,
            0,
            0,
        ),
        (
            "native-zero-progress",
            domain::WorthQueryArtifactNativeAccessDenialKind::ProviderDenied,
            2,
            1,
            1,
            0,
        ),
        (
            "native-zero-projection-progress",
            domain::WorthQueryArtifactNativeAccessDenialKind::ProviderDenied,
            2,
            1,
            0,
            0,
        ),
    ] {
        let (probe, denial) = run_denial(mode);
        assert_eq!(denial.kind(), kind, "{mode}");
        assert_eq!(
            denial.counters().provider_contacts,
            provider_contacts,
            "{mode}"
        );
        assert_eq!(probe.native_row_counts(), row_counts, "{mode}");
        assert_eq!(probe.native_row_batches(), row_batches, "{mode}");
        assert_eq!(probe.native_scalars(), scalar_calls, "{mode}");
        assert_eq!(probe.borrow_observations(), 0, "{mode}");
        assert_eq!(probe.disposals(), 1, "{mode}");
    }
}

fn run_success(mode: &str) -> (ArtifactProbe, ArtifactNativeSuccess) {
    let (mut workspace, probe) = artifact_move_workspace(&format!("artifact-{mode}")).unwrap();
    bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(move_intent(mode), &mut workspace)
        .unwrap();
    let mut observations = successes(&probe, 1);
    let success = observations.remove(0);
    assert_eq!(probe.borrow_observations(), 0);
    assert_eq!(probe.disposals(), 1);
    (probe, success)
}

fn run_denial(mode: &str) -> (ArtifactProbe, ArtifactNativeDenial) {
    let (mut workspace, probe) = artifact_move_workspace(&format!("artifact-{mode}")).unwrap();
    let outcome = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(move_intent(mode), &mut workspace);
    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_)
    ));
    let observations = probe.take_native_observations();
    assert_eq!(observations.len(), 1);
    let ArtifactNativeObservation::Denied(denial) = observations.into_iter().next().unwrap() else {
        panic!("native denial scenario recorded success");
    };
    (probe, denial)
}

fn successes(probe: &ArtifactProbe, expected: usize) -> Vec<ArtifactNativeSuccess> {
    let observations = probe.take_native_observations();
    assert_eq!(observations.len(), expected);
    observations
        .into_iter()
        .map(|observation| match observation {
            ArtifactNativeObservation::Success(success) => success,
            ArtifactNativeObservation::Denied(denial) => {
                panic!("native success scenario denied: {:?}", denial.kind())
            }
        })
        .collect()
}

fn assert_candidates(values: &ArtifactNativeValues) {
    let ArtifactNativeValues::Candidates(candidates) = values else {
        panic!("native lane did not retain candidate rows");
    };
    assert_eq!(candidates.len(), ROWS);
    for (row, candidate) in candidates.iter().enumerate() {
        assert_eq!(candidate.id(), 1_000 + row as u64);
        assert_eq!(candidate.score(), 0.25 + row as f64 * 0.5);
    }
}

mod panic_disposal;
