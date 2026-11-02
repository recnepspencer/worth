//! Literal authored paths convict flattened endpoints and stale nested discovery.
use super::super::history::basis;
use super::fixture::*;
use super::*;

#[test]
fn nested_projection_routes_empty_entrants_scalar_edits_leaf_removal_and_rewiring() {
    run(|runtime, _program, court, host, _cancel| {
        let execution = ExecutionRequest::serial(host);
        let n = seed(runtime, execution);
        expect(court, &n, parents(&n));
        assert_eq!(
            court.nested.lock().unwrap().get(&(n.d, n.w)),
            Some(&vec![record(&[n.dw], n.d, "D")])
        );
        clear(court);
        let (az, bz) = observe_links(runtime, execution, &n);
        let mut expected = parents(&n);
        for (parent, observes) in [(n.ax, az), (n.bx, bz)] {
            expected.push(record(&[parent, observes], n.z, "z"));
            expected.push(record(&[parent, observes, n.cz], n.c, "C"));
        }
        expect(court, &n, expected.clone());
        assert!(
            court.nested_checks.load(Ordering::SeqCst) > 0,
            "convergent parent occurrences at the SAME descendant reached the graft refusal"
        );
        clear(court);
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("nested-indirect-scalar").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: n.z,
                    fields: fixture::field("changed"),
                }),
            )),
        );
        let mut committed_expected = expected.clone();
        committed_expected.sort();
        for scope in [n.a, n.b] {
            assert_eq!(
                court.nested_before.lock().unwrap().get(&(scope, n.x)),
                Some(&committed_expected),
                "committed Native nested fields retain the real before image"
            );
        }
        for row in &mut expected {
            if row.entity == n.z {
                row.value = Some("changed".into());
            }
        }
        expect(court, &n, expected.clone());
        assert!(
            !court.staged.lock().unwrap().contains(&n.d),
            "unrelated real scope is not scheduled"
        );
        clear(court);
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("remove-nested-leaf").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: n.cz }),
            )),
        );
        expected.retain(|row| row.path.len() < 3);
        expect(court, &n, expected);
        clear(court);
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("rewire-nested-inner").push(MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                    relation_id: az,
                    kind_id: KindId(3),
                    source: EntityReference::Existing(n.a),
                    target: EntityReference::Existing(n.x),
                }),
            )),
        );
        let mut expected = parents(&n);
        let parent = n.ax;
        expected.push(record(&[parent, az], n.x, "x"));
        for (leaf, entity, value) in [(n.ax, n.a, "A"), (n.bx, n.b, "B")] {
            expected.push(record(&[parent, az, leaf], entity, value));
        }
        expected.push(record(&[n.bx, bz], n.z, "changed"));
        expect(court, &n, expected.clone());
        clear(court);
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("retire-nested-generation").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: n.z }),
            )),
        );
        expected.retain(|row| row.entity != n.z);
        expect(court, &n, expected.clone());
        let replacement = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("replacement-nested-row").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(4),
                    client_key: ClientKey::raw("z"),
                    fields: fixture::field("replacement"),
                }),
            )),
        )
        .created_entity(&created(4, "z"))
        .unwrap();
        assert_ne!(
            replacement, n.z,
            "full physical lifetime cannot alias by client key"
        );
        let before = basis(runtime);
        let mut stale = runtime
            .begin_branch_transaction(
                &before,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        stale
            .push_batch(
                WorkerIntentBatch::new("retired-parent-cannot-retarget").push(
                    MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
                        UpdateRelationEndpointsIntent {
                            relation_id: bz,
                            kind_id: KindId(3),
                            source: EntityReference::Existing(n.b),
                            target: EntityReference::Existing(replacement),
                        },
                    )),
                ),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(stale.commit(&execution, runtime), Err(TransactionCommitError::Conflict { error, .. })
            if matches!(error.class, ConflictClass::StaleTarget { .. }))
        );
        assert_eq!(basis(runtime), before);
        clear(court);
        let born = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("attach-new-nested-generation").push(link(
                3,
                n.b,
                replacement,
                "Bz-replacement",
            )),
        );
        let new_parent = relation(&born, 3, n.b, replacement, "Bz-replacement");
        assert_ne!(
            new_parent, bz,
            "retired parent lifetime never becomes the new occurrence"
        );
        expected.push(record(&[n.bx, new_parent], replacement, "replacement"));
        expect(court, &n, expected);
    });
}

#[test]
fn nested_projection_selected_backing_keeps_lineage_payer_after_all_loans_drop() {
    run(|runtime, program, court, host, _cancel| {
        let execution = ExecutionRequest::serial(host);
        let n = seed(runtime, execution);
        let (az, bz) = observe_links(runtime, execution, &n);
        let fresh = SerialRequest::from_memory(
            SerialMemoryBudget::new(None),
            CancellationSource::new().token(),
            None,
        );
        let execution = ExecutionRequest::serial(&fresh);
        assert_eq!(fresh.memory().unwrap().charged_memory_bytes(), 0);
        let escaped = {
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis(runtime).observation())
                .unwrap();
            let loan = runtime
                .read_truth()
                .prepare_managed_index_rebuild(&execution, &snapshot, program, n.a)
                .unwrap();
            let rows = loan.rows(execution);
            let all = rows.acquire_all().unwrap();
            let root = rows.occurrences(&all).unwrap().pop().unwrap();
            let actual = tree(&rows, &root, None).unwrap();
            let mut expected = parents(&n);
            for (parent, observes) in [(n.ax, az), (n.bx, bz)] {
                expected.push(record(&[parent, observes], n.z, "z"));
                expected.push(record(&[parent, observes, n.cz], n.c, "C"));
            }
            expected.sort();
            assert_eq!(actual, expected);
            let parent = rows.prepare_projection(&root, 0).unwrap();
            let occurrence = rows.projection_occurrences(&parent).unwrap().remove(0);
            let nested = rows
                .prepare_nested_projection(&parent, &occurrence, 1)
                .unwrap();
            let child = rows.projection_occurrences(&nested).unwrap().pop().unwrap();
            // A new selected image cannot consume the old parent's prepared backing.
            clear(court);
            commit(
                runtime,
                &ExecutionRequest::serial(host),
                WorkerIntentBatch::new("new-selected-image").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: n.z,
                        fields: fixture::field("next"),
                    }),
                )),
            );
            let new_snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis(runtime).observation())
                .unwrap();
            let new_loan = runtime
                .read_truth()
                .prepare_managed_index_rebuild(&execution, &new_snapshot, program, n.a)
                .unwrap();
            assert!(matches!(
                new_loan
                    .rows(execution)
                    .prepare_nested_projection(&parent, &occurrence, 1),
                Err(DerivedRelationReadDenial::Definition(
                    DerivedRelationDenial::ForeignObservation
                ))
            ));
            runtime.snapshots().release_snapshot(&new_snapshot).unwrap();
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
            child
        };
        assert!(
            fresh.memory().unwrap().charged_memory_bytes() > 0,
            "owned nested occurrence still retains original funding after Rows/read/loan drop"
        );
        drop(escaped);
        assert_eq!(
            fresh.memory().unwrap().charged_memory_bytes(),
            0,
            "last actual lineage share releases original payer"
        );
    });
}

#[test]
fn nested_projection_original_field_cancellation_refuses_without_moving_published_head() {
    run(|runtime, _program, court, host, cancel| {
        let execution = ExecutionRequest::serial(host);
        let n = seed(runtime, execution);
        observe_links(runtime, execution, &n);
        let before = basis(runtime);
        *court.cancel_nested_field.lock().unwrap() = Some(cancel.clone());
        let mut transaction = runtime
            .begin_branch_transaction(
                &before,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        transaction
            .push_batch(
                WorkerIntentBatch::new("cancel-nested-maintenance").push(MutationIntent::Entity(
                    EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                        entity_id: n.z,
                        fields: fixture::field("denied"),
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(transaction.commit(&execution,runtime),Err(TransactionCommitError::Execution{denial,..})
            if matches!(denial.kind,CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::Cancelled)))
        );
        assert_eq!(basis(runtime), before);
    });
}
