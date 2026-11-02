//! Immutable history reads outlive full retirement of their original source branch.
use super::*;

fn after_complete_retirement(edited_source: bool, key: u64) {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let (document, source, expected) = if edited_source {
        let (document, source) = finish(
            &host,
            ready(&host, begin(&host, document, DOCUMENT_IDENTITY, 3, key)),
            3,
        );
        (document, source, 3)
    } else {
        (document, root.clone(), SEED_RETENTION)
    };
    let first = document.preview(&request, source.id()).unwrap();
    let survivor = document.preview(&request, source.id()).unwrap();
    let (document, current) = finish(
        &host,
        ready(&host, begin(&host, document, DOCUMENT_IDENTITY, 5, key + 1)),
        5,
    );
    let before = survivor.read(&document, &request).unwrap();
    assert_eq!(before.rows()[0].retention_days, expected);
    assert_eq!(before.source(), source.endpoint());
    before.close();
    let retired = host
        .with_host_advancement(|phase| {
            host.on_branch(source.endpoint().product_branch())
                .close(&phase)
        })
        .unwrap()
        .expect("shared ancestry permits complete source retirement");
    assert!(retired.is_complete());
    assert!(host.branches().pending_cleanup().is_empty());
    first.close();
    let historical = survivor.read(&document, &request).unwrap();
    assert_eq!(historical.rows()[0].identity, DOCUMENT_IDENTITY);
    assert_eq!(historical.rows()[0].retention_days, expected);
    assert_eq!(historical.source(), source.endpoint());
    assert_eq!(document.current_step(&request).unwrap(), current);
    assert_eq!(document.read(&request).unwrap().rows()[0].retention_days, 5);
    assert_eq!(
        untouched.read(&request).unwrap().rows()[0].retention_days,
        SEED_RETENTION
    );
    historical.close();
    survivor.close();

    // The source was deliberately retired through the separate public owner
    // entry. Drop its stale document wrapper, then close each remaining actual
    // branch once, as in the existing retirement courts.
    drop(document);
    if edited_source {
        host.with_host_advancement(|phase| {
            host.on_branch(root.endpoint().product_branch())
                .close(&phase)
                .unwrap()
        })
        .unwrap();
    }
    host.with_host_advancement(|phase| {
        host.on_branch(current.endpoint().product_branch())
            .close(&phase)
            .unwrap()
    })
    .unwrap();
    close(&host, untouched, 1);
    assert!(host.branches().pending_cleanup().is_empty());
}

#[test]
fn an_inherited_root_preview_remains_readable_after_its_branch_fully_retires() {
    after_complete_retirement(false, 0x9182_6100);
}

#[test]
fn an_edited_source_preview_remains_readable_after_its_branch_fully_retires() {
    after_complete_retirement(true, 0x9182_6110);
}
