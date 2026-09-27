use super::UiScrollRuntimeState;
use crate::runtime::scroll::*;

fn registered(
    owner: UiScrollOwnerIdentity,
    incarnation: UiScrollOwnerIncarnation,
    offset: UiScrollOffset,
) -> UiScrollRuntimeState {
    let mut state = UiScrollRuntimeState::new_session_restore_candidate();
    state
        .register(UiScrollOwnerRegistration::new(
            owner,
            incarnation,
            UiScrollAxes::Block,
            UiScrollBounds::new(0, 100_000).unwrap(),
            offset,
        ))
        .unwrap();
    state.ownership_references.insert(owner, 1);
    state
}

/// A settle carried into a staged layout answers a staged offset, and the
/// accepted one stays where the host last accepted it until that layout is
/// presented.
#[test]
fn a_staged_layout_answers_a_staged_offset_and_leaves_the_accepted_one() {
    let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let owner =
        UiScrollOwnerIdentity::region(surface, crate::graph::UiGraphNodeIdentity::new(1), 1);
    let incarnation = UiScrollOwnerIncarnation::new(1).unwrap();
    let accepted = UiScrollOffset::origin();
    let mut state = registered(owner, incarnation, accepted);
    state.stage_layout_successor(surface, &registered(owner, incarnation, accepted));

    let displayed = UiScrollOffset::new(0, 40_000).unwrap();
    assert_eq!(
        state.staged_layout_offset(owner, incarnation, displayed),
        Some(UiStagedScrollOffset::staged(displayed))
    );
    assert_eq!(state.offset(owner, incarnation), Ok(accepted));
    // Past the staged bounds the layout follows only as far as they reach.
    assert_eq!(
        state.staged_layout_offset(owner, incarnation, UiScrollOffset::new(0, 250_000).unwrap()),
        Some(UiStagedScrollOffset::staged(
            UiScrollOffset::new(0, 100_000).unwrap()
        ))
    );
    state.settle_staged_layout(owner, incarnation, displayed);
    assert_eq!(
        state.staged_layout_offset(owner, incarnation, displayed),
        None
    );
    assert_eq!(state.offset(owner, incarnation), Ok(accepted));

    assert!(state.commit_presented_layout(surface));
    assert_eq!(state.offset(owner, incarnation), Ok(displayed));
}
