//! Noncommuting event semantics and actual before/after changed-row cohorts.
use super::*;
use fixture::*;
#[test]
fn ordered_event_source_preserves_newer_introductions_and_same_step_retirement() {
    with_event_source(|runtime, control, court, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = seed(runtime, &execution);
        let [a, b, c, d, u] = w.events;
        let [x, y, z] = w.rows;
        let full: [&[usize]; 5] = [&[0, 1], &[1], &[0, 1], &[0, 1], &[2]];
        expect(control, court, &w, [&[]; 5], full, &[0, 1, 2, 3, 4]);
        let born = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("ordered-event-history")
                .push(link(5, b, a, "ba"))
                .push(link(5, c, b, "cb"))
                .push(link(5, d, c, "dc"))
                .push(link(2, a, x, "ax"))
                .push(link(2, a, y, "ay"))
                .push(link(3, b, x, "bx-retired"))
                .push(link(2, c, x, "cx-reintroduced"))
                .push(link(2, d, z, "dz-introduced"))
                .push(link(3, d, z, "dz-retired"))
                .push(link(2, u, z, "uz")),
        );
        assert_scopes(court, &w, &[0, 1, 2, 3, 4]);
        assert!(
            control.checked.load(Ordering::SeqCst) >= 5,
            "actual forward/inverse observations ran"
        );
        let after: [&[usize]; 5] = [&[0, 1], &[1], &[1], &[1], &[2]];
        expect(control, court, &w, full, after, &[0, 1, 2, 3]);
        let removed = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("remove-newer-introduction").push(delete(relation(
                &born,
                2,
                c,
                x,
                "cx-reintroduced",
            ))),
        );
        assert!(
            removed.change_summary().unwrap().adjacency_delta_count > 0,
            "deleting the actual introduction must carry its physical adjacency effect: {:?}",
            removed.change_summary()
        );
        assert_scopes(court, &w, &[0, 1, 2, 3]);
        for event in [c, d] {
            assert_changed(court, event, &[x, y]);
        }
        // Deleting retirement makes x re-enter through its older introduction.
        expect(
            control,
            court,
            &w,
            after,
            [&[0, 1], &[0, 1], &[0, 1], &[0, 1], &[2]],
            &[0, 1, 2, 3],
        );
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("remove-retirement").push(delete(relation(
                &born,
                3,
                b,
                x,
                "bx-retired",
            ))),
        );
        assert_scopes(court, &w, &[0, 1, 2, 3]);
    });
}
#[test]
fn ordered_event_predecessor_rewire_replaces_old_and_new_prefix_members() {
    with_event_source(|runtime, control, court, host, _| {
        let execution = ExecutionRequest::serial(host);
        let w = seed(runtime, &execution);
        let [a, b, c, d, u] = w.events;
        let [x, _, z] = w.rows;
        let before: [&[usize]; 5] = [&[0], &[0], &[0], &[0], &[2]];
        expect(control, court, &w, [&[]; 5], before, &[0, 1, 2, 3, 4]);
        let born = commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("rewire-history")
                .push(link(5, b, a, "ba"))
                .push(link(5, c, b, "cb"))
                .push(link(5, d, c, "dc"))
                .push(link(2, a, x, "ax"))
                .push(link(2, u, z, "uz")),
        );
        let after: [&[usize]; 5] = [&[0], &[0], &[2], &[2], &[2]];
        expect(control, court, &w, before, after, &[0, 1, 2, 3, 4]);
        commit(
            runtime,
            &execution,
            WorkerIntentBatch::new("rewire-to-other-history")
                .push(delete(relation(&born, 5, c, b, "cb")))
                .push(link(5, c, u, "cu")),
        );
        assert_scopes(court, &w, &[0, 1, 2, 3, 4]);
        for event in [c, d] {
            assert_changed(court, event, &[x, z]);
        }
    });
}

#[test]
fn ordered_birth_reports_row_dependency_records_and_excludes_scope_entity() {
    let (registry, program, path) = registry::ordered_predecessor_profile();
    let court = Arc::new(owner::Court::default());
    let control = Control::default();
    let rule = CustomInvariantRegistration::new(Reader {
        base: owner::Reader {
            maximum_work: None,
            program: program.clone(),
            court: court.clone(),
        },
        path,
        control: control.clone(),
    })
    .unwrap()
    .with_managed_index_sources(vec![program])
    .unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .custom_invariant(rule)
        .build()
        .unwrap();
    let _reg = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court.clone())))
        .unwrap();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let w = seed(&runtime, &execution);
    let [a, _, _, _, _] = w.events;
    let [x, _, _] = w.rows;
    commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("seed-initial-event").push(link(2, a, x, "ax")),
    );

    // Ordered birth creating ONLY a new event scope (KindId(1), not a row
    // or projection endpoint) must take the ordered-birth path (only the
    // new scope staged) and report Some(&[]).
    court.staged.lock().unwrap().clear();
    court.occurrence_records.lock().unwrap().clear();
    let born_scope_only = commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("ordered-birth-scope-only")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("e-born-1"),
                fields: Default::default(),
            })))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(5),
                    client_key: ClientKey::raw("e-born-1-pred"),
                    source: EntityReference::Created(created(1, "e-born-1")),
                    target: EntityReference::Existing(a),
                    fields: Default::default(),
                },
            ))),
    );
    let e1 = born_scope_only
        .created_entity(&created(1, "e-born-1"))
        .unwrap();
    assert_eq!(*court.staged.lock().unwrap(), BTreeSet::from([e1]));
    assert_eq!(
        court.occurrence_records.lock().unwrap().get(&e1),
        Some(&Some(Vec::new()))
    );

    // Ordered birth creating a new event scope AND a new row (KindId(4))
    // must report Some(&[new_row]), excluding the new scope entity.
    court.staged.lock().unwrap().clear();
    court.occurrence_records.lock().unwrap().clear();
    let born_with_row = commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("ordered-birth-scope-and-row")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("e-born-2"),
                fields: Default::default(),
            })))
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(4),
                client_key: ClientKey::raw("r-born-2"),
                fields: Default::default(),
            })))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(5),
                    client_key: ClientKey::raw("e-born-2-pred"),
                    source: EntityReference::Created(created(1, "e-born-2")),
                    target: EntityReference::Existing(e1),
                    fields: Default::default(),
                },
            )))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw("e-born-2-intro"),
                    source: EntityReference::Created(created(1, "e-born-2")),
                    target: EntityReference::Created(created(4, "r-born-2")),
                    fields: Default::default(),
                },
            ))),
    );
    let e2 = born_with_row
        .created_entity(&created(1, "e-born-2"))
        .unwrap();
    let r2 = born_with_row
        .created_entity(&created(4, "r-born-2"))
        .unwrap();
    assert_eq!(*court.staged.lock().unwrap(), BTreeSet::from([e2]));
    assert_eq!(
        court.occurrence_records.lock().unwrap().get(&e2),
        Some(&Some(vec![r2]))
    );
}
