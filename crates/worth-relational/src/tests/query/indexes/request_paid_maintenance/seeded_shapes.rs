use super::*;

#[test]
fn sixteen_seeded_shapes_preserve_generations_and_answers_and_charge_native_units() {
    prove_shapes((0..16).map(|seed| (seed, 1 + (seed * 5) % 7)));
}

#[test]
fn two_two_hundred_row_shapes_preserve_answers_on_every_maintenance_path() {
    prove_shapes([(0, 200), (1, 200)]);
}

fn prove_shapes(shapes: impl IntoIterator<Item = (usize, usize)>) {
    let mut failures = Vec::new();
    for (seed, rows) in shapes {
        let host = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host);
        let fields = 1 + seed % 3;
        let mut schema = AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        );
        for field in 1..fields {
            let key = format!("field{field}");
            schema
                .entity_aspects
                .push(entity_field_aspect(aspect_key(&key), field_key(&key)));
        }
        let runtime = schema.build_runtime();
        let mut last = None;
        for row in 0..rows {
            let value = format!("row{row}");
            let patches = (0..fields)
                .map(|field| {
                    let key = if field == 0 {
                        "name".to_string()
                    } else {
                        format!("field{field}")
                    };
                    (
                        aspect_field_locator(aspect_key(&key), field_key(&key)),
                        string_aspect_value(&value),
                    )
                })
                .collect();
            let batch = WorkerIntentBatch::new("seeded").push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw(&value),
                    fields: AspectFieldPatch::new(patches),
                }),
            ));
            let mut transaction = test_owner_begin_transaction_for_main(&runtime);
            transaction
                .push_batch(
                    batch,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let created = transaction.commit(execution, &runtime).unwrap();
            if let Some(previous) = last.replace(created) {
                release_test_commit_snapshot(&runtime, &previous);
            }
        }
        let created = last.unwrap();
        let indexes: Vec<_> = (0..fields)
            .map(|field| {
                let key = if field == 0 {
                    "name".to_string()
                } else {
                    format!("field{field}")
                };
                runtime
                    .index_authority()
                    .register(DerivedIndexDefinition {
                        index_id: DerivedIndexId(0),
                        name: key.clone(),
                        branch_scoped: true,
                        kind: DerivedIndexKind::EntityField {
                            field_locator: aspect_field_locator(aspect_key(&key), field_key(&key)),
                        },
                    })
                    .index_id
            })
            .collect();
        let request = DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: indexes.clone(),
        };
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let before = runtime
            .index_authority()
            .build_for_basis(execution, request.clone(), &basis);
        assert!(before.failed_indexes.is_empty());
        for (field, index) in indexes.iter().enumerate() {
            if (seed >> field) & 1 == 0 {
                runtime
                    .index_authority()
                    .discard_generations(
                        crate::indexes::data::DerivedIndexDiscardRequest::all_bases(*index),
                    )
                    .unwrap();
            }
        }
        let (paid, report) = execution
            .run(ExecutionWorkCeiling::new(100_000), |_| {
                runtime.index_authority().refresh_for_basis(
                    request.clone(),
                    &basis,
                    None,
                    cold_budget(),
                    execution,
                )
            })
            .unwrap();
        let paid = paid.unwrap();
        let cold = (0..fields).filter(|field| (seed >> field) & 1 == 0).count();
        // Unique buckets reserve one initial map path, 64 units per later
        // map path and one row path per insertion, independently per field.
        // For these sizes the independent path bound has one level for
        // 0..=64 keys and two for 65..=199; the first empty map costs one.
        // Every unique bucket also reserves one empty row path.
        assert!(rows <= 200);
        let map_paths = 1 + 64 * (rows - 1).min(64) + 128 * rows.saturating_sub(65);
        let expected = if cold == 0 {
            fields
        } else {
            fields + (rows + 1) + 2 * rows + cold * rows + cold * (map_paths + rows)
        };
        assert_eq!(paid.work.work_units, expected, "seed {seed}");

        let mut next_generation = before
            .generations
            .iter()
            .map(|generation| generation.generation_id.0)
            .max()
            .unwrap();
        for (field, (old, new)) in before.generations.iter().zip(&paid.generations).enumerate() {
            if (seed >> field) & 1 == 0 {
                next_generation += 1;
                assert_eq!(new.generation_id.0, next_generation);
            } else {
                assert_eq!(new.generation_id, old.generation_id);
            }
            assert_eq!(old.index_id, new.index_id);
            assert_eq!(old.source_commit_id, new.source_commit_id);
            assert_eq!(old.applicability, new.applicability);
            assert_eq!(old.source_branch_id, new.source_branch_id);
            assert_eq!(old.status, new.status);
            assert_eq!(old.entries, new.entries);
            let crate::indexes::data::DerivedIndexEntries::EntityField(entries) = &new.entries
            else {
                panic!("entity field")
            };
            for row in 0..rows {
                let key = crate::storage::data::AuthoritativeFieldComparisonKey::from_aspect_value(
                    &string_aspect_value(&format!("row{row}")),
                );
                assert_eq!(entries.get(&key).unwrap().len(), 1);
            }
        }
        let view = runtime
            .read_truth()
            .project_snapshot(&created.snapshot)
            .unwrap();
        for (field, generation) in paid.generations.iter().enumerate() {
            let key = if field == 0 {
                "name".to_string()
            } else {
                format!("field{field}")
            };
            let prepared = runtime
                .index_access()
                .prepare_entity_field_lookup(
                    &view,
                    generation.index_id,
                    KindId(1),
                    &aspect_field_locator(aspect_key(&key), field_key(&key)),
                    execution,
                )
                .unwrap();
            for row in 0..rows {
                let value = string_aspect_value(&format!("row{row}"));
                let answer = runtime
                    .index_access()
                    .execute_prepared_entity_field_lookup(
                        &prepared,
                        &value,
                        rows,
                        crate::indexes::data::BoundedIndexParityMode::Production,
                        execution,
                        || Ok::<_, ()>(()),
                    )
                    .unwrap();
                let crate::indexes::data::DerivedIndexEntries::EntityField(old) =
                    &before.generations[field].entries
                else {
                    panic!("field")
                };
                let expected = old
                    .get(
                        &crate::storage::data::AuthoritativeFieldComparisonKey::from_aspect_value(
                            &value,
                        ),
                    )
                    .unwrap()
                    .iter()
                    .copied()
                    .collect::<Vec<_>>();
                assert_eq!(answer.candidate_entity_ids(), expected);
            }
        }
        println!(
            "seed={seed} rows={rows} fields={fields} native={} request={} counts={:?}",
            paid.work.work_units,
            report.charged_work(),
            paid.work
        );
        if report.charged_work() != paid.work.work_units as u64 {
            failures.push(seed);
        }
        prove_commit_paths(
            execution,
            &runtime,
            &created,
            &request,
            seed,
            expected,
            &before.generations,
        );
        release_test_commit_snapshot(&runtime, &created);
    }
    assert!(failures.is_empty(), "unpaid seeds: {failures:?}");
}

fn prove_commit_paths(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    created: &crate::facade::transactions::CommitResult,
    request: &DerivedIndexBuildRequest,
    seed: usize,
    expected: usize,
    baseline: &[crate::indexes::data::DerivedIndexGeneration],
) {
    for (field, index) in request.index_ids.iter().enumerate() {
        if (seed >> field) & 1 == 0 {
            runtime
                .index_authority()
                .discard_generations(crate::indexes::data::DerivedIndexDiscardRequest::all_bases(
                    *index,
                ))
                .unwrap();
        }
    }
    let transaction = test_owner_begin_transaction_for_main(runtime);
    let mut candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let (prepared, report) = execution
        .run(ExecutionWorkCeiling::new(100_000), |_| {
            runtime.index_authority().prepare_for_candidate(
                &mut candidate,
                &request.index_ids,
                Some(&created.snapshot),
                cold_budget(),
                execution,
            )
        })
        .unwrap();
    let prepared = prepared.unwrap();
    assert_eq!(
        prepared.work_units,
        expected + request.index_ids.len(),
        "candidate seed {seed}"
    );
    assert_eq!(report.charged_work(), prepared.work_units as u64);
    assert_eq!(
        prepared.generation_publications_reserved,
        request.index_ids.len()
    );
    assert_eq!(prepared.patch_records, 0);
    assert!(request.index_ids.iter().all(|index| runtime
        .index_access()
        .latest_generation(*index, &request.branch_id)
        .is_none_or(|generation| generation.source_commit_id == created.commit.commit_id)));
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("prepared differential candidate publishes")
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    for (index, old) in request.index_ids.iter().zip(baseline) {
        let new = runtime
            .index_access()
            .latest_generation(*index, &request.branch_id)
            .unwrap();
        assert_eq!(new.entries, old.entries, "candidate seed {seed}");
        assert_eq!(new.source_commit_id, committed.commit.commit_id);
    }
    release_test_commit_snapshot(runtime, &committed);
    // Also exercise the retained-commit reader while the source snapshot owns
    // that old root. Discard only the same seed's cold indexes.
    for (field, index) in request.index_ids.iter().enumerate() {
        if (seed >> field) & 1 == 0 {
            runtime
                .index_authority()
                .discard_generations(crate::indexes::data::DerivedIndexDiscardRequest::all_bases(
                    *index,
                ))
                .unwrap();
        }
    }
    let (reconstructed, report) = execution
        .run(ExecutionWorkCeiling::new(100_000), |_| {
            runtime.index_authority().reconstruct_for_commit(
                execution,
                request.clone(),
                cold_budget(),
            )
        })
        .unwrap();
    let reconstructed = reconstructed.unwrap();
    assert_eq!(
        reconstructed.work.work_units, expected,
        "retained seed {seed}"
    );
    assert_eq!(report.charged_work(), reconstructed.work.work_units as u64);
    for (new, old) in reconstructed.generations.iter().zip(baseline) {
        assert_eq!(new.entries, old.entries, "retained seed {seed}");
        assert_eq!(new.applicability, old.applicability);
        assert_eq!(new.source_commit_id, old.source_commit_id);
    }
}
