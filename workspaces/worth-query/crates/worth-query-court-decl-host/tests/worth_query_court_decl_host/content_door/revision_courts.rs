use super::*;
#[test]
fn content_recorded_inverse_and_reapplication_preserve_exact_source_bytes() {
    let host = world::install();
    assert!(matches!(
        world::edit(&host, 18, vec![7, 11], b"alpha".to_vec(), "alpha"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let read = || source::query::ReadState {
        root: "alpha".into(),
    };
    let original = request.query(read()).execute().unwrap().rows()[0].content;
    let result = request
        .advanced()
        .mutate(revision::Revise {
            key: "alpha".into(),
            samples: vec![1, 2],
            reference: vec![],
        })
        .without_source()
        .idempotency(&"revision".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = result else {
        panic!("{result:?}")
    };
    let changed = request.query(read()).execute().unwrap().rows()[0].content;
    assert_ne!(changed, original);
    // Python hashlib/OpenSSL over literal little-endian frames, independent of the production writer/hasher.
    assert_eq!(
        original.to_string(),
        "7bcb0e089d83cadc112d673d0f68e3200fdde9f3b4b46626f60e6f83667d165a"
    );
    assert_eq!(
        changed.to_string(),
        "c40db3f9f76eab9977a59ff05cdf9ebe3b6fb892c1ef77a8bb04744861d5bdef"
    );
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("undo")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("fresh inverse must prepare")
    };
    let undo = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt: undo, .. } = undo else {
        panic!("{undo:?}")
    };
    assert_eq!(
        request.query(read()).execute().unwrap().rows()[0].content,
        original
    );
    let page = request
        .corrections()
        .alternatives(&selected, limits, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert!(page.continuation().is_none());
    assert_eq!(page.entries().len(), 1);
    assert_eq!(
        page.entries()[0].performed_commit(),
        undo.commit_reference()
    );
    let chosen = page.select(&page.entries()[0], limits).unwrap();
    let prepared = request
        .corrections()
        .reapply(&chosen)
        .idempotency("reapply")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("retained source input must prepare")
    };
    let result = prepared.publish().unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    assert_eq!(
        request.query(read()).execute().unwrap().rows()[0].content,
        changed
    );
    // Expected IDs come from independent literal payloads, not an inverse that re-encodes old data.
    let alpha = request
        .query(source::query::ReadSource {
            root: "record".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(alpha.rows().len(), 1);
    assert_eq!(alpha.rows()[0].content, changed);
}

#[test]
fn content_original_output_closure_restores_bytes_absent_from_current_fields() {
    let host = world::install();
    assert!(matches!(
        world::edit(&host, 0, vec![3], vec![], "root"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let issued = request
        .advanced()
        .mutate(created_content::Issue {
            key: "issued".into(),
            samples: vec![7, 11],
        })
        .without_source()
        .idempotency(&"issue".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = issued else {
        panic!("{issued:?}")
    };
    let original = request
        .query(source::query::ReadState {
            root: "issued".into(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .content;
    assert_eq!(
        original.to_string(),
        "7bcb0e089d83cadc112d673d0f68e3200fdde9f3b4b46626f60e6f83667d165a"
    );
    assert!(matches!(
        world::edit(&host, 19, vec![1, 2], b"issued".to_vec(), "replace-issued"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_ne!(
        request
            .query(source::query::ReadState {
                root: "issued".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .content,
        original
    );
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("capture-original")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("declared original output must prepare")
    };
    let result = prepared.publish().unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    let restored = world::observe(&host, "observe-original").unwrap();
    assert_eq!(restored.0, original);
    assert_eq!(
        restored.1,
        [1, 0, 0, 0, 2, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0]
    );
}
