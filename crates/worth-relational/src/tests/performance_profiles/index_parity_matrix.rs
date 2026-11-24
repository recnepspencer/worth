use super::*;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_index_parity_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let warm_generation_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema();
        let alpha = create_entity_outcome(execution, &runtime, "alpha");
        let _beta = create_entity_outcome(execution, &runtime, "beta");
        let index = runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "entity.name.lookup".to_string(),
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
                branch_scoped: false,
            })
            .expect("index fixture has identity capacity");

        let (build, build_work) = measure_counted_work(execution, || {
            runtime.index_authority().build_for_commit(
                execution,
                DerivedIndexBuildRequest {
                    source_commit_id: alpha.commit.commit_id,
                    branch_id: BranchId("main".to_string()),
                    index_ids: vec![index.index_id],
                },
            )
        });
        assert!(build.failed_indexes.is_empty());

        runtime.performance_access().reset_counters();
        let (outcome, query_work) = measure_counted_work(execution, || {
            runtime
                .index_access()
                .execute_query_plan_with_index_parity(
                    runtime
                        .read_truth()
                        .plan_query_packet(
                            &alpha.snapshot,
                            entity_name_index_packet(
                                &runtime,
                                &alpha.snapshot,
                                "entity-name-equals-warm",
                                "alpha",
                            ),
                        )
                        .expect("warm entity query plan"),
                    IndexParityMode::CertificationParity,
                    execution,
                )
                .expect("request remains live")
                .expect("warm entity index query outcome")
        });

        CountedPerfSample {
            charged_work: build_work + query_work,
            metrics: perf_metrics!({
                "build_work": build_work,
                "query_work": query_work,
                "entity_result_count": outcome.execution.result.entities.len(),
                "relation_result_count": outcome.execution.result.relations.len(),
                "access_path": format!("{:?}", outcome.access_path),
                "parity_digest_present": !outcome.parity_basis_digest.is_empty(),
                "counters": runtime.performance_access().counters(),
            }),
        }
    });
    assert_counted_budget(
        &warm_generation_samples,
        "warm entity index generations should stay on the derived path with certification parity",
        |metrics| {
            metric_u64(metrics, "entity_result_count") == 1
                && metric_u64(metrics, "relation_result_count") == 0
                && metrics["parity_digest_present"].as_bool() == Some(true)
                && metrics["access_path"]
                    .as_str()
                    .unwrap_or("")
                    .contains("DerivedIndexGeneration")
                && counter_u64(metrics, "query_index_attempt_count") == 1
                && counter_u64(metrics, "query_index_path_count") == 1
                && counter_u64(metrics, "query_index_parity_verification_count") == 1
                && counter_u64(metrics, "query_index_rejection_count") == 0
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );

    let build_failed_samples = capture_counted_samples(|| {
        let runtime = runtime_with_test_schema();
        let alpha = create_entity_outcome(execution, &runtime, "alpha");
        let index = runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "entity.name.lookup".to_string(),
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
                branch_scoped: false,
            })
            .expect("index fixture has identity capacity");
        let build = runtime.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: alpha.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                index_ids: vec![index.index_id],
            },
        );
        assert!(build.failed_indexes.is_empty());
        runtime
            .indexes
            .corrupt_latest_generation_for_test(index.index_id, |generation| {
                generation.status =
                    crate::facade::indexes::DerivedIndexPublicationStatus::BuildFailed;
            });

        runtime.performance_access().reset_counters();
        let (outcome, query_work) = measure_counted_work(execution, || {
            runtime
                .index_access()
                .execute_query_plan_with_index_parity(
                    runtime
                        .read_truth()
                        .plan_query_packet(
                            &alpha.snapshot,
                            entity_name_index_packet(
                                &runtime,
                                &alpha.snapshot,
                                "entity-name-equals-build-failed",
                                "alpha",
                            ),
                        )
                        .expect("storage-read entity query plan"),
                    IndexParityMode::ProductionAdmissibility,
                    execution,
                )
                .expect("request remains live")
                .expect("storage-read entity index query outcome")
        });

        CountedPerfSample {
            charged_work: query_work,
            metrics: perf_metrics!({
                "query_work": query_work,
                "entity_result_count": outcome.execution.result.entities.len(),
                "access_path": format!("{:?}", outcome.access_path),
                "counters": runtime.performance_access().counters(),
            }),
        }
    });
    assert_counted_budget(
        &build_failed_samples,
        "build-failed generations should reject to storage read without changing truth",
        |metrics| {
            metric_u64(metrics, "entity_result_count") == 1
                && metrics["access_path"]
                    .as_str()
                    .unwrap_or("")
                    .contains("DerivedIndexRejectedStorageRead")
                && metrics["access_path"]
                    .as_str()
                    .unwrap_or("")
                    .contains("CorruptIndexEntries")
                && counter_u64(metrics, "query_index_attempt_count") == 1
                && counter_u64(metrics, "query_index_path_count") == 0
                && counter_u64(metrics, "query_index_rejection_count") == 1
                && counter_u64(metrics, "query_index_parity_verification_count") == 0
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );

    let persisted_recovery_samples = capture_counted_samples(|| {
        let medium = OwnedTestDurableMedium::new();
        let runtime = durable_runtime_for_medium(&medium);
        let alpha = create_entity_outcome(execution, &runtime, "alpha");
        let index = runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "entity.name.lookup".to_string(),
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
                },
                branch_scoped: false,
            })
            .expect("index fixture has identity capacity");
        let build = runtime.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: alpha.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                index_ids: vec![index.index_id],
            },
        );
        assert!(build.failed_indexes.is_empty());

        let original = runtime
            .index_access()
            .execute_query_plan_with_index_parity(
                runtime
                    .read_truth()
                    .plan_query_packet(
                        &alpha.snapshot,
                        entity_name_index_packet(
                            &runtime,
                            &alpha.snapshot,
                            "entity-name-equals-persisted",
                            "alpha",
                        ),
                    )
                    .expect("original persisted plan"),
                IndexParityMode::CertificationParity,
                execution,
            )
            .expect("request remains live")
            .expect("original persisted query outcome");

        let (recovered, recover_work) = measure_counted_work(execution, || {
            capture_and_file_checkpoint(execution, &runtime, &medium);
            assert!(medium.has_filed_checkpoint());
            assert_eq!(medium.committed_count(), 1);
            let mut recovered = durable_runtime_for_medium(&medium);
            recover_runtime_from_medium(execution, &mut recovered, &medium);
            let persisted_generation = recovered
                .index_access()
                .read_latest_generation(execution, index.index_id, &BranchId("main".to_string()))
                .expect("cold checkpoint index generation read succeeds")
                .expect("persisted derived index generation survives durable checkpoint recovery");
            assert_eq!(
                persisted_generation.generation_id,
                build.generations[0].generation_id
            );
            recovered
        });
        let recovered_snapshot = recovered
            .visibility_authority()
            .snapshot()
            .expect("current snapshot fixture is admitted");

        recovered.performance_access().reset_counters();
        let (recovered_outcome, query_work) = measure_counted_work(execution, || {
            recovered
                .index_access()
                .execute_query_plan_with_index_parity(
                    recovered
                        .read_truth()
                        .plan_query_packet(
                            &recovered_snapshot,
                            entity_name_index_packet(
                                &recovered,
                                &recovered_snapshot,
                                "entity-name-equals-recovered",
                                "alpha",
                            ),
                        )
                        .expect("recovered persisted plan"),
                    IndexParityMode::CertificationParity,
                    execution,
                )
                .expect("request remains live")
                .expect("recovered persisted query outcome")
        });

        CountedPerfSample {
            charged_work: recover_work + query_work,
            metrics: perf_metrics!({
                "recover_work": recover_work,
                "query_work": query_work,
                "entity_result_count": recovered_outcome.execution.result.entities.len(),
                "access_path": format!("{:?}", recovered_outcome.access_path),
                "result_digest_match": original.execution.result.reduction_digest
                    == recovered_outcome.execution.result.reduction_digest,
                "parity_digest_match": original.parity_basis_digest
                    == recovered_outcome.parity_basis_digest,
                "counters": recovered.performance_access().counters(),
            }),
        }
    });
    assert_counted_budget(
        &persisted_recovery_samples,
        "persisted recovery should preserve derived index access and parity digests",
        |metrics| {
            metric_u64(metrics, "entity_result_count") == 1
                && metrics["result_digest_match"].as_bool() == Some(true)
                && metrics["parity_digest_match"].as_bool() == Some(true)
                && metrics["access_path"]
                    .as_str()
                    .unwrap_or("")
                    .contains("DerivedIndexGeneration")
                && counter_u64(metrics, "query_index_attempt_count") == 1
                && counter_u64(metrics, "query_index_path_count") == 1
                && counter_u64(metrics, "query_index_parity_verification_count") == 1
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );
}
