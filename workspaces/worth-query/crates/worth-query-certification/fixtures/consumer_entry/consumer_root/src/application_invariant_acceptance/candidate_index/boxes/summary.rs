//! Schema-bound fixture for candidate_index::boxes.
//! A genuine Source-complete BVH with false leaf summaries cannot become Ready.
use super::super::super::authentication;
use super::super::{host, lifecycle};
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::sync::{atomic::Ordering, Arc};
use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[test]
fn checked_summary_refuses_corrupt_cold_bounds_then_rebuilds_a_lawful_twin() {
    lifecycle::run(journey);
}
#[test]
fn predecessor_lookup_on_a_genuine_cold_source_slot_refuses_before_a_lawful_build() {
    lifecycle::run(|| {
        let calls = Arc::new(Calls::default());
        let world = host::install_boxes(
            Arc::new(prefix_observation::Calls::default()),
            calls.clone(),
        );
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
        let probe = Coordinates::new(2, 2, 2, 2);
        let index = request
            .query(geometry::input("boxes", probe))
            .open_index::<BoxIndex>()
            .unwrap();
        calls.cold_lookup.store(true, Ordering::SeqCst);
        let error = request
            .query(geometry::input("boxes", probe))
            .rebuild_index(&index)
            .unwrap_err();
        let WorthQueryApplicationRequestQueryDenial::ManagedIndex(error) = error else {
            panic!("{error:?}")
        };
        assert!(
            matches!(
                error.cause(),
                WorthQueryManagedIndexDenialCause::Work(
                    WorthQueryManagedIndexWorkDenial::Controlled(
                        WorthQueryManagedIndexStop::SourceStructureUnavailable
                    )
                )
            ),
            "{error:?}"
        );
        assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Failed);
        calls.cold_lookup.store(false, Ordering::SeqCst);
        request
            .query(geometry::input("boxes", probe))
            .rebuild_index(&index)
            .unwrap();
        let source = request
            .query(query::ReadSource {
                directory: "boxes".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            request
                .query(geometry::input("boxes", probe))
                .execute()
                .unwrap()
                .rows(),
            geometry::expected(source.rows(), probe)
        );
        assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
        index.close();
    });
}
fn journey() {
    let calls = Arc::new(Calls::default());
    let world = host::install_boxes(
        Arc::new(prefix_observation::Calls::default()),
        calls.clone(),
    );
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
            directory: "boxes".into(),
        })
        .execute()
        .unwrap();
    let probe = Coordinates::new(2, 2, 2, 2);
    let index = request
        .query(geometry::input("boxes", probe))
        .open_index::<BoxIndex>()
        .unwrap();
    calls.corrupt_summary.store(true, Ordering::SeqCst);
    let error = request
        .query(geometry::input("boxes", probe))
        .rebuild_index(&index)
        .unwrap_err();
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(error) = error else {
        panic!("{error:?}");
    };
    assert!(
        matches!(
            error.cause(),
            WorthQueryManagedIndexDenialCause::Work(WorthQueryManagedIndexWorkDenial::Controlled(
                WorthQueryManagedIndexStop::SummaryMismatch
            ))
        ),
        "{error:?}"
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Failed);
    assert_eq!(calls.totals(), (1, 1, 0));
    calls.corrupt_summary.store(false, Ordering::SeqCst);
    request
        .query(geometry::input("boxes", probe))
        .rebuild_index(&index)
        .unwrap();
    for probe in [
        probe,
        Coordinates::new(100, 100, 101, 101),
        Coordinates::new(-100, -100, 100, 100),
    ] {
        assert_eq!(
            request
                .query(geometry::input("boxes", probe))
                .execute()
                .unwrap()
                .rows(),
            geometry::expected(source.rows(), probe)
        );
    }
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    index.close();
}
