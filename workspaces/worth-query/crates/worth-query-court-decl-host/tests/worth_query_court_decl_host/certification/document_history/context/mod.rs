//! Context writes may advance physical custody only with actual Native evidence.
use super::recording::{begin, close, finish, ready};
use super::*;
use std::num::{NonZeroU64, NonZeroUsize};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistoryDenial as Failure, WorthQueryDocumentHistoryDenial as Native,
    WorthQueryHistoryContextRefreshDenial as RefreshDenial,
    WorthQueryHistoryContextRefreshLimits as Limits,
};
mod governed;
mod handoff;

fn limits(commits: usize) -> Limits {
    Limits {
        maximum_commits: NonZeroUsize::new(commits).unwrap(),
        maximum_records: NonZeroUsize::new(256).unwrap(),
        maximum_evidence_bytes: NonZeroU64::new(8 * 1024 * 1024).unwrap(),
    }
}
fn outside(
    host: &App,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    status: &str,
    key: u64,
) {
    super::metadata::grant(host, branch, status, key);
}
fn bound(cause: Failure) {
    assert!(
        matches!(
            cause,
            Failure::Document(Native::ContextRefresh(RefreshDenial::BoundExceeded))
        ),
        "actual proof budget must refuse: {cause:?}"
    );
}

#[test]
fn bounds_refuse_without_rebinding_and_larger_original_source_retry_succeeds() {
    let host = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let original = document.current_step(&request).unwrap();
    outside(
        &host,
        original.endpoint().product_branch(),
        "revoked",
        0x9182_b100,
    );
    outside(
        &host,
        original.endpoint().product_branch(),
        "active",
        0x9182_b101,
    );
    for budget in [
        limits(1),
        Limits {
            maximum_records: NonZeroUsize::MIN,
            ..limits(2)
        },
        Limits {
            maximum_evidence_bytes: NonZeroU64::MIN,
            ..limits(2)
        },
    ] {
        bound(document.refresh_context(&request, budget).unwrap_err());
        assert!(matches!(
            document.current_step(&request).unwrap_err(),
            Failure::Document(Native::HeadDiverged)
        ));
    }
    document.refresh_context(&request, limits(2)).unwrap();
    assert_eq!(document.current_step(&request).unwrap(), original);
    assert_eq!(value(&host, &document), 7);
    close(&host, document, 1);
}

#[test]
fn model_aba_and_equal_value_authorship_cannot_be_called_context_only() {
    use worth_query_testkit_document_retention::facade::document_retention_model::record_history::stimuli;
    for (offset, values) in [(0, vec![3, 7]), (1, vec![7])] {
        let host = publish_on_first_program();
        let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let original = document.current_step(&request).unwrap();
        let branch = original.endpoint().product_branch();
        let mut sources = Vec::new();
        for (index, days) in values.into_iter().enumerate() {
            sources.push(stimuli::retention(
                &host,
                branch,
                days,
                0x9182_b200 + offset * 16 + index as u64,
                true,
            ));
        }
        assert_eq!(
            host.request(&principal, &scope)
                .on_branch(branch)
                .query(DocumentRetentionRead {
                    identity: DOCUMENT_IDENTITY.to_owned()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .retention_days,
            7
        );
        let refused = document.refresh_context(&request, limits(4)).unwrap_err();
        assert!(
            matches!(
                refused,
                Failure::Document(Native::ContextRefresh(RefreshDenial::RecordChanged))
            ),
            "authored target, not endpoint value equality: {refused:?}"
        );
        assert!(matches!(
            document.current_step(&request).unwrap_err(),
            Failure::Document(Native::HeadDiverged)
        ));
        drop(sources);
        close(&host, document, 1);
    }
}

#[test]
fn whole_composite_default_requires_its_original_physical_source() {
    use worth_query_testkit_document_retention::facade::document_retention_model::document_history::DetailedRetentionHistory;
    let host = publish_on_first_program();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let owned = head(&host);
    let mut document = host
        .with_host_advancement(|phase| {
            host.request(&principal, &scope)
                .on_branch(owned.branch())
                .attach_history::<DetailedRetentionHistory>(
                    &phase,
                    owned,
                    DocumentRetentionRead {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                )
                .unwrap()
        })
        .unwrap();
    let original = document.current_step(&request).unwrap();
    outside(
        &host,
        original.endpoint().product_branch(),
        "revoked",
        0x9182_b300,
    );
    assert!(matches!(
        document.refresh_context(&request, limits(2)).unwrap_err(),
        Failure::Document(Native::UnsupportedContextRefresh)
    ));
    assert!(matches!(
        document.current_step(&request).unwrap_err(),
        Failure::Document(Native::HeadDiverged)
    ));
    close(&host, document, 1);
}

#[test]
fn refresh_preserves_the_immutable_tree_cursor_and_the_next_edit_parent() {
    use worth_query_host::facade::application_entry::WorthQueryHistoryPageLimits;
    let host = publish_on_first_program();
    let (mut document, child) = finish(
        &host,
        ready(
            &host,
            begin(
                &host,
                attach(&host, head(&host), DOCUMENT_IDENTITY),
                DOCUMENT_IDENTITY,
                3,
                0x9182_b400,
            ),
        ),
        3,
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let page_limits = WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::MIN,
        maximum_examined_entries: NonZeroUsize::MIN,
        maximum_owned_bytes: NonZeroUsize::new(16384).unwrap(),
    };
    let page = document.history_page(&request, page_limits, None).unwrap();
    let root = page.rows()[0].step().clone();
    let cursor = page.continuation().unwrap();
    let preview = document.preview(&request, root.id()).unwrap();
    outside(
        &host,
        child.endpoint().product_branch(),
        "revoked",
        0x9182_b401,
    );
    document.refresh_context(&request, limits(1)).unwrap();
    assert_eq!(document.current_step(&request).unwrap(), child);
    let next = document
        .history_page(&request, page_limits, Some(&cursor))
        .unwrap();
    assert_eq!(next.rows()[0].step(), &child);
    assert_eq!(
        preview.read(&document, &request).unwrap().rows()[0].retention_days,
        7
    );
    let (document, next) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 5, 0x9182_b402),
        ),
        5,
    );
    assert_eq!(next.parent(), Some(child.id()));
    preview.close();
    close(&host, document, 3);
}
