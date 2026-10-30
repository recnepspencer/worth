pub use worth_foundational::facade::CanonicalF64;
pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::domain;

pub use super::installed_operation_fixture::{
    artifact_move_workspace, bind_artifact_workflow, move_intent, ArtifactNativeDenial,
    ArtifactNativeLane, ArtifactNativeObservation, ArtifactNativeSuccess, ArtifactNativeValues,
    ArtifactProbe,
};

pub const ROWS: usize = 32;

pub fn run_success(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    mode: &str,
) -> (ArtifactProbe, ArtifactNativeSuccess) {
    let (mut workspace, probe) =
        artifact_move_workspace(request_execution, &format!("artifact-{mode}")).unwrap();
    bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
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

pub fn run_denial(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    mode: &str,
) -> (ArtifactProbe, ArtifactNativeDenial) {
    let (mut workspace, probe) =
        artifact_move_workspace(request_execution, &format!("artifact-{mode}")).unwrap();
    let outcome = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
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

pub fn successes(probe: &ArtifactProbe, expected: usize) -> Vec<ArtifactNativeSuccess> {
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

pub fn assert_candidates(values: &ArtifactNativeValues) {
    let ArtifactNativeValues::Candidates(candidates) = values else {
        panic!("native lane did not retain candidate rows");
    };
    assert_eq!(candidates.len(), ROWS);
    for (row, candidate) in candidates.iter().enumerate() {
        assert_eq!(candidate.id(), 1_000 + row as u64);
        assert_eq!(candidate.score(), 0.25 + row as f64 * 0.5);
    }
}

pub mod panic_disposal;
