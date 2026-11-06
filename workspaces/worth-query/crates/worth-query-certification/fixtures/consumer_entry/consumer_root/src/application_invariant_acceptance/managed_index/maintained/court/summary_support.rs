//! A genuine same-Source slot outside a leaf's region cannot influence its summary.
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;

#[test]
fn checked_summary_confines_row_resolution_to_the_actual_region() {
    lifecycle::run(journey);
}
fn journey() {
    let provider = Arc::new(prefix_observation::Calls::default());
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
    // The actual root's lawful bounds reads first populate Ready ancestry.
    // A later genuine leaf is still not allowed to read that root's Source.
    provider.query_read_first.store(true, Ordering::SeqCst);
    let error = request
        .query(find("B", ""))
        .execute()
        .err()
        .expect("an outside leaf cannot reuse the root's region admission");
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
    assert_eq!(
        provider.query_region_reads.load(Ordering::SeqCst),
        1,
        "lawful root read precedes the leaf denial in the SAME query/context"
    );
    provider.query_read_first.store(false, Ordering::SeqCst);
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
    let expected = values(source.rows())
        .into_iter()
        .filter(|(_, name)| name.starts_with("al"))
        .collect::<Vec<_>>();
    let mut work = [0; 2];
    for (position, repetitions) in [32, 64].into_iter().enumerate() {
        provider
            .summary_repetitions
            .store(repetitions, Ordering::SeqCst);
        let (actual, invoice) = invoice(|| request.query(find("C", "al")).execute().unwrap());
        assert_eq!(values(actual.rows()), expected);
        work[position] = invoice;
    }
    provider.summary_repetitions.store(0, Ordering::SeqCst);
    eprintln!("actual Source8 repeated-extrema whole read SDK work32/64={work:?}");
    assert!(
        work[0] > 0 && work[1] < 3 * work[0],
        "whole fixed-Source repeated-read growth, separately from numeric parent-port reuse"
    );
    checked.close();
}

fn invoice<T>(operation: impl FnOnce() -> T) -> (T, u64) {
    use worth_execution::{
        CancellationSource, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
    };
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let (result, report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&original), |_| operation())
        .unwrap();
    (result, report.charged_work())
}
