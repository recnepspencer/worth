//! Actual current-loan row tickets refuse foreign views and removed members.
use super::*;

pub(super) fn observe(
    source: &ManagedIndexSourcePreparation<'_>,
    court: &owner::Court,
) -> Result<(), ManagedIndexSourcePreparationDenial> {
    let scopes: Vec<_> = source.scopes().collect();
    for (index, scope) in scopes.iter().enumerate() {
        if !scope.scope_exists()? {
            continue;
        }
        let rows = scope.proposed();
        let forward = rows.acquire_all()?;
        let occurrences = rows.occurrences(&forward)?;
        for occurrence in &occurrences {
            let prepared = rows.prepare_row_from_read(&forward, occurrence)?;
            assert!(
                prepared.with_field(0, |value| value.is_none())?,
                "optional projected field absence stays absent"
            );
            assert!(
                prepared
                    .with_field_content(0, |value, frame| value.is_none() && frame.is_none())?,
                "content acquisition preserves genuine optional absence"
            );
            let inverse = rows
                .prepare_occurrence(occurrence.entity())?
                .expect("actual inverse membership");
            if let Some(other) = occurrences
                .iter()
                .find(|other| other.entity() != occurrence.entity())
            {
                assert!(matches!(
                    rows.prepare_row_from_occurrence(&inverse, other),
                    Err(DerivedRelationReadDenial::Definition(
                        DerivedRelationDenial::ForeignObservation
                    ))
                ));
            }
            let prepared = rows.prepare_row_from_occurrence(&inverse, occurrence)?;
            assert!(prepared.with_field(0, |value| value.is_none())?);
            court.row_read_checks.lock().unwrap()[0] += 1;
            if let Some(cancel) = court.cancel_row_field.lock().unwrap().take() {
                prepared.with_field(0, |_| cancel.cancel())?;
                let entered = std::cell::Cell::new(false);
                let stopped = prepared.with_field_content(0, |_, _| entered.set(true));
                assert!(
                    !entered.get(),
                    "original cancellation precedes field callback"
                );
                return Err(stopped.expect_err("original ticket cannot outlive cancellation"));
            }
            if let Some(other) = scopes
                .iter()
                .enumerate()
                .find(|(other, _)| *other != index)
                .map(|(_, scope)| scope)
            {
                let foreign_rows = other.proposed();
                assert!(matches!(
                    foreign_rows.prepare_row_from_occurrence(&inverse, occurrence),
                    Err(DerivedRelationReadDenial::Definition(
                        DerivedRelationDenial::ForeignDefinition
                    ))
                ));
                let foreign_read = foreign_rows.acquire_all()?;
                assert!(matches!(
                    rows.prepare_row_from_read(&foreign_read, occurrence),
                    Err(DerivedRelationReadDenial::Definition(
                        DerivedRelationDenial::ForeignObservation
                    ))
                ));
                court.row_read_checks.lock().unwrap()[1] += 1;
            }
        }
        if scope.committed_scope_exists()? {
            let old = scope.committed()?;
            let old_read = old.acquire_all()?;
            for occurrence in old.occurrences(&old_read)? {
                let old_inverse = old
                    .prepare_occurrence(occurrence.entity())?
                    .expect("old membership");
                assert!(matches!(
                    rows.prepare_row_from_occurrence(&old_inverse, &occurrence),
                    Err(DerivedRelationReadDenial::Definition(
                        DerivedRelationDenial::ForeignObservation
                    ))
                ));
                assert!(matches!(
                    rows.prepare_row_from_read(&old_read, &occurrence),
                    Err(DerivedRelationReadDenial::Definition(
                        DerivedRelationDenial::ForeignObservation
                    ))
                ));
                court.row_read_checks.lock().unwrap()[2] += 1;
                if occurrences
                    .iter()
                    .any(|current| current.entity() == occurrence.entity())
                {
                    assert!(rows
                        .prepare_row(&occurrence)?
                        .with_field(0, |value| value.is_none())?);
                } else {
                    assert!(matches!(
                        rows.prepare_row_from_read(&forward, &occurrence),
                        Err(DerivedRelationReadDenial::Definition(
                            DerivedRelationDenial::RecordUnavailable
                        ))
                    ));
                    assert!(matches!(
                        rows.prepare_row(&occurrence),
                        Err(DerivedRelationReadDenial::Definition(
                            DerivedRelationDenial::RecordUnavailable
                        ))
                    ));
                    court.row_read_checks.lock().unwrap()[3] += 1;
                }
            }
        }
    }
    Ok(())
}
fn birth() -> WorkerIntentBatch {
    let mut batch = WorkerIntentBatch::new("current-row-birth");
    for (kind, label) in [(1, "A"), (1, "B"), (4, "x"), (4, "y")] {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(kind),
            client_key: ClientKey::raw(label),
            fields: AspectFieldPatch::from_locator(
                planned_single_field_locator(registry::key(), registry::field("other")),
                worth_foundational::facade::AspectValue::String(label.into()),
            ),
        })));
    }
    for (scope, row, label) in [("A", "x", "A-x"), ("A", "y", "A-y"), ("B", "y", "B-y")] {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw(label),
                source: EntityReference::Created(created(1, scope)),
                target: EntityReference::Created(created(4, row)),
                fields: Default::default(),
            },
        )));
    }
    batch
}
#[test]
fn current_row_reads_keep_current_membership_and_original_field_cancellation() {
    for cancel in [false, true] {
        let (registry, program) = registry::profile(registry::Profile::Fields);
        let court = Arc::new(owner::Court::default());
        court.probe_row_reads.store(true, Ordering::SeqCst);
        let rule = CustomInvariantRegistration::new(owner::Reader {
            maximum_work: None,
            program: program.clone(),
            court: court.clone(),
        })
        .unwrap()
        .with_managed_index_sources(vec![program])
        .unwrap();
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(registry)
            .custom_invariant(rule)
            .build()
            .unwrap();
        let _registration = runtime
            .publication_companion_port()
            .begin_required_registration()
            .unwrap()
            .activate(Arc::new(owner::SourceOwner(court.clone())))
            .unwrap();
        let cancellation = CancellationSource::new();
        let host =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
        let execution = ExecutionRequest::serial(&host);
        let born = commit(&runtime, &execution, birth());
        let b = born.created_entity(&created(1, "B")).unwrap();
        let x = born.created_entity(&created(4, "x")).unwrap();
        let relation = born
            .created_relation(&CreatedRelationRef {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("A-x"),
                source: EntityReference::Created(created(1, "A")),
                target: EntityReference::Created(created(4, "x")),
            })
            .unwrap();
        assert!(court.row_read_checks.lock().unwrap()[0] >= 2);
        assert!(court.row_read_checks.lock().unwrap()[1] >= 2);
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        if cancel {
            *court.cancel_row_field.lock().unwrap() = Some(cancellation.clone());
        }
        // Retiring A's edge leaves the full-generation row live under no A membership.
        let mut transaction = runtime
            .begin_branch_transaction(
                &basis,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        let batch = if cancel {
            // Keep a live occurrence so cancellation happens after a real admitted field.
            WorkerIntentBatch::new("cancel-current-row").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw("B-x"),
                    source: EntityReference::Existing(b),
                    target: EntityReference::Existing(x),
                    fields: Default::default(),
                }),
            ))
        } else {
            WorkerIntentBatch::new("retire-current-row").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                }),
            ))
        };
        transaction
            .push_batch(
                batch,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let result = transaction.commit(&execution, &runtime);
        if cancel {
            assert!(
                matches!(result, Err(TransactionCommitError::Execution { denial, .. })
                if matches!(denial.kind, CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::Cancelled)))
            );
            assert_eq!(
                runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap()
                    .1,
                basis
            );
        } else {
            result.unwrap();
            let checks = *court.row_read_checks.lock().unwrap();
            assert!(
                checks[2] > 0 && checks[3] > 0,
                "both view and retained-removal paths ran: {checks:?}"
            );
        }
    }
}
