//! Actual cold roots cannot omit Source support or launder duplicate/cyclic structure.
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use crate::application_invariant_acceptance::managed_index::structure_fault::Fault;

#[test]
fn source_tree_refuses_omitted_duplicate_and_cyclic_roots_before_ready() {
    run(journey);
}
fn journey() {
    for (fault, expected) in [
        (
            Fault::OmitSource,
            WorthQueryManagedIndexStop::SourceStructureIncomplete,
        ),
        (
            Fault::DuplicateSource,
            WorthQueryManagedIndexStop::DuplicateSourceSlot,
        ),
        (
            Fault::DuplicateChild,
            WorthQueryManagedIndexStop::DuplicateNodeParent,
        ),
        (
            Fault::DisconnectedCycle,
            WorthQueryManagedIndexStop::SourceStructureCycle,
        ),
    ] {
        let calls = Arc::new(prefix_observation::Calls::default());
        let world = host::install(calls.clone());
        let scope = authentication::request_scope();
        let principal = authenticate(&world, &scope);
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
        assert_eq!(
            source.rows().len(),
            7,
            "the actual universe contains seven distinct occurrences"
        );
        let index = request
            .query(find("al"))
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
        let head = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        *calls.structure_fault.lock().unwrap() = fault;
        let error = request.query(find("al")).rebuild_index(&index).unwrap_err();
        let WorthQueryApplicationRequestQueryDenial::ManagedIndex(error) = error else {
            panic!("structural root admission must refuse: {error:?}");
        };
        assert!(
            matches!(error.cause(), WorthQueryManagedIndexDenialCause::Work(
            WorthQueryManagedIndexWorkDenial::Controlled(actual)) if actual == &expected),
            "{fault:?}: {error:?}"
        );
        assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Failed);
        assert_eq!(calls.totals(), (1, 1, 0));
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &head
        );
        *calls.structure_fault.lock().unwrap() = Fault::None;
        request.query(find("al")).rebuild_index(&index).unwrap();
        assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
        assert_eq!(
            request.query(find("al")).execute().unwrap().rows(),
            journey::expected(source.rows(), "al")
        );
        assert_eq!(calls.totals(), (2, 2, 1));
        index.close();
    }
}
