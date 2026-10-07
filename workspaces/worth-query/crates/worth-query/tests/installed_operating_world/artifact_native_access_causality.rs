use super::installed_operation_fixture::{
    artifact_move_workspace, bind_artifact_workflow, move_intent, ArtifactNativeObservation,
    ArtifactNativeSuccess,
};
use crate::workflow_request::workflow_request;
use worth_query::facade::runtime::ExecutionRequest;

const ROWS: usize = 32;
const TWO_FIELD_BYTES: usize = ROWS * 16;

#[test]
fn native_physical_counters_move_only_with_their_declared_causal_axes() {
    let bulk = execute("native-bulk");
    let field = execute("native-field");
    let chunks = execute("native-short-chunks");
    let scalar = execute("native-scalar");
    let projection_small = execute("native-projection-small");
    let projection_wide = execute("native-projection-wide");

    assert_same_two_field_payload(&bulk, &chunks);
    assert_same_two_field_payload(&bulk, &scalar);
    assert_eq!(bulk.evidence().counters().rows_exposed, ROWS);
    assert_eq!(chunks.evidence().counters().rows_exposed, ROWS);
    assert_eq!(scalar.evidence().counters().rows_exposed, 0);
    assert_eq!(bulk.evidence().counters().provider_contacts, 1);
    assert_eq!(bulk.evidence().counters().row_batch_contacts, 1);
    assert_eq!(bulk.evidence().counters().chunk_contacts, 0);
    assert_eq!(bulk.evidence().counters().scalar_calls, 0);
    assert_eq!(chunks.evidence().counters().provider_contacts, 12);
    assert_eq!(chunks.evidence().counters().row_batch_contacts, 11);
    assert_eq!(chunks.evidence().counters().chunk_contacts, 11);
    assert_eq!(chunks.evidence().counters().scalar_calls, 0);
    assert_eq!(scalar.evidence().counters().provider_contacts, ROWS * 2);
    assert_eq!(scalar.evidence().counters().row_batch_contacts, 0);
    assert_eq!(scalar.evidence().counters().chunk_contacts, 0);
    assert_eq!(scalar.evidence().counters().scalar_calls, ROWS * 2);

    let field_counters = field.evidence().counters();
    assert_eq!(field_counters.rows_exposed, ROWS);
    assert_eq!(field_counters.values_exposed, ROWS);
    assert_eq!(field_counters.source_bytes, ROWS * 8);
    assert_eq!(field_counters.provider_contacts, 1);
    assert_eq!(field_counters.field_slice_contacts, 1);
    assert_eq!(field_counters.row_batch_contacts, 0);
    assert_eq!(bulk.evidence().counters().values_exposed, ROWS * 2);
    assert_eq!(bulk.evidence().counters().source_bytes, TWO_FIELD_BYTES);

    assert_projection_chunk_axis(&projection_small, &projection_wide);
    for success in [
        &bulk,
        &field,
        &chunks,
        &scalar,
        &projection_small,
        &projection_wide,
    ] {
        assert_eq!(success.evidence().counters().generic_row_clones, 0);
    }
}

fn assert_same_two_field_payload(left: &ArtifactNativeSuccess, right: &ArtifactNativeSuccess) {
    let left = left.evidence().counters();
    let right = right.evidence().counters();
    assert_eq!(left.values_exposed, ROWS * 2);
    assert_eq!(right.values_exposed, ROWS * 2);
    assert_eq!(left.source_bytes, TWO_FIELD_BYTES);
    assert_eq!(right.source_bytes, TWO_FIELD_BYTES);
}

fn assert_projection_chunk_axis(small: &ArtifactNativeSuccess, wide: &ArtifactNativeSuccess) {
    let small = small.evidence().counters();
    let wide = wide.evidence().counters();
    assert_eq!(small.rows_exposed, wide.rows_exposed);
    assert_eq!(small.values_exposed, wide.values_exposed);
    assert_eq!(small.source_bytes, wide.source_bytes);
    assert_eq!(small.result_bytes, wide.result_bytes);
    assert_eq!(small.projection_contacts, 8);
    assert_eq!(wide.projection_contacts, 1);
    assert_eq!(small.provider_contacts, 9);
    assert_eq!(wide.provider_contacts, 2);
    assert!(small.peak_result_capacity_bytes < wide.peak_result_capacity_bytes);
}

fn execute(mode: &str) -> ArtifactNativeSuccess {
    let (mut workspace, probe) =
        artifact_move_workspace(&format!("artifact-native-causal-{mode}")).unwrap();
    bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            move_intent(mode),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();
    let observations = probe.take_native_observations();
    assert_eq!(observations.len(), 1);
    let ArtifactNativeObservation::Success(success) = observations.into_iter().next().unwrap()
    else {
        panic!("causal native-access case was denied");
    };
    success
}
