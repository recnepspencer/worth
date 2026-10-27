//! Exact selected generations seed forks and maintain actual predecessor patches.
use super::*;

#[test]
fn request_paid_selected_field_refresh_leaves_unrelated_cold_index_unpublished() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (runtime, basis, request) = fixture(execution);
    let unrelated = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "unrelated-cold-field".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");
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
        .read_published_generation_for_observation(execution, selected, &observation)
        .unwrap()
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
        .read_published_generation_for_observation(execution, selected, &observation)
        .unwrap()
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
    let forked = runtime
        .fork_branch(execution, fork.clone(), fork_source)
        .unwrap();
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
        .read_published_generation_for_observation(execution, selected, &observation)
        .unwrap()
        .is_some());
    assert!(runtime
        .index_access()
        .read_published_generation_for_observation(execution, unrelated.index_id, &observation)
        .unwrap()
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
        .read_published_generation_for_observation(
            execution,
            selected,
            &changed_basis.observation()
        )
        .unwrap()
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
        .read_published_generation_for_observation(
            execution,
            selected,
            &changed_basis.observation()
        )
        .unwrap()
        .is_none());
    let unfunded = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(0)),
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
        .read_published_generation_for_observation(
            execution,
            selected,
            &changed_basis.observation()
        )
        .unwrap()
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
        .read_published_generation_for_observation(
            execution,
            selected,
            &changed_basis.observation()
        )
        .unwrap()
        .is_some());
    assert!(runtime
        .index_access()
        .read_published_generation_for_observation(
            execution,
            unrelated.index_id,
            &changed_basis.observation()
        )
        .unwrap()
        .is_none());
    runtime.snapshots().release_snapshot(&predecessor).unwrap();
    release_test_commit_snapshot(&runtime, &changed);
}
