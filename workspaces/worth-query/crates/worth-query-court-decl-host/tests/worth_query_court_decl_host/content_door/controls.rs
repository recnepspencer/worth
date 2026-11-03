//! Public controlled journeys use the same generic content application.
use super::*;
use std::sync::{atomic::Ordering, Arc};

#[test]
fn content_size_refuses_in_native_admission_before_candidate_effects() {
    let calls = Arc::new(source::Calls::default());
    let host = world::install_with_calls(None, calls.clone());
    let before = world::pin(&host);
    let counts = (
        calls.updates.load(Ordering::SeqCst),
        calls.content_reads.load(Ordering::SeqCst),
        calls.records.lock().unwrap().len(),
    );
    // exact-sample-grid is 17 bytes: complete frame = 49 + 8*n.
    let count = (16 * 1024 * 1024 - 49) / 8 + 1;
    let refused =
        world::try_edit(&host, 20, vec![count as i64], vec![], "oversize-frame").unwrap_err();
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(cause),
    ) = refused
    else {
        panic!("{refused:?}")
    };
    let ContentWriteDenial::Content(content) = cause.downcast::<ContentWriteDenial>().unwrap()
    else {
        panic!("Native admission must own the size refusal")
    };
    assert_eq!(
        content,
        RelationalContentDenial::SizeLimit {
            observed: 49 + count * 8,
            maximum: 16 * 1024 * 1024
        }
    );
    assert_eq!(
        world::pin(&host).selected_commit(),
        before.selected_commit()
    );
    assert_eq!(
        (
            calls.updates.load(Ordering::SeqCst),
            calls.content_reads.load(Ordering::SeqCst),
            calls.records.lock().unwrap().len()
        ),
        counts
    );
    assert!(world::observe(&host, "after-size-refusal").is_none());
}

#[test]
fn content_source_cancel_inside_original_borrow_refuses_and_fresh_rebuild_succeeds() {
    let calls = Arc::new(source::Calls::default());
    let host = world::install_with_calls(None, calls.clone());
    let created = world::edit(&host, 18, vec![7, 11], b"alpha".to_vec(), "alpha");
    assert!(matches!(
        created,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let find = || source::query::Find {
        root: "record".into(),
        minimum: 10,
    };
    let index = request.query(find()).open_index::<source::Index>().unwrap();
    *calls.cancel_content.lock().unwrap() = Some(cancellation);
    let before = world::pin(&host);
    let refused = request.query(find()).rebuild_index(&index).unwrap_err();
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(refused) = refused else {
        panic!("{refused:?}")
    };
    assert!(
        matches!(
            refused.cause(),
            WorthQueryManagedIndexDenialCause::Work(WorthQueryManagedIndexWorkDenial::Controlled(
                WorthQueryManagedIndexStop::Request(
                    worth_query_host::facade::runtime::WorkCeilingDenial::Stopped(
                        worth_query_host::facade::runtime::MapKernelStop::Cancelled
                    )
                )
            ))
        ),
        "{refused:?}"
    );
    assert!(
        calls.cancel_content.lock().unwrap().is_none(),
        "cancellation happened inside the actual byte borrow"
    );
    assert_eq!(
        world::pin(&host).selected_commit(),
        before.selected_commit()
    );
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    let fresh = host.runtime().request(&fresh_principal, &fresh_scope);
    let fresh_index = fresh.query(find()).open_index::<source::Index>().unwrap();
    fresh.query(find()).rebuild_index(&fresh_index).unwrap();
    let result = fresh.query(find()).execute().unwrap();
    assert_eq!(
        result
            .rows()
            .iter()
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>(),
        ["alpha"]
    );
    fresh_index.close();
    index.close();
}
