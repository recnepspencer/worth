//! Genuine reverse/repeated emissions preserve ordinary identity order and slot ownership.
use super::super::authentication;
use super::{host, prefix, query};
use std::sync::{atomic::Ordering, Arc};
use worth_query_host::facade::{application_entry::*, primary_graph::*};
fn input(directory: &str, prefix: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: directory.into(),
        prefix: prefix.into(),
    }
}
#[test]
fn actual_candidate_slots_are_output_ordered_duplicate_idempotent_and_foreign_refused() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(journey)
        .unwrap()
        .join()
        .unwrap();
}
fn journey() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(calls.clone());
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
    let source = request
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 7);
    let index = request
        .query(input("catalog", "al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request
        .query(input("catalog", "al"))
        .rebuild_index(&index)
        .unwrap();
    let foreign = calls
        .first_source
        .lock()
        .unwrap()
        .expect("genuine catalog Source slot");
    for repeat in [false, true] {
        calls.repeat_candidates.store(repeat, Ordering::SeqCst);
        for prefix in ["", "al", "alpha", "ál", "absent"] {
            let mut expected: Vec<_> = source
                .rows()
                .iter()
                .filter(|row| row.enabled && row.name.starts_with(prefix))
                .cloned()
                .collect();
            expected.sort_by(|left, right| left.identity.cmp(&right.identity));
            let result = request.query(input("catalog", prefix)).execute().unwrap();
            assert_eq!(
                result.rows(),
                expected,
                "actual reverse emits, repeat={repeat}, prefix={prefix}"
            );
        }
    }
    let empty = request
        .query(input("empty", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request
        .query(input("empty", ""))
        .rebuild_index(&empty)
        .unwrap();
    *calls.foreign_candidate.lock().unwrap() = Some(foreign);
    let error = request
        .query(input("empty", ""))
        .execute()
        .err()
        .expect("genuine foreign Source slot refuses");
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(error) = error else {
        panic!("{error:?}")
    };
    assert!(
        matches!(
            error.cause(),
            WorthQueryManagedIndexDenialCause::Work(WorthQueryManagedIndexWorkDenial::Controlled(
                WorthQueryManagedIndexStop::ForeignSourceSlot
            ))
        ),
        "{error:?}"
    );
    *calls.foreign_candidate.lock().unwrap() = None;
    assert!(request
        .query(input("empty", ""))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    assert_eq!(empty.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    empty.close();
    index.close();
}
