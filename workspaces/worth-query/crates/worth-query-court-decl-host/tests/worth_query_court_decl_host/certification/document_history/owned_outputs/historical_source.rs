//! Public navigation regenerates a stale output whose source exists only at D.
use super::*;

fn restore_deleted_source(host: &world::Host, revoke: bool) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let original = request
        .query(query::member_public::Read("m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    if revoke {
        restricted::settle(host, &request, "m");
        let document = world::attach(host);
        let document = host
            .with_host_advancement(|phase| {
                document
                    .with_outputs::<program::Program, program::RestrictedTree>(
                        &phase,
                        &request,
                        host,
                        restricted::Demand("m".into()),
                        WorthQueryOutputDemandControls::default(),
                    )
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
            })
            .unwrap();
        exercise_selection(host, document, original, true);
        return;
    }
    child::settle(host, &request, "m");
    let document = world::attach(host);
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_outputs::<program::Program, program::ChildTree>(
                    &phase,
                    &request,
                    host,
                    child::Demand("m".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    exercise_selection(host, document, original, revoke);
}

fn exercise_selection<O: WorthQueryHistoryOutputsMode<Schema, History>>(
    host: &world::Host,
    document: WorthQueryApplicationHistoryDocument<'_, Schema, History, OrdinaryHistoryPayload, O>,
    original: worth_relational::facade::identity::EntityId,
    revoke: bool,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = document
        .edit(member_edit::Intent {
            key: "m".into(),
            value: 6,
        })
        .idempotency_owned(&request, "change-before-delete".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, program::MemberEditAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    if let WorthQueryHistoryEditOutcome::NotRecorded { terminal, .. } = &outcome {
        panic!(
            "not recorded: {:?} {:?}",
            terminal.native_outcome(),
            terminal.native_admission_denial()
        );
    }
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    let before_delete = document.current_step(&request).unwrap().id();
    let document = if revoke {
        let mut edit = document
            .edit(edit::Intent {
                key: "a".into(),
                value: 2,
            })
            .idempotency_owned(&request, "revoke-source-permission".to_owned())
            .unwrap()
            .retain_in_program::<program::Program, program::EditAction>(host)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        loop {
            edit = match edit.advance(&request) {
                WorthQueryHistoryEditAdvance::Progress(next) => next,
                WorthQueryHistoryEditAdvance::Ready(next) => {
                    edit = next;
                    break;
                }
                WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            };
        }
        let WorthQueryHistoryEditFinish { document, outcome } = edit
            .finish(&request)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        assert!(matches!(
            outcome,
            WorthQueryHistoryEditOutcome::Recorded { .. }
        ));
        drop(outcome);
        document
    } else {
        document
    };
    let mut deletion = document
        .edit(member_delete::Intent)
        .idempotency_owned(&request, "delete-source".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, program::DeleteMemberAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        deletion = match deletion.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                deletion = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = deletion
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    if let WorthQueryHistoryEditOutcome::NotRecorded { terminal, .. } = &outcome {
        panic!(
            "not recorded: {:?} {:?}",
            terminal.native_outcome(),
            terminal.native_admission_denial()
        );
    }
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    let current = document.selection(&request).unwrap();
    let on_current = host
        .request(&principal, &scope)
        .on_branch(current.physical_endpoint().product_branch());
    assert!(
        on_current
            .query(query::member::Read("m".into()))
            .execute()
            .is_err(),
        "the source must really be absent at C"
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    if revoke {
        let denial = courts::checkout(
            &document,
            &request,
            before_delete,
            current.clone(),
            0xd1207_02,
        )
        .expect_err("D must not lend its restored grants to this C-denied principal");
        let (
            "advance",
            WorthQueryHistoryNavigationAdvanceDenial::Outputs(
                WorthQueryHistoryOutputDenial::Source(
                    WorthQueryHistoryOutputReadDenial::Admission(denial),
                ),
            ),
        ) = denial
        else {
            panic!("{denial:?}")
        };
        assert!(
            matches!(
                denial.kind(),
                WorthQueryApplicationQueryAdmissionDenialKind::Authorization(_)
            ),
            "{denial:?}"
        );
        assert_eq!(denial.historical_source(), Some((original, "restricted")));
        assert_eq!(
            document.selection(&request).unwrap(),
            current,
            "no head effect"
        );
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
        assert_eq!(
            world::model(host, current.physical_endpoint().product_branch()).value,
            2
        );
        host.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
        return;
    }
    let restored = courts::checkout(&document, &request, before_delete, current, 0xd1207_01)
        .unwrap_or_else(|cause| panic!("restored source fallback: {cause:?}"));
    let restored = host
        .request(&principal, &scope)
        .on_branch(restored.physical_endpoint().product_branch());
    let source = restored
        .query(query::member::Read("m".into()))
        .execute()
        .unwrap();
    assert_eq!(
        (source.rows()[0].native_id, source.rows()[0].value),
        (original, 6)
    );
    let output = restored
        .query(query::child_output::Read("child:m".into()))
        .execute()
        .unwrap();
    assert_eq!(
        output.rows()[0].value,
        600,
        "fallback reads restored E, not the old source value 5"
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

#[test]
fn navigation_regenerates_from_a_source_absent_at_c_and_live_at_d() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    restore_deleted_source(&world::install(), false);
}

#[test]
fn worker_only_navigation_regenerates_from_a_source_absent_at_c_and_live_at_d() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    restore_deleted_source(&world::install_worker_only(), false);
}

#[test]
fn navigation_cannot_borrow_restored_grants_for_a_principal_denied_at_c() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    restore_deleted_source(&world::install(), true);
}
