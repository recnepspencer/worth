use super::*;
mod certification_memory;
mod population;
use crate::facade::indexes::{
    BoundedEntityFieldLookupAdmissionStop as Stop, BoundedEntityFieldLookupDenialKind,
    BoundedIndexParityMode,
};

#[test]
fn prepared_field_selection_keeps_its_root_without_reselecting_the_generation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(execution, &runtime, "pinned-name");
    let entity = changed_entities(&created)[0];
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.prepared".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: true,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());
    let view = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    let before = runtime.index_access().generation_selection_counters();
    let prepared = runtime
        .index_access()
        .prepare_entity_field_lookup(&view, index.index_id, KindId(1), &locator, execution)
        .unwrap();
    let foreign = runtime_with_index_field_aspects();
    let denied = foreign
        .index_access()
        .prepare_entity_field_lookup(&view, index.index_id, KindId(1), &locator, execution)
        .unwrap_err();
    assert_eq!(
        denied.kind(),
        BoundedEntityFieldLookupDenialKind::SnapshotUnavailable,
        "foreign view fails before consulting the other runtime's index catalog"
    );
    assert_eq!(
        runtime
            .index_access()
            .generation_selection_counters()
            .generation_payload_reads
            - before.generation_payload_reads,
        1
    );
    let discarded = runtime
        .index_authority()
        .discard_generations(
            crate::facade::indexes::DerivedIndexDiscardRequest::all_bases(index.index_id),
        )
        .unwrap();
    assert_eq!(discarded.removed_generation_count(), 1);
    let cold = runtime
        .index_access()
        .prepare_entity_field_lookup(&view, index.index_id, KindId(1), &locator, execution)
        .unwrap_err();
    assert_eq!(
        cold.kind(),
        BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
    );
    drop(view);
    runtime
        .visibility_authority()
        .release_snapshot(&created.snapshot)
        .unwrap();
    update_entity(execution, &runtime, entity, "new-name");
    let before = runtime.index_access().generation_selection_counters();
    let handles = runtime.visibility.snapshot_handle_registry_cost_counters();
    for (name, expected) in [
        ("pinned-name", vec![entity]),
        ("new-name", vec![]),
        ("absent", vec![]),
    ] {
        let selected = runtime
            .index_access()
            .execute_prepared_entity_field_lookup(
                &prepared,
                &string_aspect_value(name),
                1024,
                BoundedIndexParityMode::Production,
                execution,
                || Ok::<_, ()>(()),
            )
            .unwrap();
        assert_eq!(selected.candidate_entity_ids(), expected);
        assert_eq!(selected.generation_id(), build.generations[0].generation_id);
        assert!(!selected.overflowed());
    }
    assert_eq!(
        runtime.index_access().generation_selection_counters(),
        before
    );
    assert_eq!(
        runtime.visibility.snapshot_handle_registry_cost_counters(),
        handles
    );
    let certified = runtime
        .index_access()
        .execute_prepared_entity_field_lookup(
            &prepared,
            &string_aspect_value("pinned-name"),
            1024,
            BoundedIndexParityMode::Certification,
            execution,
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(certified.candidate_entity_ids(), &[entity]);

    let mut contacted = false;
    let denied = foreign
        .index_access()
        .execute_prepared_entity_field_lookup(
            &prepared,
            &string_aspect_value("pinned-name"),
            1024,
            BoundedIndexParityMode::Production,
            execution,
            || {
                contacted = true;
                Ok::<_, ()>(())
            },
        )
        .unwrap_err();
    assert!(matches!(denied, Stop::ExactBasisRequired));
    assert!(!contacted, "foreign affinity fails before caller work");
    let stopped = runtime
        .index_access()
        .execute_prepared_entity_field_lookup(
            &prepared,
            &string_aspect_value("pinned-name"),
            1024,
            BoundedIndexParityMode::Production,
            execution,
            || Err("cancelled"),
        )
        .unwrap_err();
    assert!(matches!(stopped, Stop::Admission("cancelled")));
}

#[test]
fn request_cancellation_precedes_the_callers_exhausted_admission() {
    let cancellation = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "cancellation-order");
    let first_entity = changed_entities(&first)[0];
    release_test_commit_snapshot(&runtime, &first);
    let created = create_entity_outcome(execution, &runtime, "cancellation-order");
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "entity.name.cancellation-order".into(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: true,
    });
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());
    let view = runtime
        .read_truth()
        .project_snapshot(&created.snapshot)
        .unwrap();
    let prepared = runtime
        .index_access()
        .prepare_entity_field_lookup(&view, index.index_id, KindId(1), &locator, execution)
        .unwrap();
    let mut failures = Vec::new();
    for parity in [
        BoundedIndexParityMode::Production,
        BoundedIndexParityMode::Certification,
    ] {
        let baseline = runtime
            .index_access()
            .execute_prepared_entity_field_lookup(
                &prepared,
                &string_aspect_value("cancellation-order"),
                16,
                parity,
                execution,
                || Ok::<_, &str>(()),
            )
            .unwrap();
        assert_eq!(
            baseline.candidate_entity_ids(),
            &[first_entity, changed_entities(&created)[0]]
        );
        let cancellation = worth_execution::CancellationSource::new();
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            cancellation.token(),
            None,
        );
        let request = worth_execution::ExecutionRequest::serial(&host);
        cancellation.cancel();
        let mut calls = 0;
        let stopped = runtime.index_access().execute_prepared_entity_field_lookup(
            &prepared,
            &string_aspect_value("cancellation-order"),
            16,
            parity,
            &request,
            || {
                calls += 1;
                Err::<(), _>("admission exhausted")
            },
        );
        assert!(
            matches!(stopped, Err(Stop::Lookup(denial)) if denial.kind() ==
            BoundedEntityFieldLookupDenialKind::Execution(crate::execution::RelationalExecutionDenialCause::Cancelled))
        );
        assert_eq!(calls, 0, "pre-cancelled {parity:?} admission");
        // Admissions one and two guard entry and the encoded key, three the
        // result reservation, and four/five the two index candidates. Seven
        // guards Certification's final admission after its storage candidates.
        let rendezvous: &[usize] = if parity == BoundedIndexParityMode::Production {
            &[1, 2, 3, 4, 5]
        } else {
            &[1, 2, 3, 4, 5, 7]
        };
        for &after in rendezvous {
            for exhaust_work in [false, true] {
                let cancellation = worth_execution::CancellationSource::new();
                let host = worth_execution::SerialRequest::from_memory(
                    worth_execution::SerialMemoryBudget::new(None),
                    cancellation.token(),
                    None,
                );
                let execution = worth_execution::ExecutionRequest::serial(&host);
                let mut admission_calls = 0;
                let identity = worth_foundational::PartitionIdentity::new(1);
                let scan = worth_execution::ExecutionScan::try_from_ordered(
                    vec![identity],
                    vec![(identity, ())],
                )
                .unwrap();
                let mut stopped = None;
                let (scan_outcome, _) = execution
                    .run(worth_execution::ExecutionWorkCeiling::new(Some(2 * crate::visibility::materialization::read_records::read_prices::KEYED_RECORD)), |lease| {
                        scan.run::<(), (), (), _>(lease, (), 0, 0, 0, 0, |_, _, work| {
                            stopped =
                                Some(runtime.index_access().execute_prepared_entity_field_lookup(
                                    &prepared,
                                    &string_aspect_value("cancellation-order"),
                                    16,
                                    parity,
                                    &execution,
                                    || {
                                        admission_calls += 1;
                                        if admission_calls > after {
                                            return Err("reader callback interrupted");
                                        }
                                        if admission_calls == after {
                                            if exhaust_work {
                                                assert_eq!(
                                                    work.checkpoint(work.remaining_work().expect("this kernel has an explicit finite ceiling") + 1),
                                                    Err(
                                                        worth_execution::MapKernelStop::WorkCeiling
                                                    )
                                                );
                                            } else {
                                                cancellation.cancel();
                                            }
                                        }
                                        Ok(())
                                    },
                                ));
                            Ok(((), ()))
                        })
                    })
                    .unwrap();
                let stopped = stopped.expect("the metered kernel ran the lookup");
                if exhaust_work {
                    // This checkpoint belongs to the outer computation. During
                    // Certification, the reader is inside another native scope;
                    // its interruption result is not the outer stop's authority.
                    assert!(admission_calls >= after, "the work refusal was injected");
                    assert!(
                        stopped.is_err(),
                        "an interrupted read cannot escape a value"
                    );
                    assert!(
                        matches!(
                            scan_outcome,
                            worth_execution::ScanOutcome::Stopped {
                                completed_prefix,
                                boundary: Some(boundary),
                                reason: worth_execution::MapStop::WorkExhausted {
                                    identity: stopped_identity,
                                },
                                ..
                            } if completed_prefix.is_empty()
                                && boundary == identity && stopped_identity == identity
                        ),
                        "{parity:?}, after={after}: the owning scan must retain its work stop"
                    );
                } else {
                    let expected = BoundedEntityFieldLookupDenialKind::Execution(
                        crate::execution::RelationalExecutionDenialCause::Cancelled,
                    );
                    let actual = match stopped {
                        Err(Stop::Lookup(denial)) => Some(denial.kind()),
                        _ => None,
                    };
                    if actual != Some(expected) || admission_calls != after {
                        failures.push(format!("{parity:?}, after={after}: cause={actual:?}, admissions={admission_calls}, expected={expected:?}"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
