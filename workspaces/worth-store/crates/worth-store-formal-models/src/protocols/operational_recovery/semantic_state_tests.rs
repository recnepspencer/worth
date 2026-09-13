use super::binding::OperationalRecoveryActionBinding as Binding;
use super::semantic_state::OperationalRecoverySemanticState;
use super::{
    OperationalRecoveryAction, OperationalRecoveryActionKind, OperationalRecoveryInvariant,
};

#[test]
fn one_operation_accepts_distinct_staging_and_cutover_authorizations_but_rejects_replay() {
    let mut state = OperationalRecoverySemanticState::default();
    for plan in [id(1), id(3)] {
        state
            .apply(&action(Binding::Authorization {
                plan,
                execution: Some(id(2)),
                replayed: false,
            }))
            .unwrap();
    }
    assert_eq!(
        state.apply(&action(Binding::Authorization {
            plan: id(1),
            execution: Some(id(2)),
            replayed: false,
        })),
        Err(OperationalRecoveryInvariant::AuthorizationReplayRejected)
    );
}

#[test]
fn promotion_requires_exact_fence_plan_receipt_publication_and_epoch_continuity() {
    let mut state = authorized_state();
    state
        .apply(&action(Binding::PromotionFence {
            authorization_plan: id(1),
            execution_plan: id(2),
            fence: id(3),
            epoch: 8,
        }))
        .unwrap();

    let denial = state.apply(&action(Binding::PromotionRecorded {
        authorization_plan: id(1),
        execution_plan: id(2),
        receipt: id(4),
        fence: id(9),
        epoch: 8,
    }));
    assert_eq!(
        denial,
        Err(OperationalRecoveryInvariant::PromotionBindingPreserved)
    );
}

#[test]
fn bootstrap_completion_cannot_substitute_the_source_lease() {
    let mut state = authorized_state();
    state
        .apply(&action(Binding::BootstrapTransfer {
            authorization_plan: id(1),
            execution_plan: id(2),
            receipt: id(3),
            source_lease: id(4),
            target: id(5),
        }))
        .unwrap();

    let denial = state.apply(&action(Binding::BootstrapCompleted {
        receipt: id(3),
        source_lease: id(8),
        verification: id(6),
    }));
    assert_eq!(
        denial,
        Err(OperationalRecoveryInvariant::BootstrapBindingPreserved)
    );
}

#[test]
fn rejoin_completion_requires_disposition_specific_forensic_evidence() {
    let mut state = promotion_state();
    state
        .apply(&action(Binding::RejoinPlanned {
            promotion_receipt: id(4),
            plan: id(7),
            disposition: 2,
        }))
        .unwrap();

    let denial = state.apply(&action(Binding::RejoinCompleted {
        plan: id(7),
        receipt: id(8),
        forensic_retention: [0; 32],
        rebootstrap_target: id(9),
        disposition: 2,
    }));
    assert_eq!(
        denial,
        Err(OperationalRecoveryInvariant::RejoinDispositionComplete)
    );
}

fn authorized_state() -> OperationalRecoverySemanticState {
    let mut state = OperationalRecoverySemanticState::default();
    state
        .apply(&action(Binding::Authorization {
            plan: id(1),
            execution: Some(id(2)),
            replayed: false,
        }))
        .unwrap();
    state
}

fn promotion_state() -> OperationalRecoverySemanticState {
    let mut state = authorized_state();
    state
        .apply(&action(Binding::PromotionFence {
            authorization_plan: id(1),
            execution_plan: id(2),
            fence: id(3),
            epoch: 8,
        }))
        .unwrap();
    state
        .apply(&action(Binding::PromotionRecorded {
            authorization_plan: id(1),
            execution_plan: id(2),
            receipt: id(4),
            fence: id(3),
            epoch: 8,
        }))
        .unwrap();
    state
        .apply(&action(Binding::PromotionPublished {
            receipt: id(4),
            publication: id(5),
            verification: id(6),
            target: id(10),
            epoch: 8,
        }))
        .unwrap();
    state
}

fn action(binding: Binding) -> OperationalRecoveryAction {
    OperationalRecoveryAction {
        authority_identity: id(20),
        operation_identity: "semantic-test".to_owned(),
        transition_identity: "transition".to_owned(),
        kind: OperationalRecoveryActionKind::WorkflowOpened,
        owner_tag: None,
        binding,
    }
}

const fn id(byte: u8) -> [u8; 32] {
    [byte; 32]
}
