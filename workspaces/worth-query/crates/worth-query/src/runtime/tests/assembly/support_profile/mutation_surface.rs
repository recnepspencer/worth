use super::super::super::support::*;

#[test]
fn runtime_public_support_gate_keeps_support_gated_rows_fail_closed_for_ordinary_dx() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.support-gated-invariant")
        .expect("task runtime should open a named workspace");
    let matrix = workspace.public_support_matrix();

    for family in [
        WorthQueryRuntimeFacadeFamily::Temporal,
        WorthQueryRuntimeFacadeFamily::AsyncResource,
        WorthQueryRuntimeFacadeFamily::MixedCauseDelivery,
    ] {
        let matrix_row = matrix
            .row_for_family(family)
            .expect("support-matrix row should exist");

        assert_eq!(
            matrix_row.status(),
            WorthQueryRuntimeFamilySupportStatus::Supported
        );
        assert!(matrix_row.admission_fail_closed());
        assert!(matrix_row.parallel_api_forbidden());
        assert!(!matrix_row.ordinary_downstream_dx());
        assert_eq!(
            matrix_row.teaching_posture(),
            WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly
        );

        let error = workspace
            .admit_public_api_family(family)
            .expect_err("support-gated runtime-backed family should fail closed at admission");
        match error {
            WorthQueryRuntimeError::UnsupportedFacadeFamily(denial) => {
                assert_eq!(denial.family(), family);
                assert_eq!(
                    denial.status(),
                    WorthQueryRuntimeFamilySupportStatus::Supported
                );
                assert_eq!(
                    denial.teaching_posture(),
                    Some(WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly)
                );
            }
            other => panic!("expected support-gated admission denial, got {other:?}"),
        }
    }
}

#[test]
fn runtime_public_mutation_surface_report_lists_only_live_lower_level_command_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.mutation-surface")
        .expect("task runtime should open a named workspace");
    let report = workspace.public_mutation_surface_report();

    assert_eq!(report.lower_level_stable_count(), 4);
    assert_eq!(report.support_gated_count(), 2);
    assert!(report
        .row_by_surface("WorthQueryWriteCommand::Insert")
        .is_none());
    assert_eq!(
        report
            .row_by_surface("workspace.submissions()?.submit_batch(commands)")
            .expect("submission batch row should exist")
            .posture(),
        WorthQueryMutationSurfacePosture::PreferredStable
    );
    assert!(report.row_by_surface("workspace.batch(...)").is_none());
    assert!(report.row_by_surface("workspace.write(...)").is_none());
    assert_eq!(
        report
            .row_by_surface("WorthQueryWriteCommand::InsertAspects")
            .expect("aspect insert command row should exist")
            .posture(),
        WorthQueryMutationSurfacePosture::LowerLevelStable
    );
}
