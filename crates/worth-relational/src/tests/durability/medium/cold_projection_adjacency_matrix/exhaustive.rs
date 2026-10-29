use super::*;

#[test]
fn residency_failure_matrix_projection_adjacency_guards_selected_heads() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for subject in Subject::ALL {
        for door in Door::ALL {
            for historical in [false, true] {
                for (cause, sticky) in FAILURES {
                    let cold = recover_shape_inner(
                        execution,
                        None,
                        4,
                        true,
                        matches!(subject, Subject::RemoteSymbols),
                    );
                    let partition = if subject.remote() {
                        cold.entities[3].partition_id
                    } else {
                        cold.entities[0].partition_id
                    };
                    if subject.remote() {
                        let mut transaction = test_owner_begin_transaction_for_main(&cold.runtime);
                        assert!(transaction
                            .read_entity(execution, cold.entities[0])
                            .unwrap()
                            .effective()
                            .is_some());
                    }
                    let symbol_page = cold.remote_symbol.map(|symbol| {
                        cold.manifest
                            .symbols
                            .iter()
                            .find(|page| {
                                page.first <= u64::from(symbol.0)
                                    && u64::from(symbol.0) <= page.last
                            })
                            .unwrap()
                    });
                    let targets = if subject.symbols() {
                        if let Some(page) = symbol_page {
                            std::collections::BTreeSet::from([page.member])
                        } else {
                            cold.manifest
                                .symbols
                                .iter()
                                .map(|page| page.member)
                                .collect()
                        }
                    } else {
                        cold.manifest
                            .roots
                            .iter()
                            .flat_map(|root| &root.regions)
                            .chain(&cold.manifest.mirror)
                            .filter(|region| region.partition_id == partition)
                            .map(|region| {
                                if subject.history() {
                                    region.history
                                } else {
                                    region.head
                                }
                            })
                            .collect::<std::collections::BTreeSet<_>>()
                    };
                    if matches!(subject, Subject::RemoteSymbols) {
                        assert!(
                            !cold
                                .source
                                .member_reads
                                .lock()
                                .unwrap()
                                .iter()
                                .any(|member| targets.contains(member)),
                            "anchor demand leaves the actual remote text page cold"
                        );
                    }
                    cold.source.failures.lock().unwrap().extend(
                        targets
                            .iter()
                            .map(|member| (ColdRead::Member(*member), cause)),
                    );
                    let frontier = cold.runtime.history().latest_patch_stream_position();
                    let offered = attached_test_medium(&cold.runtime).offered().len();
                    let expected = Execution::StorageUnavailable(
                        crate::storage::data::RelationalStorageUnavailable {
                            subject: if subject.symbols() {
                                let page = symbol_page.unwrap_or(&cold.manifest.symbols[0]);
                                crate::storage::data::RelationalStorageSubject::Symbols {
                                    first: crate::symbols::data::Symbol(
                                        page.first.try_into().unwrap(),
                                    ),
                                    last: crate::symbols::data::Symbol(
                                        page.last.try_into().unwrap(),
                                    ),
                                }
                            } else {
                                crate::storage::data::RelationalStorageSubject::Region(partition)
                            },
                            cause,
                        },
                    );
                    let actual = door.read(&cold, execution, historical);
                    if subject.required(door, historical) {
                        assert_eq!(
                            actual,
                            Err(expected),
                            "{subject:?} x {door:?} x historical={historical} x {cause:?}"
                        );
                        cold.source.failures.lock().unwrap().clear();
                        let reads = cold.source.reads.load(Ordering::Relaxed);
                        if sticky {
                            let failed_reads = || {
                                cold.source
                                    .member_reads
                                    .lock()
                                    .unwrap()
                                    .iter()
                                    .filter(|member| targets.contains(member))
                                    .count()
                            };
                            let failed_before = failed_reads();
                            assert_eq!(door.read(&cold, execution, historical), Err(expected));
                            assert_eq!(
                                failed_reads(),
                                failed_before,
                                "the proven damaged subject is never read again"
                            );
                            // Head decode may retry after a dictionary refusal: damage
                            // belongs to the page, never to the healthy region carrier.
                            if !subject.symbols() {
                                assert_eq!(cold.source.reads.load(Ordering::Relaxed), reads);
                            }
                        } else {
                            door.read(&cold, execution, historical).unwrap();
                            assert!(cold.source.reads.load(Ordering::Relaxed) > reads);
                        }
                    } else {
                        actual.unwrap_or_else(|error| panic!("unconsulted {subject:?} x {door:?} x historical={historical}: {error:?}"));
                        assert!(
                            !cold
                                .source
                                .member_reads
                                .lock()
                                .unwrap()
                                .iter()
                                .any(|member| targets.contains(member)),
                            "unconsulted {subject:?} x {door:?}"
                        );
                    }
                    assert_eq!(
                        cold.runtime.history().latest_patch_stream_position(),
                        frontier
                    );
                    assert_eq!(attached_test_medium(&cold.runtime).offered().len(), offered);
                }
            }
        }
    }
}
