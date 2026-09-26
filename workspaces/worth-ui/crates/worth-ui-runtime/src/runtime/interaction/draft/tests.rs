use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostObservationSequence, UiHostObservationTimeBasis,
    UiHostPresentationEpoch, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

use super::model::{
    UiActiveLocalRecipient, UiDraftLifecycleCounters, UiDraftProcessingOutcome,
    UiDraftRuntimeState, UiDraftSession,
};
use super::{
    UiDraftByteBudget, UiDraftFieldIdentity, UiDraftMutationKind, UiDraftSessionIdentity,
    UiLocalInputStopReason, UI_DRAFT_SESSION_LIMIT,
};
use crate::runtime::interaction::UiSemanticInteraction;

struct DraftFixture {
    state: UiDraftRuntimeState,
    session: UiDraftSessionIdentity,
}

struct DraftTestPayload;

impl crate::capability::UiIntentPayload for DraftTestPayload {
    const SCHEMA: crate::capability::UiIntentSchema =
        crate::capability::UiIntentSchema::stable("worth_ui.runtime.draft_test", 1);
    const FIELDS: crate::capability::UiIntentPayloadFieldSet =
        crate::capability::UiIntentPayloadFieldSet::EMPTY;

    fn project(
        _fields: &mut crate::capability::UiIntentPayloadProjection<Self>,
    ) -> Result<Self, crate::capability::UiIntentPayloadProjectionViolation> {
        Ok(Self)
    }
}

struct ExpectedDraftMutation {
    source_sequence: u64,
    draft_revision: u64,
    committed_bytes: usize,
    preedit_bytes: usize,
}

#[test]
fn interaction_committed_text_and_unicode_backspace_preserve_scalar_boundaries() {
    let mut fixture = draft_fixture(32);

    let appended = append_text(&mut fixture, 1, 1, "A🦀");
    assert_mutation(
        appended,
        ExpectedDraftMutation {
            source_sequence: 1,
            draft_revision: 1,
            committed_bytes: 5,
            preedit_bytes: 0,
        },
    );

    let deleted = fixture
        .state
        .backspace(fixture.session, sequence(2))
        .expect("the final Unicode scalar is present");
    assert_mutation(
        deleted,
        ExpectedDraftMutation {
            source_sequence: 2,
            draft_revision: 2,
            committed_bytes: 1,
            preedit_bytes: 0,
        },
    );
}

#[test]
fn interaction_preedit_stays_out_of_commit_and_blocks_keyboard_mutation() {
    let mut fixture = draft_fixture(64);
    let presentation = fixture
        .state
        .sessions
        .get(&fixture.session)
        .expect("draft fixture is live")
        .target
        .presentation();

    prime_composition(&mut fixture);

    assert_unsettled_composition_stop(
        fixture
            .state
            .backspace(fixture.session, sequence(3))
            .expect("active composition returns an exact stop"),
    );
    assert_unsettled_composition_stop(fixture.state.commit(
        fixture.session,
        presentation,
        sequence(3),
        monotonic_time(3),
    ));
    assert_eq!(fixture.state.snapshot().active_sessions, 1);

    assert_mutation(
        fixture
            .state
            .cancel_preedit(fixture.session, sequence(4), 3),
        ExpectedDraftMutation {
            source_sequence: 4,
            draft_revision: 3,
            committed_bytes: 4,
            preedit_bytes: 0,
        },
    );
    let committed = fixture.state.commit(
        fixture.session,
        presentation,
        sequence(5),
        monotonic_time(5),
    );
    let UiDraftProcessingOutcome::Semantic(UiSemanticInteraction::EditCommit(commit)) = committed
    else {
        panic!("declared commit gesture must seal one edit interaction")
    };
    assert_eq!(commit.committed_text(), "base");
    assert_eq!(commit.draft_revision(), 3);
    assert_eq!(fixture.state.snapshot().active_sessions, 0);
}

fn prime_composition(fixture: &mut DraftFixture) {
    assert_mutation(
        append_text(fixture, 1, 1, "base"),
        ExpectedDraftMutation {
            source_sequence: 1,
            draft_revision: 1,
            committed_bytes: 4,
            preedit_bytes: 0,
        },
    );
    assert_mutation(
        fixture.state.apply_preedit(
            fixture.session,
            sequence(2),
            2,
            worth_ui_host_contract::UiHostImePreedit::from_unicode_scalar_range("仮", None)
                .expect("fixture preedit is canonical"),
        ),
        ExpectedDraftMutation {
            source_sequence: 2,
            draft_revision: 2,
            committed_bytes: 4,
            preedit_bytes: 3,
        },
    );
}

#[test]
fn interaction_revision_gap_settles_draft_once() {
    let mut fixture = draft_fixture(32);
    let first = append_text(&mut fixture, 1, 10, "a");
    assert_mutation(
        first,
        ExpectedDraftMutation {
            source_sequence: 1,
            draft_revision: 1,
            committed_bytes: 1,
            preedit_bytes: 0,
        },
    );

    let stopped = append_text(&mut fixture, 2, 12, "b");
    let stop = expect_stop(stopped);
    assert_eq!(
        stop.reason(),
        UiLocalInputStopReason::InputRevisionDiscontinuity {
            previous: 10,
            observed: 12,
        }
    );
    assert!(stop.settled_recipient());
    assert!(stop.settled_session());
    assert!(fixture
        .state
        .cancel_all(UiLocalInputStopReason::Shutdown)
        .is_empty());
    assert_eq!(fixture.state.snapshot().counters.sessions_settled, 1);
}

#[test]
fn interaction_draft_byte_overflow_settles_draft_once() {
    let mut fixture = draft_fixture(4);
    assert_mutation(
        append_text(&mut fixture, 1, 1, "éé"),
        ExpectedDraftMutation {
            source_sequence: 1,
            draft_revision: 1,
            committed_bytes: 4,
            preedit_bytes: 0,
        },
    );

    let overflow = append_text(&mut fixture, 2, 2, "x");
    let stop = expect_stop(overflow);
    assert_eq!(
        stop.reason(),
        UiLocalInputStopReason::DraftByteBudgetExceeded {
            limit: 4,
            attempted: 5,
        }
    );
    assert!(stop.settled_session());
    assert!(fixture
        .state
        .cancel_all(UiLocalInputStopReason::Shutdown)
        .is_empty());
}

#[test]
fn interaction_recipient_replacement_suspends_then_shutdown_settles_draft() {
    let mut fixture = draft_fixture(32);
    let displaced = fixture
        .state
        .suspend_active(UiLocalInputStopReason::RecipientReplaced)
        .expect("the active recipient is displaced");

    assert_eq!(
        displaced.reason(),
        UiLocalInputStopReason::RecipientReplaced
    );
    assert!(displaced.settled_recipient());
    assert!(!displaced.settled_session());
    assert_eq!(fixture.state.snapshot().active_recipients, 0);
    assert_eq!(fixture.state.snapshot().active_sessions, 1);

    let shutdown = fixture.state.cancel_all(UiLocalInputStopReason::Shutdown);
    assert_eq!(shutdown.len(), 1);
    assert!(shutdown[0].settled_session());
    assert_eq!(fixture.state.snapshot().counters.sessions_settled, 1);
    assert!(fixture
        .state
        .cancel_all(UiLocalInputStopReason::Shutdown)
        .is_empty());
}

#[test]
fn interaction_draft_session_capacity_is_hard_bounded() {
    let mut state = UiDraftRuntimeState::new();
    let generation = generation();
    let target = target_view();
    let budget = UiDraftByteBudget::new(1).expect("one byte is a valid draft budget");
    for slot in 0..UI_DRAFT_SESSION_LIMIT {
        state
            .create_session(
                target,
                &generation,
                draft_field(u8::try_from(slot).unwrap(), 1),
                budget,
            )
            .expect("capacity admits the declared bound");
    }

    let denial = state
        .create_session(
            target,
            &generation,
            draft_field(u8::try_from(UI_DRAFT_SESSION_LIMIT).unwrap(), 1),
            budget,
        )
        .expect_err("the first session past the bound must stop");
    assert_eq!(
        denial,
        super::UiLocalInputRecipientBindingStopReason::DraftCapacityExceeded {
            limit: UI_DRAFT_SESSION_LIMIT,
        }
    );
    assert_eq!(state.snapshot().active_sessions, UI_DRAFT_SESSION_LIMIT);
}

fn draft_fixture(byte_budget: usize) -> DraftFixture {
    let mut state = UiDraftRuntimeState::new();
    let session = UiDraftSessionIdentity::mint(1);
    let target = target_view();
    state.next_identity = Some(2);
    state.sessions.insert(
        session,
        UiDraftSession {
            target,
            generation: generation(),
            field: draft_field(1, byte_budget),
            budget: UiDraftByteBudget::new(byte_budget).expect("fixture budget is valid"),
            committed: String::new(),
            preedit: None,
            last_input_revision: None,
            draft_revision: 0,
        },
    );
    state.active = Some(UiActiveLocalRecipient::Draft(session));
    state.counters = UiDraftLifecycleCounters {
        recipients_bound: 1,
        sessions_started: 1,
        ..Default::default()
    };
    DraftFixture { state, session }
}

fn generation() -> crate::runtime::WorthUiActiveApplicationGenerationIdentity {
    crate::runtime::tests::active_application_session_test_support::source_backed_component_session(
    )
    .active_generation_identity()
}

fn draft_field(slot: u8, byte_budget: usize) -> UiDraftFieldIdentity {
    UiDraftFieldIdentity::from_payload_field(crate::capability::UiIntentPayloadField::<
        DraftTestPayload,
        crate::capability::UiIntentText,
    >::text(slot, "runtime.draft-test", byte_budget))
}

fn target_view() -> crate::runtime::interaction::UiPresentedInteractionTargetView {
    let binding = UiSurfaceBindingGeneration::mint_unbound().expect("binding identity capacity");
    let presentation = UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound()
            .expect("host surface identity capacity"),
        UiMountedFrameIdentity::mint_unbound().expect("frame identity capacity"),
        binding,
        UiHostPresentationEpoch::issued_by_host(1),
    );
    crate::runtime::interaction::targeting::interaction_target_view_for_test(
        presentation,
        crate::mounting::UiMountedInteractionAffinityInput {
            surface: UiSemanticSurfaceIdentity::mint_unbound().expect("surface identity capacity"),
            binding,
            mounted_instance: UiMountedInstanceIdentity::mint_unbound()
                .expect("instance identity capacity"),
            node_receipt: UiMountedNodeReceiptIdentity::mint_unbound()
                .expect("node receipt identity capacity"),
        },
    )
}

fn sequence(value: u64) -> UiHostObservationSequence {
    UiHostObservationSequence::new(value)
}

fn monotonic_time(value: u64) -> UiHostObservationTimeBasis {
    UiHostObservationTimeBasis::HostMonotonicMillis(value)
}

fn append_text(
    fixture: &mut DraftFixture,
    source_sequence: u64,
    revision: u64,
    text: &str,
) -> UiDraftProcessingOutcome {
    fixture.state.apply_committed_text(
        fixture.session,
        super::mutation::UiCommittedTextMutation {
            sequence: sequence(source_sequence),
            revision,
            text,
            kind: UiDraftMutationKind::CommittedText,
        },
    )
}

fn assert_mutation(outcome: UiDraftProcessingOutcome, expected: ExpectedDraftMutation) {
    let UiDraftProcessingOutcome::Mutation(receipt) = outcome else {
        panic!("fixture operation must produce a draft mutation")
    };
    assert_eq!(receipt.source_sequence().value(), expected.source_sequence);
    assert_eq!(receipt.draft_revision(), expected.draft_revision);
    assert_eq!(receipt.committed_utf8_bytes(), expected.committed_bytes);
    assert_eq!(receipt.preedit_utf8_bytes(), expected.preedit_bytes);
}

fn assert_unsettled_composition_stop(outcome: UiDraftProcessingOutcome) {
    let stop = expect_stop(outcome);
    assert_eq!(stop.reason(), UiLocalInputStopReason::CompositionActive);
    assert!(!stop.settled_recipient());
    assert!(!stop.settled_session());
}

fn expect_stop(outcome: UiDraftProcessingOutcome) -> super::UiLocalInputStop {
    let UiDraftProcessingOutcome::Stopped(stop) = outcome else {
        panic!("fixture operation must produce an exact stop")
    };
    stop
}
