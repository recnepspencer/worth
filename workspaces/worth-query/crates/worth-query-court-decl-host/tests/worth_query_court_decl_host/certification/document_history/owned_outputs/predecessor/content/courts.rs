//! Old bytes and the denied-field twin cross the real producer loan boundary.
use super::*;
#[test]
fn predecessor_content_keeps_old_bytes_and_current_retry_reuses_the_output() {
    journey(true);
}
#[test]
fn predecessor_content_refuses_an_undeclared_field_without_exposing_bytes() {
    journey(false);
}
fn journey(granted: bool) {
    let host = install(false);
    super::super::courts::reset(false);
    PROBE.with(|probe| probe.borrow_mut().content_mode = Some(true));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    super::super::courts::settle(&request, &host).unwrap();
    edit_source(&host, &request, 2);
    super::super::courts::settle(&request, &host).unwrap();
    PROBE.with(|probe| assert_eq!(probe.borrow().content_values.as_slice(), &[None]));
    edit_source(&host, &request, 3);
    PROBE.with(|probe| probe.borrow_mut().content_mode = Some(granted));
    let result = super::super::courts::settle(&request, &host);
    if granted {
        result.unwrap();
        PROBE.with(|probe| {
            assert_eq!(
                probe.borrow().content_values.last(),
                Some(&Some(2u64.to_le_bytes().to_vec()))
            );
            assert!(!probe.borrow().content_denied);
        });
        let count = PROBE.with(|probe| probe.borrow().content_values.len());
        super::super::courts::settle(&request, &host).unwrap();
        PROBE.with(|probe| assert_eq!(probe.borrow().content_values.len(), count));
    } else {
        result.unwrap();
        PROBE.with(|probe| {
            let probe = probe.borrow();
            assert!(
                probe.content_denied,
                "the independently authorized operation has no content field grant"
            );
            assert_eq!(
                probe.content_values.as_slice(),
                &[None],
                "the refused read must expose no old bytes"
            );
        });
    }
    let output = request
        .query(query::first_output::Read("predecessor:a".into()))
        .execute()
        .unwrap();
    assert_eq!(output.rows()[0].value, 9003);
}

fn edit_source(host: &Host, request: &family::Request<'_, '_, '_>, value: u64) {
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value,
        })
        .idempotency(&format!("content-{value}"))
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
}
