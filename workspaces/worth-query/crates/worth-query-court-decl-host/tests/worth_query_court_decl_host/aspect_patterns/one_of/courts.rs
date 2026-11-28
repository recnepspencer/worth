use super::*;
#[test]
fn class_transition_is_atomic_and_invalid_final_states_do_not_publish() {
    let dir = worth_query_testkit_document_retention::facade::directory_home::unique_directory(
        "one-of-transition",
    );
    let host = world::open_directory(dir);
    assert_eq!(
        world::read(&host),
        query::Row {
            class: 1,
            first: Some(11),
            second: None,
            required: None,
            third: None
        }
    );
    for (change, case, key) in [
        (
            authoring::Change::RemoveNamedOnly,
            First::reference().name(),
            "remove-active",
        ),
        (
            authoring::Change::AddSecondOnly,
            Second::reference().name(),
            "activate-sibling",
        ),
    ] {
        let before = world::pin(&host);
        let outcome = world::try_edit(&host, change, key);
        let Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Handler(denial),
        )) = outcome
        else {
            panic!("raw candidate authoring must refuse: {outcome:?}");
        };
        let denial = denial
            .downcast::<WorthQueryApplicationAttemptDenial>()
            .unwrap();
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep
        );
        assert!(denial
            .subject()
            .ends_with(&format!(".{case} requires an atomic case transition")));
        assert_eq!(
            before.selected_commit(),
            world::pin(&host).selected_commit()
        );
        assert_eq!(
            world::read(&host),
            query::Row {
                class: 1,
                first: Some(11),
                second: None,
                required: None,
                third: None
            }
        );
    }
    let outcome = world::edit(&host, authoring::Change::Second, "valid");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    drop(outcome);
    assert_eq!(
        world::read(&host),
        query::Row {
            class: 2,
            first: None,
            second: Some(22),
            required: Some(220),
            third: None
        }
    );
    let dir = host.checkpoint_and_close_for_reopen();
    let host = world::open_directory(dir);
    assert_eq!(
        world::read(&host),
        query::Row {
            class: 2,
            first: None,
            second: Some(22),
            required: Some(220),
            third: None
        }
    );
}

#[test]
fn directional_operations_refuse_the_wrong_source_class_without_publication() {
    let host = world::install();
    let before = world::pin(&host);
    let outcome = world::restore_variant(
        &host,
        query::Row {
            class: 1,
            first: Some(11),
            second: None,
            required: None,
            third: None,
        },
        "wrong-inverse-source",
    );
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::DomainDenied(authoring::Denial::WrongSourceClass)
        ),
        "{outcome:?}"
    );
    assert_eq!(
        before.selected_commit(),
        world::pin(&host).selected_commit()
    );
    assert!(matches!(
        world::edit(&host, authoring::Change::Second, "right-source"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let before = world::pin(&host);
    let outcome = world::edit(&host, authoring::Change::Second, "wrong-forward-source");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::DomainDenied(authoring::Denial::WrongSourceClass)
        ),
        "{outcome:?}"
    );
    assert_eq!(
        before.selected_commit(),
        world::pin(&host).selected_commit()
    );
    assert_eq!(
        world::read(&host),
        query::Row {
            class: 2,
            first: None,
            second: Some(22),
            required: Some(220),
            third: None
        }
    );
}
