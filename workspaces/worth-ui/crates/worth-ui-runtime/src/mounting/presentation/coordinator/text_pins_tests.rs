use super::*;
use crate::certification_support::{
    initial_presentation_mechanics_for_certification, semantic_text_projection_for_certification,
    UiSemanticTextProjectionCertificationMutation,
};
use crate::mounting::qualified_text_test_support::inert_qualified_layout;
use crate::native_platform::text_presentation::{
    prepare_mounted_semantic_text, UiMountedEventTimeDpiAuthority,
    UiNativeTextPresentationPreparation,
};
use std::collections::HashSet;
use worth_ui_host_contract::{
    UiHostSurfaceIdentity, UiHostSurfacePresentationMode, UiMountedPaintCommandChange,
    UiMountedPaintOrderIntegrity, UiMountedPresentationDelta, UiMountedPresentationDeltaInput,
    UiMountedPresentationWorkView, UiMountedSurfaceBindingRequirement,
    WorthUiHostCapabilityObservationGeneration,
};

fn prepared_initial(
    projection: &worth_ui_host_contract::UiMountedProjectionView,
    requirement: UiMountedSurfaceBindingRequirement,
) -> (
    worth_ui_host_contract::UiMountedPresentationInitial,
    UiNativeTextPresentationPrepared,
) {
    let mechanics = initial_presentation_mechanics_for_certification(projection, requirement);
    let layout = inert_qualified_layout("ONLINE");
    let dpi = UiMountedEventTimeDpiAuthority::from_requirement(requirement).unwrap();
    let Some(UiNativeTextPresentationPreparation::Prepared(prepared)) =
        prepare_mounted_semantic_text(
            UiMountedPresentationWorkView::Initial(&mechanics),
            dpi,
            |identity| (identity == layout.identity()).then_some(layout.as_ref()),
        )
    else {
        panic!("exact mounted text must prepare native raster demands");
    };
    (mechanics, prepared)
}

fn requirement_for(
    projection: &worth_ui_host_contract::UiMountedProjectionView,
) -> UiMountedSurfaceBindingRequirement {
    UiMountedSurfaceBindingRequirement::new(
        projection.surface(),
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        projection.binding(),
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::NativeDisplay,
    )
}

fn prepared_pins(prepared: &UiNativeTextPresentationPrepared) -> Vec<UiGlyphRasterPinRequest> {
    prepared
        .demand_batches()
        .iter()
        .flat_map(|demand| {
            demand.records().iter().map(|record| {
                UiGlyphRasterPinRequest::from_text_mechanics(demand.layout_identity(), record.key())
            })
        })
        .collect()
}

/// Two candidates prepared from the same committed pins can both land. Each
/// lands over the pins committed when it lands, so the binding counts its
/// pins once and releasing them leaves no owner behind.
#[test]
fn a_candidate_lands_over_the_pins_committed_when_it_lands() {
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = requirement_for(&projection);
    let (_, prepared) = prepared_initial(&projection, requirement);
    let binding = requirement.binding();
    let mut owner = UiMountedTextPinState::default();

    let first = owner.candidate(binding, &prepared);
    let second = owner.candidate(binding, &prepared);
    owner.commit_presented(first);
    owner.commit_presented(second);

    assert!(!owner.global_pin_owners.is_empty());
    assert!(owner.global_pin_owners.values().all(|owners| *owners == 1));
    owner.commit_presented(owner.deregistration_candidate(binding));
    assert!(owner.global_pin_owners.is_empty());
}

#[test]
fn real_prepared_demands_advance_pins_only_after_accepted_settlement() {
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = requirement_for(&projection);
    let (_, prepared) = prepared_initial(&projection, requirement);
    let binding = requirement.binding();
    let expected = prepared_pins(&prepared);
    let mut owner = UiMountedTextPinState::default();

    let denied = owner.candidate(binding, &prepared);
    assert_eq!(denied.additions().len(), expected.len());
    assert!(expected.iter().all(|pin| denied.additions().contains(pin)));
    assert!(denied.releases().is_empty());
    drop(denied);
    assert!(owner.committed(binding).is_empty());

    let accepted = owner.candidate(binding, &prepared);
    owner.commit_presented(accepted);
    let committed = owner.committed(binding);
    assert_eq!(committed.len(), expected.len());
    assert!(expected.iter().all(|pin| committed.contains(pin)));

    let retained = owner.candidate(binding, &prepared);
    assert!(retained.additions().is_empty());
    assert!(retained.releases().is_empty());
    owner.commit_presented(retained);
}

/// A complete re-presentation at a new device scale changes every raster key
/// and omits a command the binding held: the binding adds exactly the new
/// pins and releases exactly the old ones, the omitted command's included.
#[test]
fn a_complete_representation_releases_every_pin_it_no_longer_holds() {
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = requirement_for(&projection);
    let (initial, prepared) = prepared_initial(&projection, requirement);
    let binding = requirement.binding();
    let mut owner = UiMountedTextPinState::default();
    owner.commit_presented(owner.candidate(binding, &prepared));
    let old = owner.committed(binding).into_iter().collect::<HashSet<_>>();
    let other_projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let (other_initial, _) =
        prepared_initial(&other_projection, requirement_for(&other_projection));
    let dropped_command = other_initial.commands()[0].identity();
    let dropped_pins = old.iter().copied().collect::<Vec<_>>();
    let committed = owner.committed.get_mut(&binding).unwrap();
    committed
        .by_command
        .insert(dropped_command, dropped_pins.clone().into_boxed_slice());
    add_pin_owners(&mut committed.pin_owners, &dropped_pins);
    let rescaled_requirement = UiMountedSurfaceBindingRequirement::with_baseline_and_device_scale(
        initial.affinity().surface(),
        requirement.host_surface(),
        binding,
        requirement.capability_generation(),
        requirement.capability_profile_digest(),
        UiHostSurfacePresentationMode::NativeDisplay,
        initial.affinity().baseline(),
        1_500,
    );
    let (_, rescaled) = prepared_initial(&projection, rescaled_requirement);
    assert!(rescaled.pin_set_complete());
    let new = prepared_pins(&rescaled).into_iter().collect::<HashSet<_>>();
    assert!(!old.is_empty() && old.is_disjoint(&new));

    let candidate = owner.candidate(binding, &rescaled);
    assert_eq!(
        candidate
            .additions()
            .iter()
            .copied()
            .collect::<HashSet<_>>(),
        new
    );
    assert_eq!(
        candidate.releases().iter().copied().collect::<HashSet<_>>(),
        old
    );
    owner.commit_presented(candidate);
    assert_eq!(
        owner.committed(binding).into_iter().collect::<HashSet<_>>(),
        new
    );
    assert_eq!(
        owner
            .global_pin_owners
            .keys()
            .copied()
            .collect::<HashSet<_>>(),
        new
    );
}

#[test]
fn shared_pins_release_only_after_the_last_binding_is_deregistered() {
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = requirement_for(&projection);
    let (_, prepared) = prepared_initial(&projection, requirement);
    let first_binding = requirement.binding();
    let second_binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let mut owner = UiMountedTextPinState::default();

    let first = owner.candidate(first_binding, &prepared);
    assert!(!first.additions().is_empty());
    owner.commit_presented(first);

    let second = owner.candidate(second_binding, &prepared);
    assert!(second.additions().is_empty());
    assert!(second.releases().is_empty());
    owner.commit_presented(second);

    let release_first = owner.deregistration_candidate(first_binding);
    assert!(release_first.releases().is_empty());
    owner.commit_presented(release_first);
    assert!(owner.committed(first_binding).is_empty());
    assert!(!owner.committed(second_binding).is_empty());

    let release_last = owner.deregistration_candidate(second_binding);
    assert!(!release_last.releases().is_empty());
    owner.commit_presented(release_last);
    assert!(owner.committed(second_binding).is_empty());
}

#[test]
fn command_owner_removal_is_visible_even_when_shared_keys_stay_pinned() {
    let first_projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let second_projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let first_requirement = requirement_for(&first_projection);
    let second_requirement = requirement_for(&second_projection);
    let (first_initial, prepared) = prepared_initial(&first_projection, first_requirement);
    let (second_initial, _) = prepared_initial(&second_projection, second_requirement);
    let first_command = first_initial.commands()[0].identity();
    let second_command = second_initial.commands()[0].identity();
    assert_ne!(first_command, second_command);
    let pins = prepared_pins(&prepared);
    let mut previous = UiMountedBindingPins::default();
    previous
        .by_command
        .insert(first_command, pins.clone().into_boxed_slice());
    previous
        .by_command
        .insert(second_command, pins.clone().into_boxed_slice());
    add_pin_owners(&mut previous.pin_owners, &pins);
    add_pin_owners(&mut previous.pin_owners, &pins);
    let binding = first_requirement.binding();
    let mut owner = UiMountedTextPinState::default();
    owner.committed.insert(binding, previous.clone());
    add_pin_owners(&mut owner.global_pin_owners, &pins);

    let mut retained = previous.clone();
    let removed = retained.by_command.remove(&first_command).unwrap();
    remove_pin_owners(&mut retained.pin_owners, &removed);
    let first_release = owner.candidate_from_next(binding, Some(&previous), retained);
    assert!(first_release.changes_binding());
    assert!(first_release.additions().is_empty());
    assert!(first_release.releases().is_empty());
    owner.commit_presented(first_release);

    let previous = owner.committed.get(&binding).cloned().unwrap();
    let last_release =
        owner.candidate_from_next(binding, Some(&previous), UiMountedBindingPins::default());
    assert!(last_release.changes_binding());
    assert_eq!(last_release.releases().len(), pins.len());
}

#[test]
fn partial_delta_replaces_only_its_command_and_preserves_unchanged_shared_pins() {
    let first_projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let second_projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let first_requirement = requirement_for(&first_projection);
    let second_requirement = requirement_for(&second_projection);
    let (first_initial, first_prepared) = prepared_initial(&first_projection, first_requirement);
    let (second_initial, _) = prepared_initial(&second_projection, second_requirement);
    let binding = first_requirement.binding();
    let mut owner = UiMountedTextPinState::default();
    let first_candidate = owner.candidate(binding, &first_prepared);
    owner.commit_presented(first_candidate);
    let unchanged_pins = owner.committed(binding);
    assert!(!unchanged_pins.is_empty());

    let affinity = first_initial.affinity();
    let inserted_successor =
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let second_command = second_initial.commands()[0].clone();
    let insert =
        UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
            predecessor: affinity.successor(),
            successor: inserted_successor,
            surface: affinity.surface(),
            binding,
            content: affinity.content(),
            baseline: affinity.baseline(),
            changes: vec![UiMountedPaintCommandChange::Insert(second_command.clone())],
            nodes: Vec::new(),
            order: Vec::new(),
            order_integrity: UiMountedPaintOrderIntegrity::for_order(&[]),
            damage: vec![
                worth_ui_host_contract::UiMountedLogicalDamage::from_runtime_mounting(
                    second_command.bounds(),
                ),
            ],
            auxiliary: None,
            production_cost: first_initial.production_cost(),
        });
    let layout = inert_qualified_layout("ONLINE");
    let Some(UiNativeTextPresentationPreparation::Prepared(inserted_prepared)) =
        prepare_mounted_semantic_text(
            UiMountedPresentationWorkView::Delta(&insert),
            UiMountedEventTimeDpiAuthority::from_requirement(first_requirement).unwrap(),
            |identity| (identity == layout.identity()).then_some(layout.as_ref()),
        )
    else {
        panic!("the second lawful text command must prepare its pin ownership");
    };
    let inserted = owner.candidate(binding, &inserted_prepared);
    assert!(inserted.additions().is_empty());
    assert!(inserted.releases().is_empty());
    owner.commit_presented(inserted);

    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: inserted_successor,
        successor: worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        surface: affinity.surface(),
        binding,
        content: affinity.content(),
        baseline: affinity.baseline(),
        changes: vec![UiMountedPaintCommandChange::replacement(
            first_initial.commands()[0].identity(),
            first_initial.commands()[0].clone(),
        )],
        nodes: Vec::new(),
        order: Vec::new(),
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&[]),
        damage: vec![
            worth_ui_host_contract::UiMountedLogicalDamage::from_runtime_mounting(
                first_initial.commands()[0].bounds(),
            ),
        ],
        auxiliary: None,
        production_cost: first_initial.production_cost(),
    });
    let scaled_requirement = UiMountedSurfaceBindingRequirement::with_baseline_and_device_scale(
        affinity.surface(),
        first_requirement.host_surface(),
        binding,
        first_requirement.capability_generation(),
        first_requirement.capability_profile_digest(),
        UiHostSurfacePresentationMode::NativeDisplay,
        affinity.baseline(),
        1_500,
    );
    let Some(UiNativeTextPresentationPreparation::Prepared(delta_prepared)) =
        prepare_mounted_semantic_text(
            UiMountedPresentationWorkView::Delta(&delta),
            UiMountedEventTimeDpiAuthority::from_requirement(scaled_requirement).unwrap(),
            |identity| (identity == layout.identity()).then_some(layout.as_ref()),
        )
    else {
        panic!("the changed text command must prepare a partial pin delta");
    };
    assert!(!delta_prepared.pin_set_complete());
    let candidate = owner.candidate(binding, &delta_prepared);
    assert!(candidate.releases().is_empty());
    assert!(!candidate.additions().is_empty());
    owner.commit_presented(candidate);
    let committed = owner.committed(binding);
    assert!(unchanged_pins.iter().all(|pin| committed.contains(pin)));
    assert!(delta_prepared.demand_batches()[0]
        .records()
        .iter()
        .all(|record| committed.iter().any(|pin| pin.key() == record.key())));
}
