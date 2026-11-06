use super::*;
#[test]
fn class_transition_is_atomic_and_invalid_final_states_do_not_publish() {
    let host = world::install();
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
    for (n, (change, present)) in [
        (authoring::Change::Zero, 0),
        (authoring::Change::RemoveNamedOnly, 0),
        (authoring::Change::AddSecondOnly, 2),
        (authoring::Change::Two, 2),
    ]
    .into_iter()
    .enumerate()
    {
        let before = world::pin(&host);
        let outcome = world::edit(&host, change, &format!("invalid-{n}"));
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome
        else {
            panic!("must reach installed rule: {outcome:?}");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::CustomInvariantDenied,
            "case {n}: {denial:?}"
        );
        assert_eq!(
            denial
                .decode_invariant_refusal::<rule::RefusalBinding>()
                .unwrap(),
            rule::Refusal {
                subject: "record".into(),
                present
            }
        );
        let after = world::pin(&host);
        assert_eq!(before.selected_commit(), after.selected_commit());
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
    let checkpoint = host
        .runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    drop(host);
    let host = world::restore(checkpoint);
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
fn recorded_inverse_restores_the_selected_class_and_all_payload_fields() {
    let host = world::install();
    let before = world::pin(&host);
    let outcome = world::edit(&host, authoring::Change::Second, "original-transition");
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("transition must commit: {outcome:?}");
    };
    let after = world::pin(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("undo-transition")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("recorded inverse must prepare");
    };
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
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
    drop((before, after));
    let recovered = world::restore(world::checkpoint(&host));
    assert_eq!(
        world::read(&recovered),
        query::Row {
            class: 1,
            first: Some(11),
            second: None,
            required: None,
            third: None
        }
    );
}
#[test]
fn incomplete_present_class_refuses_without_publishing() {
    let host = world::install();
    for (n, change) in [
        authoring::Change::Incomplete,
        authoring::Change::AddIncompleteOnly,
    ]
    .into_iter()
    .enumerate()
    {
        let before = world::pin(&host);
        let outcome = world::edit(&host, change, &format!("incomplete-{n}"));
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome
        else {
            panic!("malformed candidate must refuse: {outcome:?}");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ProviderRejected
        );
        assert_eq!(
            denial.stage(),
            WorthQueryApplicationCommitDenialStage::ProposalBinding
        );
        // Existing public envelope renders the internal InvalidEffectValue and
        // original validator's field subject; it has no typed validation accessor.
        let detail = denial.detail().expect("actual validation subject");
        assert!(
            detail.starts_with("InvalidEffectValue: Record EntityId")
                || detail.starts_with("InvalidEffectValue: Record RecordId"),
            "{detail}"
        );
        assert!(
            detail.contains(&format!(
                ".{}: MissingRequiredField(FieldKey(\"SecondRequired\"))",
                Second::reference().name()
            )),
            "{detail}"
        );
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
}

#[test]
fn inverse_requires_a_complete_first_image() {
    let host = world::install();
    let outcome = world::edit(
        &host,
        authoring::Change::Second,
        "second-before-invalid-restore",
    );
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    let before = world::pin(&host);
    let outcome = world::restore_variant(
        &host,
        query::Row {
            class: 1,
            first: None,
            second: None,
            required: None,
            third: None,
        },
        "invalid-restore",
    );
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::DomainDenied(authoring::Denial::InvalidRestore)
        ),
        "{outcome:?}"
    );
    let after = world::pin(&host);
    assert_eq!(before.selected_commit(), after.selected_commit());
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
