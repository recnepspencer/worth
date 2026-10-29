pub(crate) use worth_query_worlds::facade::certification::native_graph_input_guards::retiring_evidence::*;

#[test]
fn explicit_retirement_deletes_only_the_predecessor_while_ordinary_replacement_retains_it() {
    let app = install();
    let first = retire(&app, input("row-a", "first", 10), 300);
    assert!(matches!(first, Outcome::Committed { .. }), "{first:?}");
    assert_eq!(
        (current(&app, "row-a"), snapshot(&app, "first")),
        ("first".into(), Some(10))
    );
    keep(&app, "row-a", "second", 20, 301);
    keep(&app, "row-mismatch", "foreign", 99, 302);
    assert_eq!(
        (current(&app, "row-a"), snapshot(&app, "first")),
        ("second".into(), Some(10))
    );
    let original = input("row-a", "third", 30);
    let receipt = match retire(&app, original.clone(), 303) {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("retirement: {other:?}"),
    };
    assert_eq!(
        (
            snapshot(&app, "second"),
            snapshot(&app, "third"),
            row_value(&app, "row-a")
        ),
        (None, Some(30), 30)
    );
    assert!(matches!(
        retire(&app, input("row-a", "fourth", 40), 304),
        Outcome::Committed { .. }
    ));
    match retire(&app, original, 303) {
        Outcome::AlreadyCommitted(replay) => {
            assert_eq!(replay.outcome_identity(), receipt.outcome_identity())
        }
        other => panic!("original performed replay: {other:?}"),
    }
    assert_eq!(
        (
            current(&app, "row-a"),
            snapshot(&app, "third"),
            snapshot(&app, "fourth"),
            snapshot(&app, "first"),
            snapshot(&app, "foreign")
        ),
        ("fourth".into(), None, Some(40), Some(10), Some(99))
    );
}

#[test]
fn empty_and_existing_selection_races_publish_no_losing_creation_write_or_delete() {
    for existing in [false, true] {
        let app = install();
        if existing {
            keep(&app, "row-a", "old", 10, 310);
        }
        let scope = request_scope();
        let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
        let mut request = app
            .request(&principal, &scope)
            .advanced()
            .mutate(input("row-a", "loser", 42))
            .without_source()
            .idempotency(&311);
        let prepared = match request
            .prepare_in_program(&app, Allocation::SystemAllocation)
            .unwrap()
        {
            Preparation::Prepared(candidate) => candidate,
            Preparation::Settled(_) => panic!("fresh selection"),
        };
        let winner = retire(&app, input("row-a", "winner", 43), 312);
        assert!(matches!(winner, Outcome::Committed { .. }), "{winner:?}");
        let loser = prepared.commit(Allocation::SystemAllocation);
        assert!(
            matches!(
                loser,
                Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
            ),
            "{loser:?}"
        );
        assert_eq!(
            (
                current(&app, "row-a"),
                snapshot(&app, "loser"),
                snapshot(&app, "winner"),
                snapshot(&app, "old"),
                row_value(&app, "row-a")
            ),
            ("winner".into(), None, Some(43), None, 43)
        );
    }
}
