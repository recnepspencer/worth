use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::index_refresh::commit_empty;

use worth_foundational::facade::AspectKey;
use worth_foundational::facade::FieldKey;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;
use worth_relational::facade::indexes::DerivedIndexDefinition;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::indexes::DerivedIndexKind;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_world;
#[test]
fn bootstrap_seeds_exact_workflow_transition_lookup_before_any_settlement() {
    use worth_foundational::facade::AspectValue;
    use worth_foundational::facade::InternedString;
    use worth_relational::facade::indexes::BoundedEntityFieldLookupRequest;
    use worth_relational::facade::indexes::BoundedIndexParityMode;
    let world = installed_world(&[(
        "workflow-transition-seed",
        WorthQueryPrincipalMappingStatus::Enabled,
    )]);
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let transition = &(*(*graph.layout_for_test()).workflow().transition_for_test());
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host_request);
    handle.with_runtime_mut_for_test(&execution, |runtime| {
        let basis = runtime.admit_branch_basis(&runtime.main_branch_identity()).unwrap();
        assert!(runtime.index_access().read_published_generation_for_observation(&execution,
            (*transition.identity_index_id_for_test()), &basis.observation(),
        ).unwrap().is_some(), "bootstrap must seed the exact installed transition index");
        let snapshot = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(runtime, &basis).unwrap();
        let request = BoundedEntityFieldLookupRequest::new(
            snapshot.clone(), (*transition.identity_index_id_for_test()), (*transition.entity_kind_for_test()),
            (*transition.identity_for_test()).clone(), AspectValue::String(InternedString::Raw("unsettled-operation".to_owned())), 2,
        ).unwrap();
        let outcome = runtime.index_access().execute_bounded_entity_field_lookup(
            request, BoundedIndexParityMode::Production, &execution,
        ).expect("an absent settlement is a lawful empty lookup, not an unavailable generation");
        assert!(outcome.candidate_entity_ids().is_empty());
        assert!(!outcome.overflowed());
        worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
    });
}
#[test]
fn mutation_refreshes_every_exact_performed_commit_across_branches() {
    let world = installed_world(&[("index-refresh", WorthQueryPrincipalMappingStatus::Enabled)]);
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .expect("fixture publishes its primary graph");
    let handle = graph.integration_handle_for_test();
    let index_ids = (*handle.primary_index_ids_for_test()).to_vec();

    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (feature_commit, main_commit) = handle
        .execute_mutation_with_index_refresh(execution, |runtime| {
            let (_, source) = runtime
                .observe_fork_source(&BranchId("main".to_owned()))
                .expect("main exposes an exact fork source");
            runtime
                .fork_branch(
                    execution,
                    BranchId("index-refresh-feature".to_owned()),
                    source,
                )
                .expect("feature branch forks from main");
            let feature = commit_empty(execution, runtime, "index-refresh-feature");
            let main = commit_empty(execution, runtime, "main");
            assert!(index_ids.iter().any(|index_id| {
                runtime
                    .index_access()
                    .read_published_generation_for_commit(execution, *index_id, &feature)
                    .unwrap()
                    .is_none()
            }));
            Ok::<_, &'static str>((feature, main))
        })
        .expect("exact publication delta refresh succeeds")
        .expect("mutation succeeds");

    handle.with_runtime(execution, |runtime| {
        for committed in [&feature_commit, &main_commit] {
            for index_id in &index_ids {
                assert!(
                    runtime
                        .index_access()
                        .read_published_generation_for_commit(execution, *index_id, committed)
                        .unwrap()
                        .is_some(),
                    "index {index_id:?} must be current for exact commit {:?} on branch {:?}",
                    committed.commit_id,
                    committed.branch_id,
                );
            }
        }
    });
}
#[test]
fn multi_commit_callback_reconstructs_each_retained_generation_with_cold_budget() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world(&[(
        "index-refresh-cold",
        WorthQueryPrincipalMappingStatus::Enabled,
    )]);
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .expect("published graph");
    let handle = graph.integration_handle_for_test();
    let index_ids = (*handle.primary_index_ids_for_test()).to_vec();
    let commits = handle
        .execute_mutation_with_index_refresh(execution, |runtime| {
            Ok::<_, &'static str>([
                commit_empty(execution, runtime, "main"),
                commit_empty(execution, runtime, "main"),
            ])
        })
        .expect("bounded reconstruction succeeds")
        .expect("both commits succeed");
    handle.with_runtime(execution, |runtime| {
        for commit in &commits {
            for index_id in &index_ids {
                assert!(runtime
                    .index_access()
                    .read_published_generation_for_commit(execution, *index_id, commit)
                    .unwrap()
                    .is_some());
            }
        }
    });
}
#[test]
fn captured_before_root_with_missing_generation_uses_explicit_cold_fallback() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world(&[(
        "index-refresh-missing",
        WorthQueryPrincipalMappingStatus::Enabled,
    )]);
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .expect("published graph");
    let handle = graph.integration_handle_for_test();
    (*handle.source_owner_for_test()).with_runtime_mut(execution, |runtime| {
        let identity = runtime.main_branch_identity();
        let (_, before_basis) = runtime.observe_branch(&identity).unwrap();
        let before = runtime
            .snapshots()
            .snapshot_for_observation(&before_basis.observation())
            .unwrap();
        let index = runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "missing-prior-generation".into(),
                kind: DerivedIndexKind::EntityField {
                    field_locator:
                        worth_relational::facade::transactions::planned_single_field_locator(
                            AspectKey::new("missing-prior").unwrap(),
                            FieldKey::new("value").unwrap(),
                        ),
                },
                branch_scoped: true,
            })
            .expect("index fixture has identity capacity");
        let committed = commit_empty(execution, runtime, "main");
        let (_, basis) = runtime.observe_branch(&identity).unwrap();
        let refreshed = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::index_maintenance_budget::refresh_with_cold_fallback(
            execution,
            runtime,
            DerivedIndexBuildRequest {
                source_commit_id: committed.commit_id,
                branch_id: committed.branch_id.clone(),
                index_ids: vec![index.index_id],
            },
            &basis,
            Some(&before),
        )
        .unwrap();
        assert_eq!(refreshed.generations.len(), 1);
        runtime.snapshots().release_snapshot(&before).unwrap();
    });
}
