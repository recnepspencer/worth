pub use super::installed_operation_fixture::{
    artifact_move_workspace, bind_artifact_workflow, move_intent, ArtifactNativeObservation,
    ArtifactNativeSuccess,
};

pub const ROWS: usize = 32;
pub const TWO_FIELD_BYTES: usize = ROWS * 16;

pub fn assert_same_two_field_payload(left: &ArtifactNativeSuccess, right: &ArtifactNativeSuccess) {
    let left = left.evidence().counters();
    let right = right.evidence().counters();
    assert_eq!(left.values_exposed, ROWS * 2);
    assert_eq!(right.values_exposed, ROWS * 2);
    assert_eq!(left.source_bytes, TWO_FIELD_BYTES);
    assert_eq!(right.source_bytes, TWO_FIELD_BYTES);
}

pub fn assert_projection_chunk_axis(small: &ArtifactNativeSuccess, wide: &ArtifactNativeSuccess) {
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

pub fn execute(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    mode: &str,
) -> ArtifactNativeSuccess {
    let (mut workspace, probe) =
        artifact_move_workspace(request_execution, &format!("artifact-native-causal-{mode}"))
            .unwrap();
    bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(move_intent(mode), &mut workspace)
        .unwrap();
    let observations = probe.take_native_observations();
    assert_eq!(observations.len(), 1);
    let ArtifactNativeObservation::Success(success) = observations.into_iter().next().unwrap()
    else {
        panic!("causal native-access case was denied");
    };
    success
}
