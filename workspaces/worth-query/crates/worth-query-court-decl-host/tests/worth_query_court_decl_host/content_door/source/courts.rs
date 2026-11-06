use super::*;
fn committed(host: &world::Host, mode: u64, name: &str, samples: &[i64], key: &str) {
    let result = world::edit(host, mode, samples.to_vec(), name.as_bytes().to_vec(), key);
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
}
fn keys(rows: &[query::Row]) -> Vec<String> {
    rows.iter().map(|row| row.key.clone()).collect()
}
fn oracle(rows: &[(&str, &[i64])]) -> Vec<String> {
    let mut answer: Vec<_> = rows
        .iter()
        .filter(|(_, values)| values.iter().any(|value| *value >= 10))
        .map(|(key, _)| key.to_string())
        .collect();
    answer.sort();
    answer
}
#[test]
fn content_source_identity_changes_reach_exact_candidates_and_rebuild() {
    let calls = Arc::new(Calls::default());
    let host = world::install_with_calls(None, calls.clone());
    for (name, values) in [
        ("alpha", &[7, 11][..]),
        ("reject", &[-1, 3][..]),
        ("anchor", &[23][..]),
    ] {
        committed(&host, 18, name, values, &format!("create-{name}"));
    }
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let find = || query::Find {
        root: "record".into(),
        minimum: 10,
    };
    let source = || query::ReadSource {
        root: "record".into(),
    };
    let index = request.query(find()).open_index::<Index>().unwrap();
    request.query(find()).rebuild_index(&index).unwrap();
    let initial = request.query(source()).execute().unwrap();
    assert_eq!(keys(initial.rows()), ["alpha", "anchor", "reject"]);
    let alpha_a = initial.rows()[0].content;
    assert_eq!(
        keys(request.query(find()).execute().unwrap().rows()),
        oracle(&[("alpha", &[7, 11]), ("reject", &[-1, 3]), ("anchor", &[23])])
    );
    calls.records.lock().unwrap().clear();
    let before_update = calls.updates.load(Ordering::SeqCst);
    committed(&host, 19, "alpha", &[1, 2], "alpha-b");
    assert!(calls.updates.load(Ordering::SeqCst) > before_update);
    let b = request.query(source()).execute().unwrap();
    assert_ne!(b.rows()[0].content, alpha_a);
    let expected_b = oracle(&[("alpha", &[1, 2]), ("reject", &[-1, 3]), ("anchor", &[23])]);
    assert_eq!(
        keys(request.query(find()).execute().unwrap().rows()),
        expected_b
    );
    assert!(calls
        .records
        .lock()
        .unwrap()
        .iter()
        .any(|(before, after)| before.as_ref()
            == Some(&vec!["alpha".to_owned(), "anchor".to_owned()])
            && after == &expected_b));
    committed(&host, 19, "reject", &[17], "rejected-row-enters");
    assert_eq!(
        keys(request.query(find()).execute().unwrap().rows()),
        oracle(&[("alpha", &[1, 2]), ("reject", &[17]), ("anchor", &[23])])
    );
    let at_b = world::pin(&host);
    committed(&host, 19, "alpha", &[7, 11], "alpha-a-again");
    assert_ne!(world::pin(&host).selected_commit(), at_b.selected_commit());
    assert_eq!(
        request.query(source()).execute().unwrap().rows()[0].content,
        alpha_a
    );
    let expected_a = oracle(&[("alpha", &[7, 11]), ("reject", &[17]), ("anchor", &[23])]);
    assert_eq!(
        keys(request.query(find()).execute().unwrap().rows()),
        expected_a
    );
    request.query(find()).rebuild_index(&index).unwrap();
    assert_eq!(
        keys(request.query(find()).execute().unwrap().rows()),
        expected_a
    );
    assert!(calls.content_reads.load(Ordering::SeqCst) >= 3);
    assert!(calls.exact_reads.load(Ordering::SeqCst) >= 3);
    index.close();
}
#[test]
fn content_source_bad_reference_stops_before_provider_or_invariant_publication() {
    let calls = Arc::new(Calls::default());
    let host = world::install_with_calls(None, calls.clone());
    committed(&host, 18, "anchor", &[23], "anchor");
    committed(&host, 0, "", &[7, 11], "root-content");
    committed(&host, 4, "", &[5], "other-format");
    committed(&host, 5, "", &[13], "other-version");
    let values = world::edit(&host, 99, vec![], vec![], "read-references");
    let values = values.result().unwrap();
    for (i, reference, expected) in [
        (
            0,
            ContentRefId([0xE7; 32]),
            RelationalContentDenial::MissingContent,
        ),
        (
            1,
            values.other.unwrap(),
            RelationalContentDenial::FormatMismatch,
        ),
        (
            2,
            values.version.unwrap(),
            RelationalContentDenial::UnsupportedVersion,
        ),
    ] {
        let selected = world::pin(&host);
        let counts = (
            calls.content_reads.load(Ordering::SeqCst),
            calls.exact_reads.load(Ordering::SeqCst),
            calls.updates.load(Ordering::SeqCst),
            calls.records.lock().unwrap().len(),
        );
        let result = world::edit(
            &host,
            2,
            vec![],
            reference.0.to_vec(),
            &format!("refused-{i}"),
        );
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = result
        else {
            panic!("{result:?}")
        };
        assert_eq!(denial.content_denial(), Some(expected));
        assert_eq!(
            world::pin(&host).selected_commit(),
            selected.selected_commit()
        );
        assert_eq!(
            (
                calls.content_reads.load(Ordering::SeqCst),
                calls.exact_reads.load(Ordering::SeqCst),
                calls.updates.load(Ordering::SeqCst),
                calls.records.lock().unwrap().len()
            ),
            counts
        );
    }
}

#[test]
fn content_source_swallowed_projection_failure_remains_a_typed_refusal() {
    let calls = Arc::new(Calls::default());
    let host = world::install_with_calls(None, calls.clone());
    committed(&host, 18, "alpha", &[7, 11], "alpha");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let find = || query::Find {
        root: "record".into(),
        minimum: 10,
    };
    let index = request.query(find()).open_index::<Index>().unwrap();
    calls.swallowed_projection.store(1, Ordering::SeqCst);
    let before = world::pin(&host);
    let refused = request.query(find()).rebuild_index(&index).unwrap_err();
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(refused) = refused else {
        panic!("{refused:?}")
    };
    let WorthQueryManagedIndexDenialCause::Work(WorthQueryManagedIndexWorkDenial::Controlled(
        WorthQueryManagedIndexStop::Projection(cause),
    )) = refused.cause()
    else {
        panic!("{refused:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationProjectionDenialKind::FieldContractMismatch
    );
    assert_eq!(
        world::pin(&host).selected_commit(),
        before.selected_commit()
    );
    calls.swallowed_projection.store(0, Ordering::SeqCst);
    request.query(find()).rebuild_index(&index).unwrap();
    assert_eq!(
        keys(request.query(find()).execute().unwrap().rows()),
        ["alpha"]
    );
    index.close();
}

#[test]
fn content_source_equal_final_identity_does_not_erase_authored_history() {
    let host = world::install();
    committed(&host, 18, "alpha", &[7, 11], "alpha");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let original = request
        .query(query::ReadSource {
            root: "record".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(original.rows().len(), 1);
    let observed = original.observed_sources()[0].clone();
    committed(&host, 19, "alpha", &[1, 2], "b");
    committed(&host, 19, "alpha", &[7, 11], "a-again");
    assert_eq!(
        request
            .query(query::ReadSource {
                root: "record".into()
            })
            .execute()
            .unwrap()
            .rows(),
        original.rows()
    );
    let before = world::pin(&host);
    let result = request
        .advanced()
        .mutate(expectation::Observe)
        .expect_source(observed)
        .idempotency(&"stale-aba".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    ) = result.unwrap_err()
    else {
        panic!("stale Source must refuse as an attempt")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
    assert_eq!(
        world::pin(&host).selected_commit(),
        before.selected_commit()
    );
}

#[test]
fn content_recovered_current_source_rebuilds_from_native_checkpoint_bytes() {
    let host = world::install();
    committed(&host, 18, "alpha", &[7, 11], "alpha");
    committed(&host, 18, "reject", &[1, 2], "reject");
    let checkpoint = world::checkpoint(&host);
    let calls = Arc::new(Calls::default());
    let recovered = world::restore(checkpoint, calls.clone());
    let scope = request_scope();
    let principal = authenticate_operator(recovered.installed_schema(), &scope);
    let request = recovered.runtime().request(&principal, &scope);
    let source = request
        .query(query::ReadSource {
            root: "record".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(keys(source.rows()), ["alpha", "reject"]);
    assert_eq!(
        source.rows()[0].content.to_string(),
        "7bcb0e089d83cadc112d673d0f68e3200fdde9f3b4b46626f60e6f83667d165a"
    );
    assert_eq!(
        source.rows()[1].content.to_string(),
        "c40db3f9f76eab9977a59ff05cdf9ebe3b6fb892c1ef77a8bb04744861d5bdef"
    );
    let find = || query::Find {
        root: "record".into(),
        minimum: 10,
    };
    let selected = world::pin(&recovered);
    let refused = match request.query(find()).open_index::<Index>() {
        Err(cause) => cause,
        Ok(_) => panic!("recovered bytes cannot mint a live managed Source image"),
    };
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause) = refused else {
        panic!("{refused:?}")
    };
    assert!(matches!(
        cause.cause(),
        WorthQueryManagedIndexDenialCause::Refused(WorthQueryManagedIndexRefusal::RootUnavailable)
    ));
    assert_eq!(
        world::pin(&recovered).selected_commit(),
        selected.selected_commit()
    );
    calls.records.lock().unwrap().clear();
    // A genuine authored transition may cold-stage current data; it does not mint old live history.
    // Alpha remains unchanged, so its recovered bytes must also reach the consuming candidate.
    committed(&recovered, 19, "reject", &[17], "entrant-after-recovery");
    let before = oracle(&[("alpha", &[7, 11]), ("reject", &[1, 2])]);
    let after = oracle(&[("alpha", &[7, 11]), ("reject", &[17])]);
    assert!(calls
        .records
        .lock()
        .unwrap()
        .iter()
        .any(|(b, a)| b.as_ref() == Some(&before) && a == &after));
    let index = request.query(find()).open_index::<Index>().unwrap();
    request.query(find()).rebuild_index(&index).unwrap();
    assert_eq!(keys(request.query(find()).execute().unwrap().rows()), after);
    index.close();
}
