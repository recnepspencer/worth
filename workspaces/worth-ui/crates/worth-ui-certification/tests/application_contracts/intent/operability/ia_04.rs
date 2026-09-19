use worth_ui::facade::intent::{
    UiIntentAffinityPosture, UiIntentInoperableCause, UiIntentMutabilityPosture,
    UiIntentOccupancyPosture, UiIntentOperabilityOutcome, UiIntentPolicyPosture,
    UiIntentReadinessPosture, UiIntentSupportPosture,
};
use worth_ui_test_support::{
    classify_intent_operability_for_certification, UiIntentOccupancyReleasePosture,
    UiIntentOperabilityDecisionCertificationInput, WorthUiIntentOccupancyCertificationExt,
    WorthUiMountedInteractionLifecycleCertificationExt,
};

use super::model::{causes, ModelAffinity, ModelCause, ModelInput};
use super::world::{OccupancyLayout, OperabilityWorld, PEER_POINT, PRIMARY_POINT};

#[test]
fn ia_04_exhausts_the_small_semantic_lattice_without_materializing_it_in_runtime() {
    let mut rows = 0;
    for mask in 0u16..256 {
        let input = ModelInput {
            supported: mask & 1 != 0,
            writable: mask & 2 != 0,
            ready: mask & 4 != 0,
            idle: mask & 8 != 0,
            policy_admitted: mask & 16 != 0,
            affinity: match (mask >> 5) & 3 {
                0 => ModelAffinity::Current,
                1 => ModelAffinity::Stale,
                2 => ModelAffinity::WrongWorld,
                _ => ModelAffinity::RebindRequired,
            },
            confirmation_required: mask & 128 != 0,
        };
        let expected = causes(input);
        let decision = classify_intent_operability_for_certification(classifier_input(input));
        let observed = decision.causes().map(map_cause).collect::<Vec<_>>();
        assert_eq!(observed, expected);
        assert_eq!(decision.is_operable(), model_operable(input));
        assert_eq!(
            decision.primary_cause().map(map_cause),
            expected.first().copied()
        );
        assert_eq!(decision.cost().selected_dependencies_visited(), 0);
        rows += 1;
    }
    assert_eq!(rows, 256);
}

#[test]
fn ia_04_production_decision_exhausts_boolean_axes_at_idle_and_in_flight() {
    assert_boolean_lattice(false);
    assert_boolean_lattice(true);
}

#[test]
fn ia_04_payload_and_operability_retain_one_exact_predecessor_basis() {
    let mut world = OperabilityWorld::scoped(OccupancyLayout::TargetRoute);
    let predecessor = world.prepare(PRIMARY_POINT);
    assert_eq!(
        predecessor.retained_operability_dependency_reference_count(),
        4
    );
    world.set_axes(false, false, false, true);

    let predecessor = world
        .interaction
        .session
        .evaluate_intent_operability(predecessor);
    drop(operable(predecessor));

    let successor = inoperable(world.evaluate(PRIMARY_POINT));
    assert_eq!(
        successor.causes().collect::<Vec<_>>(),
        [
            UiIntentInoperableCause::PolicyDenied,
            UiIntentInoperableCause::Readonly,
            UiIntentInoperableCause::Pending,
            UiIntentInoperableCause::ConfirmationRequired {
                policy_identity: "phase3.operability.confirmation-policy".into(),
            },
        ]
    );
    let _ = world.interaction.session.shutdown();
}

#[test]
fn ia_04_affinity_postures_are_distinct_closed_axes() {
    let mut source = OperabilityWorld::scoped(OccupancyLayout::TargetRoute);
    let candidate = source.prepare(PRIMARY_POINT);
    let mut foreign = OperabilityWorld::scoped(OccupancyLayout::TargetRoute);
    let decision = inoperable(
        foreign
            .interaction
            .session
            .evaluate_intent_operability(candidate),
    );
    assert_eq!(decision.affinity(), UiIntentAffinityPosture::WrongWorld);
    let _ = source.interaction.session.shutdown();
    let _ = foreign.interaction.session.shutdown();

    let mut rebound = OperabilityWorld::scoped(OccupancyLayout::TargetRoute);
    let candidate = rebound.prepare(PRIMARY_POINT);
    rebound.interaction.publish_successor();
    let decision = inoperable(
        rebound
            .interaction
            .session
            .evaluate_intent_operability(candidate),
    );
    assert_eq!(decision.affinity(), UiIntentAffinityPosture::RebindRequired);
    let _ = rebound.interaction.session.shutdown();

    let mut stale = OperabilityWorld::scoped(OccupancyLayout::TargetRoute);
    let candidate = stale.prepare(PRIMARY_POINT);
    let mounted = candidate.input_basis().target().mounted_instance();
    stale
        .interaction
        .session
        .unmount_instance_with_interaction_receipt(mounted)
        .expect("the exact candidate target unmounts");
    let decision = inoperable(
        stale
            .interaction
            .session
            .evaluate_intent_operability(candidate),
    );
    assert_eq!(decision.affinity(), UiIntentAffinityPosture::Stale);
    let _ = stale.interaction.session.shutdown();
}

#[test]
fn ia_04_occupancy_respects_target_declaration_definition_and_application_scope() {
    assert_scope(OccupancyLayout::TargetRoute, false);
    assert_scope(OccupancyLayout::Declaration, true);
    assert_scope(OccupancyLayout::Definition, true);
    assert_scope(OccupancyLayout::Application, true);
}

fn assert_boolean_lattice(in_flight: bool) {
    let mut world = OperabilityWorld::scoped(OccupancyLayout::Application);
    let reservation = if in_flight {
        let proof = operable(world.evaluate(PRIMARY_POINT));
        Some(
            world
                .interaction
                .session
                .reserve_intent_occupancy_for_certification(proof)
                .expect("the first exact proof atomically reserves its scope"),
        )
    } else {
        None
    };
    for mask in 0u8..16 {
        let writable = mask & 1 != 0;
        let ready = mask & 2 != 0;
        let policy = mask & 4 != 0;
        let confirmation = mask & 8 != 0;
        world.set_axes(writable, ready, policy, confirmation);
        let outcome = world.evaluate(PEER_POINT);
        let expected = causes(ModelInput {
            supported: true,
            writable,
            ready,
            idle: !in_flight,
            policy_admitted: policy,
            affinity: ModelAffinity::Current,
            confirmation_required: confirmation,
        });
        assert_production_decision(outcome, &expected);
    }
    if let Some(reservation) = reservation {
        let release = world
            .interaction
            .session
            .release_intent_occupancy_for_certification(reservation);
        assert_eq!(release, UiIntentOccupancyReleasePosture::Released);
    }
    assert_eq!(
        world
            .interaction
            .session
            .active_intent_occupancy_count_for_certification(),
        0
    );
    let _ = world.interaction.session.shutdown();
}

fn assert_scope(layout: OccupancyLayout, peer_is_occupied: bool) {
    let mut world = OperabilityWorld::scoped(layout);
    let proof = operable(world.evaluate(PRIMARY_POINT));
    let reservation = world
        .interaction
        .session
        .reserve_intent_occupancy_for_certification(proof)
        .expect("the first selected route reserves only its declared scope");
    assert_eq!(
        inoperable(world.evaluate(PRIMARY_POINT)).occupancy(),
        UiIntentOccupancyPosture::InFlight
    );
    let peer = world.evaluate(PEER_POINT);
    if peer_is_occupied {
        assert_eq!(
            inoperable(peer).primary_cause(),
            Some(UiIntentInoperableCause::Occupied)
        );
    } else {
        drop(operable(peer));
    }
    let release = world
        .interaction
        .session
        .release_intent_occupancy_for_certification(reservation);
    assert_eq!(release, UiIntentOccupancyReleasePosture::Released);
    assert_eq!(
        world
            .interaction
            .session
            .active_intent_occupancy_count_for_certification(),
        0
    );
    let _ = world.interaction.session.shutdown();
}

fn assert_production_decision(outcome: UiIntentOperabilityOutcome, expected: &[ModelCause]) {
    let decision = match &outcome {
        UiIntentOperabilityOutcome::Operable(proof) => proof.decision(),
        UiIntentOperabilityOutcome::Inoperable(candidate) => candidate.decision(),
    };
    assert_eq!(decision.cost().selected_dependencies_visited(), 7);
    assert_eq!(decision.support(), UiIntentSupportPosture::Supported);
    assert_eq!(decision.affinity(), UiIntentAffinityPosture::Current);
    assert_eq!(
        decision.mutability(),
        if expected.contains(&ModelCause::Readonly) {
            UiIntentMutabilityPosture::Readonly
        } else {
            UiIntentMutabilityPosture::Writable
        }
    );
    assert_eq!(
        decision.readiness(),
        if expected.contains(&ModelCause::Pending) {
            UiIntentReadinessPosture::Pending
        } else {
            UiIntentReadinessPosture::Ready
        }
    );
    assert_eq!(
        decision.policy(),
        if expected.contains(&ModelCause::PolicyDenied) {
            UiIntentPolicyPosture::Denied
        } else {
            UiIntentPolicyPosture::Admitted
        }
    );
    assert_eq!(
        decision.confirmation().required_policy_identity(),
        expected
            .contains(&ModelCause::ConfirmationRequired)
            .then_some("phase3.operability.confirmation-policy")
    );
    let observed = decision
        .causes()
        .map(map_cause)
        .collect::<Vec<ModelCause>>();
    assert_eq!(observed, expected);
    assert_eq!(
        matches!(outcome, UiIntentOperabilityOutcome::Operable(_)),
        expected.is_empty()
    );
}

fn inoperable(
    outcome: UiIntentOperabilityOutcome,
) -> worth_ui::facade::intent::UiIntentOperabilityDecision {
    match outcome {
        UiIntentOperabilityOutcome::Inoperable(candidate) => candidate.decision().clone(),
        UiIntentOperabilityOutcome::Operable(_) => panic!("expected an inoperable candidate"),
    }
}

fn operable(
    outcome: UiIntentOperabilityOutcome,
) -> worth_ui::facade::intent::UiIntentOperabilityProof {
    match outcome {
        UiIntentOperabilityOutcome::Operable(proof) => proof,
        UiIntentOperabilityOutcome::Inoperable(candidate) => {
            let causes = candidate.decision().causes().collect::<Vec<_>>();
            panic!("expected operable proof, got {causes:?}")
        }
    }
}

fn map_cause(cause: UiIntentInoperableCause) -> ModelCause {
    match cause {
        UiIntentInoperableCause::Unsupported => ModelCause::Unsupported,
        UiIntentInoperableCause::WrongWorld => ModelCause::WrongWorld,
        UiIntentInoperableCause::RebindRequired => ModelCause::RebindRequired,
        UiIntentInoperableCause::StaleTarget => ModelCause::StaleTarget,
        UiIntentInoperableCause::PolicyDenied => ModelCause::PolicyDenied,
        UiIntentInoperableCause::Occupied => ModelCause::Occupied,
        UiIntentInoperableCause::Readonly => ModelCause::Readonly,
        UiIntentInoperableCause::Pending => ModelCause::Pending,
        UiIntentInoperableCause::ConfirmationRequired { .. } => ModelCause::ConfirmationRequired,
    }
}

fn classifier_input(input: ModelInput) -> UiIntentOperabilityDecisionCertificationInput {
    UiIntentOperabilityDecisionCertificationInput {
        support: if input.supported {
            UiIntentSupportPosture::Supported
        } else {
            UiIntentSupportPosture::Unsupported
        },
        mutability: if input.writable {
            UiIntentMutabilityPosture::Writable
        } else {
            UiIntentMutabilityPosture::Readonly
        },
        readiness: if input.ready {
            UiIntentReadinessPosture::Ready
        } else {
            UiIntentReadinessPosture::Pending
        },
        occupancy: if input.idle {
            UiIntentOccupancyPosture::Idle
        } else {
            UiIntentOccupancyPosture::InFlight
        },
        policy: if input.policy_admitted {
            UiIntentPolicyPosture::Admitted
        } else {
            UiIntentPolicyPosture::Denied
        },
        affinity: match input.affinity {
            ModelAffinity::Current => UiIntentAffinityPosture::Current,
            ModelAffinity::Stale => UiIntentAffinityPosture::Stale,
            ModelAffinity::WrongWorld => UiIntentAffinityPosture::WrongWorld,
            ModelAffinity::RebindRequired => UiIntentAffinityPosture::RebindRequired,
        },
        confirmation: if input.confirmation_required {
            worth_ui::facade::intent::UiIntentConfirmationPosture::Required {
                policy_identity: "phase3.operability.confirmation-policy".into(),
            }
        } else {
            worth_ui::facade::intent::UiIntentConfirmationPosture::NotRequired
        },
    }
}

fn model_operable(input: ModelInput) -> bool {
    input.supported
        && input.writable
        && input.ready
        && input.idle
        && input.policy_admitted
        && matches!(input.affinity, ModelAffinity::Current)
        && !input.confirmation_required
}
