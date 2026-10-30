pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_approval::instance_deadline::*;

#[test]
fn a_step_is_refused_once_the_deadline_is_reached_but_its_replay_answers() {
    let (time, application, _, instance) = deadline_instance(93_000);
    time.advance(DEADLINE - Duration::from_millis(1));
    let proposal = match propose_instance(&application, instance.clone(), 93_002)
        .expect("a proposal a millisecond early prepares")
    {
        WorkflowProposalOutcome::Published(performed) => performed,
        other => panic!("a proposal before the deadline did not publish: {other:?}"),
    };
    assert!(!proposal.replayed());

    time.advance(Duration::from_millis(1));
    warm_and_cold(&application, |state| {
        match propose_instance(&application, instance.clone(), 93_002) {
            Ok(WorkflowProposalOutcome::Published(replay)) => {
                assert!(replay.replayed(), "the {state} retry replays");
                assert_eq!(replay.transition(), proposal.transition());
            }
            other => panic!("the {state} retry after the deadline must replay: {other:?}"),
        }
        assert_eq!(
            advance_denial(advance_instance(&application, instance.clone(), 93_003)),
            ELAPSED,
            "the {state} next step is refused at the deadline itself",
        );
    });
    let ended = cancelled(cancel_instance(&application, instance, 93_004));
    assert!(!ended.replayed(), "an overdue instance still cancels");
}

#[test]
fn a_fresh_proposal_after_the_deadline_names_it() {
    let (time, application, _, instance) = deadline_instance(93_100);
    time.advance(DEADLINE);
    warm_and_cold(&application, |state| {
        assert_eq!(
            step_denial(propose_instance(&application, instance.clone(), 93_102)),
            ELAPSED,
            "the {state} proposal is refused",
        );
    });
}

#[test]
fn an_approval_or_back_after_the_deadline_is_refused() {
    let (time, application, _, instance) = deadline_instance(93_200);
    let (proposal, required, _) = approval_requirement(&application, instance.clone(), 93_200);
    time.advance(DEADLINE);
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    warm_and_cold(&application, |state| {
        let back = runtime
            .request(&principal, &scope)
            .mutate(
                super::super::document_retention_model::workflow::WorkflowAdvanceIntent {
                    input: super::super::document_retention_model::workflow::WorkflowAdvanceInput {
                        document_identity: DOCUMENT_IDENTITY.to_owned(),
                    },
                },
            )
            .without_source()
            .idempotency(&93_212_u64)
            .prepare_workflow_navigate_back(&application, instance.clone())
            .expect("Back receives fresh admission")
            .execute();
        assert!(
            matches!(
                back,
                Err(WorkflowProgressOutcome::PreparationDenied(ref denial))
                    if denial.kind() == ELAPSED
            ),
            "the {state} Back is refused: {back:?}",
        );
        assert_eq!(
            advance_denial(approve_instance(
                &application,
                instance.clone(),
                &required,
                &proposal,
                WorkflowApprovalDecision::Approve,
                93_210,
            )),
            ELAPSED,
            "the {state} approval is refused",
        );
    });
    let ended = cancelled(cancel_instance(&application, instance, 93_211));
    assert!(!ended.replayed());
}

#[test]
fn a_step_performed_in_time_replays_after_the_deadline() {
    let (time, application, _, instance) = deadline_instance(93_700);
    let (proposal, required, _) = approval_requirement(&application, instance.clone(), 93_700);
    approve(&application, &instance, &required, &proposal, 93_710);
    time.advance(DEADLINE);
    warm_and_cold(&application, |state| {
        match advance_instance(&application, instance.clone(), 93_707) {
            Ok(WorkflowProgressOutcome::Completed(replay)) => {
                assert!(replay.replayed(), "the {state} advance retry replays")
            }
            other => panic!("the {state} advance retry must replay: {other:?}"),
        }
        match approve_instance(
            &application,
            instance.clone(),
            &required,
            &proposal,
            WorkflowApprovalDecision::Approve,
            93_710,
        ) {
            Ok(WorkflowProgressOutcome::Completed(replay)) => {
                assert!(replay.replayed(), "the {state} approval retry replays")
            }
            other => panic!("the {state} approval retry must replay: {other:?}"),
        }
    });
}

#[test]
fn an_operation_admitted_in_time_is_refused_at_commit_after_the_deadline() {
    assert_eq!(
        overdue_operation(93_300, |time| time.advance(DEADLINE)),
        Ok(WorthQueryApplicationCommitDenialKind::WorkflowSettlementDenied { kind: ELAPSED }),
    );
}

#[test]
fn a_successor_keeps_the_earlier_deadline_and_an_overdue_source_cannot_migrate() {
    let (time, application, definition, instance) = deadline_instance(93_400);
    approval_requirement(&application, instance.clone(), 93_400);
    time.advance(DEADLINE / 2);
    let target = replace_definition(
        &application,
        definition,
        reviewed_document_definition_with_deadline("done", DEADLINE),
        93_410,
    );
    let successor = started(migrate_instance(
        &application,
        instance,
        target.clone(),
        "propose",
        93_411,
    ))
    .instance()
    .clone();
    proposal::published_proposal(&application, successor.clone(), 93_412);

    time.advance(DEADLINE / 2);
    warm_and_cold(&application, |state| {
        assert_eq!(
            advance_denial(advance_instance(&application, successor.clone(), 93_413)),
            ELAPSED,
            "the {state} successor spends its source's deadline, not a fresh one",
        );
    });
    let next = replace_definition(
        &application,
        target,
        reviewed_document_definition_with_deadline("finished", DEADLINE),
        93_420,
    );
    assert_eq!(
        migration_denial(migrate_instance(
            &application,
            successor.clone(),
            next.clone(),
            "propose",
            93_421,
        )),
        ELAPSED,
        "an overdue instance cannot restart its deadline by migrating",
    );
    let fork = fork_of(&application, successor.branch());
    assert_eq!(
        migration_denial(continue_on_fork(
            &application,
            fork,
            successor.clone(),
            next,
            "propose",
            93_423,
        )),
        ELAPSED,
        "nor by continuing on a fork",
    );
    let ended = cancelled(cancel_instance(&application, successor, 93_422));
    assert!(!ended.replayed());
}

/// Trusted time that cannot be read admits nothing: authorization refuses a
/// new step, and the approval behind an operation admitted in time, before
/// any deadline is judged against an unknown instant.
#[test]
fn an_unreadable_trusted_clock_admits_no_step() {
    let (time, application, _, instance) = deadline_instance(93_500);
    time.set_unavailable(true);
    assert!(matches!(
        propose_instance(&application, instance.clone(), 93_502),
        Err(WorthQueryWorkflowProposalPreparationDenial::RequestAdmission(
            WorthQueryApplicationRequestMutationDenial::Authorization(ref denial)
        )) if denial.kind() == WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::TrustedTimeUnavailable)
    ));
    time.set_unavailable(false);
    match propose_instance(&application, instance, 93_503) {
        Ok(WorkflowProposalOutcome::Published(_)) => {}
        other => panic!("a readable clock admits the step again: {other:?}"),
    }
    assert_eq!(
        overdue_operation(93_600, |time| time.set_unavailable(true)),
        Err(WorthQueryApplicationAttemptDenialKind::WorkflowTrustedTimeUnavailable),
    );
}
