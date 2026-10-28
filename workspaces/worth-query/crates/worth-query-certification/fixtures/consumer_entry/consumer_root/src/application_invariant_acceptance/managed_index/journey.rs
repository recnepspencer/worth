//! One public lifecycle journey, with an independent ordinary-source scan oracle.
use super::super::authentication;
use super::{host, prefix, query};
use std::sync::{atomic::Ordering, Arc};
use worth_query_host::facade::{application_entry::*, primary_graph::*};

fn input(directory: &str, prefix: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: directory.to_owned(),
        prefix: prefix.to_owned(),
    }
}
fn not_ready(cause: WorthQueryApplicationRequestQueryDenial) {
    match cause {
        WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause) => assert!(matches!(
            cause.cause(),
            WorthQueryManagedIndexDenialCause::Refused(WorthQueryManagedIndexRefusal::NotReady)
        )),
        cause => panic!("expected actual managed NotReady, got {cause:?}"),
    }
}
#[test]
fn public_prefix_index_rebuild_preserves_source_slots_and_ordinary_query_results() {
    std::thread::Builder::new()
        .name("managed-prefix-consumer".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}
fn run() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(Arc::clone(&calls));
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());

    let source = request
        .query(query::ReadSource {
            directory: "catalog".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 7);
    assert_eq!(
        source
            .rows()
            .iter()
            .map(|row| row.name.as_str())
            .collect::<Vec<_>>(),
        ["álamo", "beta", "alpine", "alphabet", "alpha", "alpha", ""]
    );
    // The ordinary authored order genuinely differs from canonical full-GEN
    // source slots; the provider independently checks every issued slot's ID.
    assert!(!source
        .rows()
        .windows(2)
        .all(|pair| pair[0].coordinate < pair[1].coordinate));
    assert_eq!(calls.totals(), (0, 0, 0));
    let index = request
        .query(input("catalog", "al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Created);
    not_ready(
        request
            .query(input("catalog", "al"))
            .execute()
            .err()
            .expect("the cold image refuses"),
    );
    assert_eq!(
        calls.totals(),
        (0, 0, 0),
        "Created refusal never invokes the provider"
    );
    request
        .query(input("catalog", "unused probe"))
        .rebuild_index(&index)
        .unwrap();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(calls.totals(), (1, 1, 0));
    assert_eq!(calls.canonical_rows.load(Ordering::SeqCst), 7);
    let mut canonical = source
        .rows()
        .iter()
        .map(|row| row.coordinate)
        .collect::<Vec<_>>();
    canonical.sort();
    assert_eq!(&calls.canonical_coordinates.lock().unwrap()[..7], canonical);
    let sibling = request
        .query(input("catalog", "another probe"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(sibling.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(
        calls.totals(),
        (1, 1, 0),
        "probe values do not fork or reconstruct the source image"
    );
    for prefix in ["", "al", "alpha", "ál", "a\u{301}", "missing", "β"] {
        let actual = request.query(input("catalog", prefix)).execute().unwrap();
        assert_eq!(actual.rows(), expected(source.rows(), prefix), "UTF-8 prefix {prefix:?} preserves actual distinct source occurrences and Parent ordering");
    }
    let duplicates = request.query(input("catalog", "alpha")).execute().unwrap();
    assert_eq!(
        duplicates
            .rows()
            .iter()
            .map(|row| row.identity.as_str())
            .collect::<Vec<_>>(),
        ["item-b", "item-c", "item-g"]
    );

    let before = calls.totals();
    index.evict().unwrap();
    assert_eq!(
        sibling.lifecycle(),
        WorthQueryManagedIndexLifecycle::Evicted
    );
    not_ready(
        request
            .query(input("catalog", "al"))
            .execute()
            .err()
            .expect("the cold image refuses"),
    );
    assert_eq!(
        calls.totals(),
        before,
        "eviction refusal never invokes or silently rebuilds the provider"
    );
    request
        .query(input("catalog", "al"))
        .rebuild_index(&sibling)
        .unwrap();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(calls.totals(), (2, 2, before.2));
    // Ready/callback counts alone cannot establish reconstructed query meaning.
    for prefix in ["", "al", "alpha", "\u{e1}l", "missing"] {
        let actual = request.query(input("catalog", prefix)).execute().unwrap();
        assert_eq!(actual.rows(), expected(source.rows(), prefix));
    }
    sibling.close();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
    assert_eq!(
        sibling.lifecycle(),
        WorthQueryManagedIndexLifecycle::Disposed
    );
    let fresh = request
        .query(input("catalog", "al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Created);
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
    let after_close = calls.totals();
    not_ready(
        request
            .query(input("catalog", "al"))
            .execute()
            .err()
            .expect("the cold image refuses"),
    );
    assert_eq!(calls.totals(), after_close);
    fresh.close();

    let empty_source = request
        .query(query::ReadSource {
            directory: "empty".to_owned(),
        })
        .execute()
        .unwrap();
    assert!(empty_source.rows().is_empty());
    let empty = request
        .query(input("empty", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request
        .query(input("empty", ""))
        .rebuild_index(&empty)
        .unwrap();
    let empty_result = request.query(input("empty", "")).execute().unwrap();
    assert!(empty_result.rows().is_empty());
    assert_eq!(calls.canonical_rows.load(Ordering::SeqCst), 0);
    assert_eq!(empty.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    empty.close();
}

pub(super) fn expected(rows: &[query::ItemRow], prefix: &str) -> Vec<query::ItemRow> {
    let mut expected = rows
        .iter()
        .filter(|row| row.enabled && row.name.starts_with(prefix))
        .cloned()
        .collect::<Vec<_>>();
    expected.sort_by(|a, b| a.identity.cmp(&b.identity));
    expected
}
