use super::{UiScrollRuntimeState, STATE_CLONES};
use crate::runtime::scroll::transition::{UiScrollWheelInput, UiScrollWheelLineDelta};
use crate::runtime::scroll::*;

fn input() -> UiScrollWheelInput {
    UiScrollWheelInput::admit(
        UiScrollWheelLineDelta::from_notches(0, 1, 3),
        worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
        10,
        20,
        8,
    )
    .expect("coarse input")
}

#[test]
fn scroll_route_candidate_copies_only_the_chain_and_merges_without_losing_unrelated_state() {
    for unrelated in [0, 1_024] {
        let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let incarnation = UiScrollOwnerIncarnation::new(1).unwrap();
        let bounds = UiScrollBounds::new(0, 1_000_000).unwrap();
        let mut state = UiScrollRuntimeState::new_session_restore_candidate();
        let mut selected = None;
        for index in 0..=unrelated {
            let owner = UiScrollOwnerIdentity::region(
                surface,
                crate::graph::UiGraphNodeIdentity::new(index + 1),
                1,
            );
            state
                .register(UiScrollOwnerRegistration::new(
                    owner,
                    incarnation,
                    UiScrollAxes::Block,
                    bounds,
                    UiScrollOffset::origin(),
                ))
                .unwrap();
            state
                .stage_wheel_transition(UiScrollChainEntry::new(owner, incarnation), input())
                .unwrap();
            let mounted =
                worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
            // Catalog provenance is not this state-fork test's claim. Include
            // unrelated negative cache entries to catch accidental copying or
            // replacement of the installed ownership catalog.
            state.ownership_catalog.insert(
                mounted,
                super::super::UiScrollOwnershipCatalogRecord {
                    incarnation,
                    resolution: Err(UiScrollOwnershipResolutionDenial::UnknownGraphNode),
                },
            );
            state.ownership_references.insert(owner, 1);
            selected.get_or_insert(owner);
        }
        let selected = selected.unwrap();
        let chain = [UiScrollChainEntry::new(selected, incarnation)];
        let original_catalog = state.ownership_catalog.clone();
        let original_targets = state.transition_targets.clone();
        // Check that the observer actually sees derived whole-state cloning.
        let _ = &state.clone_observation;
        STATE_CLONES.with(|count| count.set(0));
        drop(state.clone());
        assert_eq!(STATE_CLONES.with(|count| count.replace(0)), 1);

        let mut candidate = state.route_candidate(&chain, false).unwrap();
        assert_eq!(STATE_CLONES.with(|count| count.get()), 0);
        assert_eq!(candidate.state.owners.len(), 1);
        assert_eq!(candidate.state.pending_transition_count(), 1);
        assert!(candidate.state.ownership_catalog.is_empty());
        assert!(candidate.state.ownership_references.is_empty());
        assert!(candidate.state.pending_layouts.is_empty());
        assert!(candidate.state.pending_direct.is_empty());
        let receipt = candidate
            .state_mut()
            .route_with_reconciled_bounds(
                UiScrollDeltaRequest::new(
                    chain.to_vec(),
                    UiScrollDelta::new(0, 0),
                    UiScrollDeltaCause::Host {
                        source: worth_ui_host_contract::UiHostScrollDeltaSource::PointerWheel,
                        phase: worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
                        precision: worth_ui_host_contract::UiHostScrollDeltaPrecision::Pixel,
                    },
                )
                .unwrap(),
                &[bounds],
            )
            .unwrap();
        assert_eq!(receipt.owners_visited(), 1);
        candidate.state_mut().retire_transition(selected);
        state.commit_routed_candidate(candidate);
        assert_eq!(STATE_CLONES.with(|count| count.get()), 0);
        assert_eq!(state.owner_count(), unrelated as usize + 1);
        assert_eq!(state.ownership_catalog, original_catalog);
        assert_eq!(state.ownership_references.len(), unrelated as usize + 1);
        assert_eq!(state.pending_transition_count(), unrelated as usize);
        for (owner, target) in original_targets.pending_owners() {
            if owner == selected {
                assert_eq!(state.transition_target(owner, incarnation), None);
            } else {
                assert_eq!(state.transition_target(owner, incarnation), Some(target));
            }
        }
    }
}

#[test]
fn scroll_route_candidate_direct_intent_accumulates_without_advancing_accepted_state() {
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let owner = UiScrollOwnerIdentity::viewport(surface);
    let incarnation = UiScrollOwnerIncarnation::new(1).unwrap();
    let bounds = UiScrollBounds::new(0, 1_000).unwrap();
    let occurrence = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(UiScrollOwnerRegistration::new(
            owner,
            incarnation,
            UiScrollAxes::Block,
            bounds,
            UiScrollOffset::origin(),
        ))
        .unwrap();
    let chain = [UiScrollChainEntry::new(owner, incarnation)];
    let mut first = state.route_candidate(&chain, true).unwrap();
    let receipt = first
        .state_mut()
        .route_with_reconciled_bounds(
            UiScrollDeltaRequest::new(
                chain.to_vec(),
                UiScrollDelta::new(0, 25),
                UiScrollDeltaCause::Host {
                    source: worth_ui_host_contract::UiHostScrollDeltaSource::PointerWheel,
                    phase: worth_ui_host_contract::UiHostScrollDeltaPhase::Updated,
                    precision: worth_ui_host_contract::UiHostScrollDeltaPrecision::Pixel,
                },
            )
            .unwrap(),
            &[bounds],
        )
        .unwrap();
    let prepared = first
        .state()
        .prepare_direct_succession(&receipt, occurrence, &[None]);
    state.stage_direct_succession(first.state(), &prepared);
    STATE_CLONES.with(|count| count.set(0));
    let second = state.route_candidate(&chain, true).unwrap();
    assert_eq!(STATE_CLONES.with(|count| count.get()), 0);
    assert_eq!(
        second
            .state()
            .offset(owner, incarnation)
            .unwrap()
            .block_subpixels(),
        25
    );
    assert_eq!(
        state.offset(owner, incarnation).unwrap(),
        UiScrollOffset::origin()
    );
    assert!(state.has_direct_succession(&prepared[0]));
    drop(second); // A rejected/discarded candidate changes no pending authority.
    assert!(state.commit_presented_direct(prepared[0]));
    assert_eq!(
        state.offset(owner, incarnation).unwrap().block_subpixels(),
        25
    );
}
