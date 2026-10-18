use super::*;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::indexes::data::{DerivedIndexMaintenanceAdmissionStop, DerivedIndexMaintenanceBudget};

fn fixture(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    RelationalRuntime,
    AdmittedRelationalBranchBasis,
    DerivedIndexBuildRequest,
) {
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "selected-field");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "admitted.selected-field".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let request = DerivedIndexBuildRequest {
        source_commit_id: first.commit.commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![index.index_id],
    };
    release_test_commit_snapshot(&runtime, &first);
    (runtime, basis, request)
}

fn budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(1_000_000),
        maximum_cold_record_slots: Some(100_000),
        maximum_derived_rows: Some(100_000),
    }
}

#[test]
fn selected_field_cold_admission_matches_native_build_and_warm_reuse_is_not_cold() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, basis, request) = fixture(execution);
    let mut work = 0;
    let mut bytes = 0;
    let refreshed = runtime
        .index_authority()
        .refresh_field_indexes_for_basis_admitted(
            request.clone(),
            &basis,
            budget(),
            execution,
            |units, allocation| {
                work += units;
                bytes += allocation;
                Ok::<_, ()>(())
            },
        )
        .unwrap();
    assert!(work > 0 && bytes > 0);
    assert!(refreshed.work.cold_record_slots > 0);
    let rebuilt = runtime
        .index_authority()
        .build_for_basis(execution, request.clone(), &basis);
    assert!(rebuilt.failed_indexes.is_empty());
    assert_eq!(
        refreshed.generations[0].entries,
        rebuilt.generations[0].entries
    );
    let warm = runtime
        .index_authority()
        .refresh_field_indexes_for_basis_admitted(
            request,
            &basis,
            DerivedIndexMaintenanceBudget {
                maximum_cold_record_slots: Some(0),
                ..budget()
            },
            execution,
            |_, _| Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(warm.work.cold_record_slots, 0);
    assert_eq!(warm.work.reused_generations, 1);
    assert_eq!(warm.generations[0].entries, rebuilt.generations[0].entries);
}

#[derive(Debug, Eq, PartialEq)]
enum OriginalStop {
    Work,
    Bytes,
    Cancelled,
}

#[test]
fn selected_field_refusals_preserve_original_stop_and_publish_no_partial_generation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (measure_runtime, measure_basis, measure_request) = fixture(execution);
    let (mut total_work, mut total_bytes, mut calls) = (0, 0, 0);
    measure_runtime
        .index_authority()
        .refresh_field_indexes_for_basis_admitted(
            measure_request,
            &measure_basis,
            budget(),
            execution,
            |work, bytes| {
                total_work += work;
                total_bytes += bytes;
                calls += 1;
                Ok::<_, OriginalStop>(())
            },
        )
        .unwrap();
    for (work_limit, byte_limit, cancel_call, expected) in [
        (0, u64::MAX, usize::MAX, OriginalStop::Work),
        (total_work - 1, u64::MAX, usize::MAX, OriginalStop::Work),
        (u64::MAX, total_bytes - 1, usize::MAX, OriginalStop::Bytes),
        (u64::MAX, u64::MAX, calls, OriginalStop::Cancelled),
    ] {
        let (runtime, basis, request) = fixture(execution);
        let before = runtime.current_version_id();
        let (mut work_used, mut bytes_used, mut call) = (0, 0, 0);
        let denied = runtime
            .index_authority()
            .refresh_field_indexes_for_basis_admitted(
                request.clone(),
                &basis,
                budget(),
                execution,
                |work, bytes| {
                    call += 1;
                    if call == cancel_call {
                        return Err(OriginalStop::Cancelled);
                    }
                    if work > work_limit - work_used {
                        return Err(OriginalStop::Work);
                    }
                    if bytes > byte_limit - bytes_used {
                        return Err(OriginalStop::Bytes);
                    }
                    work_used += work;
                    bytes_used += bytes;
                    Ok(())
                },
            )
            .unwrap_err();
        let DerivedIndexMaintenanceAdmissionStop::Admission(stop) = denied else {
            panic!("original caller stop")
        };
        assert_eq!(stop, expected);
        assert_eq!(runtime.current_version_id(), before);
        assert!(runtime
            .index_access()
            .published_generation_for_observation(request.index_ids[0], &basis.observation(),)
            .is_none());
    }
}

#[test]
fn request_paid_selected_field_refresh_leaves_unrelated_cold_index_unpublished() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (runtime, basis, request) = fixture(execution);
    let unrelated = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "unrelated-cold-field".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: true,
    });
    let selected = request.index_ids[0];
    let observation = basis.observation();
    let mut refused_budget = budget();
    refused_budget.maximum_work_units = Some(0);
    assert!(runtime
        .index_authority()
        .refresh_field_indexes_for_basis(request.clone(), &basis, None, refused_budget, execution)
        .is_err());
    assert!(runtime
        .index_access()
        .published_generation_for_observation(selected, &observation)
        .is_none());
    let cold = runtime
        .index_authority()
        .refresh_field_indexes_for_basis(request.clone(), &basis, None, budget(), execution)
        .unwrap_err();
    assert!(matches!(
        cold.kind,
        crate::indexes::data::DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
    ));
    let no_population = DerivedIndexMaintenanceBudget {
        maximum_cold_record_slots: Some(0),
        ..budget()
    };
    let admitted_cold = runtime
        .index_authority()
        .refresh_field_indexes_for_basis_admitted(
            request.clone(),
            &basis,
            no_population,
            execution,
            |_, _| Ok::<_, ()>(()),
        )
        .unwrap_err();
    assert!(matches!(
        admitted_cold,
        DerivedIndexMaintenanceAdmissionStop::Native(ref cause)
            if matches!(cause.kind,
                crate::indexes::data::DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired)
    ));
    assert!(runtime
        .index_access()
        .published_generation_for_observation(selected, &observation)
        .is_none());
    // Genuine native bootstrap builds the authoring generation; the ordinary selector
    // path then seeds this exact root on a fresh branch without a population scan.
    assert!(runtime
        .index_authority()
        .build_for_basis(execution, request.clone(), &basis)
        .failed_indexes
        .is_empty());
    let fork = BranchId("selector-fork".into());
    let (_, fork_source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    let forked = runtime.fork_branch(fork.clone(), fork_source).unwrap();
    let (_, fork_basis) = runtime.observe_branch(forked.target_identity()).unwrap();
    let mut fork_request = request;
    fork_request.branch_id = fork;
    let outcome = runtime
        .index_authority()
        .refresh_field_indexes_for_basis(fork_request, &fork_basis, None, budget(), execution)
        .unwrap();
    let observation = fork_basis.observation();
    assert_eq!(outcome.generations.len(), 1);
    assert_eq!(outcome.generations[0].index_id, selected);
    assert!(runtime
        .index_access()
        .published_generation_for_observation(selected, &observation)
        .is_some());
    assert!(runtime
        .index_access()
        .published_generation_for_observation(unrelated.index_id, &observation)
        .is_none());
    assert_eq!(
        outcome.work.cold_record_slots, 0,
        "exact authoring generation seeds without population reconstruction"
    );
    let admitted_warm = runtime
        .index_authority()
        .refresh_field_indexes_for_basis_admitted(
            DerivedIndexBuildRequest {
                source_commit_id: observation.commit_id().unwrap(),
                branch_id: observation.identity().branch_id().clone(),
                index_ids: vec![selected],
            },
            &fork_basis,
            no_population,
            execution,
            |_, _| Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(admitted_warm.work.cold_record_slots, 0);
    assert_eq!(admitted_warm.work.reused_generations, 1);
    // A real data patch cannot masquerade as the unchanged-record seed.
    let predecessor = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let changed = create_entity_outcome(execution, &runtime, "changed-field");
    let (_, changed_basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let changed_request = DerivedIndexBuildRequest {
        source_commit_id: changed.commit.commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![selected],
    };
    let refused = runtime
        .index_authority()
        .refresh_field_indexes_for_basis(
            changed_request.clone(),
            &changed_basis,
            None,
            budget(),
            execution,
        )
        .unwrap_err();
    assert!(matches!(
        refused.kind,
        crate::indexes::data::DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
    ));
    assert!(runtime
        .index_access()
        .published_generation_for_observation(selected, &changed_basis.observation())
        .is_none());
    let denied = runtime
        .index_authority()
        .refresh_field_indexes_for_basis(
            changed_request.clone(),
            &changed_basis,
            Some(&predecessor),
            refused_budget,
            execution,
        )
        .unwrap_err();
    assert!(matches!(
        denied.kind,
        crate::indexes::data::DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
    ));
    assert!(runtime
        .index_access()
        .published_generation_for_observation(selected, &changed_basis.observation())
        .is_none());
    let unfunded = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(0),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let memory_denied = runtime
        .index_authority()
        .refresh_field_indexes_for_basis(
            changed_request.clone(),
            &changed_basis,
            Some(&predecessor),
            budget(),
            &worth_execution::ExecutionRequest::serial(&unfunded),
        )
        .unwrap_err();
    assert!(matches!(
        memory_denied.kind,
        crate::indexes::data::DerivedIndexMaintenanceDenialKind::Execution(
            crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
                | crate::execution::RelationalExecutionDenialCause::DeclaredMemoryExhausted { .. }
                | crate::execution::RelationalExecutionDenialCause::ProcessMemoryExhausted { .. }
        )
    ));
    assert!(runtime
        .index_access()
        .published_generation_for_observation(selected, &changed_basis.observation())
        .is_none());
    let maintained = runtime
        .index_authority()
        .refresh_field_indexes_for_basis(
            changed_request,
            &changed_basis,
            Some(&predecessor),
            budget(),
            execution,
        )
        .unwrap();
    assert!(maintained.work.patch_records > 0);
    assert_eq!(maintained.work.cold_record_slots, 0);
    assert!(runtime
        .index_access()
        .published_generation_for_observation(selected, &changed_basis.observation())
        .is_some());
    assert!(runtime
        .index_access()
        .published_generation_for_observation(unrelated.index_id, &changed_basis.observation())
        .is_none());
    runtime.snapshots().release_snapshot(&predecessor).unwrap();
    release_test_commit_snapshot(&runtime, &changed);
}
