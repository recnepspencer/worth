//! Actual World obligations distinguish controller release from hidden source pins.
use super::*;

#[test]
fn the_last_preview_releases_the_original_world_obligation_while_the_document_stays_live() {
    let host = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let (next, source) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_6500),
        ),
        3,
    );
    document = next;
    let first = document.preview(&request, source.id()).unwrap();
    let survivor = document.preview(&request, source.id()).unwrap();
    for (offset, days) in [5, 6].into_iter().enumerate() {
        let (next, _) = finish(
            &host,
            ready(
                &host,
                begin(
                    &host,
                    document,
                    DOCUMENT_IDENTITY,
                    days,
                    0x9182_6501 + offset as u64,
                ),
            ),
            days,
        );
        document = next;
    }
    window(&host, &mut document, 1);
    for _ in 0..16 {
        if document
            .history_retention_status(&request)
            .unwrap()
            .pending_releases
            == 0
        {
            break;
        }
        document.maintain_history(&request).unwrap();
    }
    assert_eq!(
        document
            .history_retention_status(&request)
            .unwrap()
            .pending_releases,
        0
    );
    host.with_host_advancement(|phase| {
        use worth_query_host::facade::product::{
            WorthQueryProductBranchAdmissionDenial as Admission,
            WorthQueryProductBranchAdmissionRefusalKind as Refusal,
        };
        assert!(matches!(
            host.on_branch(source.endpoint().product_branch())
                .select(&phase),
            Err(Admission::Refused(Refusal::RetiredBranch))
        ));
    })
    .unwrap();
    let both = host.world_retention_snapshot_for_test().observations();
    first.close();
    assert_eq!(
        host.world_retention_snapshot_for_test().observations(),
        both
    );
    let read = survivor.read(&document, &request).unwrap();
    assert_eq!(read.rows()[0].retention_days, 3);
    read.close();
    // These inspect the real World capacity obligation. No query, maintenance,
    // descriptor reconstruction or new admission occurs between the two samples.
    let last_holder = host.world_retention_snapshot_for_test().observations();
    survivor.close();
    let released = host.world_retention_snapshot_for_test().observations();
    assert_eq!(
        last_holder.checked_sub(released),
        Some(1),
        "the controller must not retain a hidden all-history source Arc"
    );
    assert_eq!(value(&host, &document), 6);
    close(&host, document, 4);
}
