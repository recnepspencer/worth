pub(crate) use worth_query_testkit_certification_operation_body::facade::certification::native_operation_bodies::operation_body::draft::*;

#[test]
fn native_optional_draft_set_empty_text_clear_and_replay() {
    let app = super::install::<20>();
    assert_eq!(super::draft_query::read_note(&app), None);
    assert!(matches!(
        invoke::<SetDraftNote>(&app, 300, ""),
        Outcome::Committed { .. }
    ));
    assert_eq!(super::draft_query::read_note(&app), Some(String::new()));
    assert!(matches!(
        invoke::<SetDraftNote>(&app, 300, ""),
        Outcome::AlreadyCommitted(_)
    ));
    assert!(matches!(
        invoke::<SetDraftNote>(&app, 301, "typed"),
        Outcome::Committed { .. }
    ));
    assert_eq!(
        super::draft_query::read_note(&app),
        Some("typed".to_owned())
    );
    // No optional step means Unchanged, even while another body writes the scope.
    assert!(matches!(
        super::invoke(&app, 303),
        Outcome::Committed { .. }
    ));
    assert_eq!(
        super::draft_query::read_note(&app),
        Some("typed".to_owned())
    );
    assert!(matches!(
        invoke::<ClearDraftNote>(&app, 302, ""),
        Outcome::Committed { .. }
    ));
    assert_eq!(super::draft_query::read_note(&app), None);
    assert!(matches!(
        invoke::<ClearDraftNote>(&app, 302, ""),
        Outcome::AlreadyCommitted(_)
    ));
}

#[test]
fn concurrent_optional_set_from_absence_stales_without_overwriting_the_winner() {
    use worth_query_host::facade::application_entry::WorthQueryApplicationProgramMutationPreparation as Preparation;
    use worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted;
    let app = super::install::<20>();
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let key = 304_u64;
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(DraftIntent::<20, SetDraftNote>::new("loser"))
        .without_source()
        .idempotency(&key);
    let candidate = match request
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh optional body must prepare"),
    };
    assert!(matches!(
        invoke::<SetDraftNote>(&app, 305, "winner"),
        Outcome::Committed { .. }
    ));
    match candidate.commit(ExecutionAllocationPolicy::SystemAllocation) {
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(stale)) => {
            assert!(stale.stale_fact_count() > 0)
        }
        other => panic!("absent optional source must stale: {other:?}"),
    }
    assert_eq!(
        super::draft_query::read_note(&app),
        Some("winner".to_owned())
    );
}
