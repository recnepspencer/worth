//! A genuine same-Source slot outside a leaf's region cannot influence its summary.
use super::*;

#[test]
fn checked_summary_confines_row_resolution_to_the_actual_region() {
    lifecycle::run(journey);
}
fn journey() {
    let provider = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(provider.clone()));
    let world = host::install(provider.clone(), rules);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    // B has two genuine Source occurrences, so a leaf cannot contain them both.
    let index = request
        .query(find("B", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let ordinary = request
        .query(query::ReadSource {
            directory: "B".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(ordinary.rows().len(), 2);
    request.query(find("B", "")).evict_index(&index).unwrap();
    provider.summary_read_first.store(true, Ordering::SeqCst);
    let error = request
        .query(find("B", ""))
        .rebuild_index(&index)
        .unwrap_err();
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(error) = error else {
        panic!("{error:?}");
    };
    assert!(
        matches!(
            error.cause(),
            WorthQueryManagedIndexDenialCause::Work(WorthQueryManagedIndexWorkDenial::Controlled(
                WorthQueryManagedIndexStop::ForeignSummarySource
            ))
        ),
        "{error:?}"
    );
    provider.summary_read_first.store(false, Ordering::SeqCst);
    request.query(find("B", "")).rebuild_index(&index).unwrap();
    assert_eq!(
        values(request.query(find("B", "")).execute().unwrap().rows()),
        values(ordinary.rows())
    );
    index.close();
    let edits = std::iter::once(super::super::authoring::Edit::Directory {
        identity: "C".into(),
    })
    .chain(
        [
            "álamo", "alpha", "alphabet", "beta", "a\u{301}", "β", "", "alpha",
        ]
        .into_iter()
        .enumerate()
        .map(|(index, name)| super::super::authoring::Edit::Item {
            directory: "C".into(),
            identity: format!("unicode-{index}"),
            name: name.into(),
        }),
    );
    for (index, edit) in edits.enumerate() {
        let result = request
            .advanced()
            .mutate(edit)
            .idempotency(&(0x919630u64 + index as u64))
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                result,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{result:?}"
        );
    }
    let source = request
        .query(query::ReadSource {
            directory: "C".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 8);
    let checked = request
        .query(find("C", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    for probe in ["", "al", "alpha", "ál", "a\u{301}", "β", "missing"] {
        let expected = values(source.rows())
            .into_iter()
            .filter(|(_, name)| name.starts_with(probe))
            .collect::<Vec<_>>();
        assert_eq!(
            values(request.query(find("C", probe)).execute().unwrap().rows()),
            expected,
            "checked extrema preserve independent UTF-8 membership for {probe:?}"
        );
    }
    checked.close();
}
