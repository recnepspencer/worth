use std::collections::{HashMap, HashSet};

use worth_ui_host_contract::{
    UiMountedPaintCommandChange, UiMountedPresentationAuxiliaryState,
    UiMountedPresentationDeltaInput, UiMountedPresentationInitialInput,
    UiMountedPresentationReconstructionInput, UiMountedPresentationSampleInput,
    UiMountedPresentationUnchangedInput,
};

pub(super) fn validate_initial(input: &UiMountedPresentationInitialInput) {
    validate_initial_affinity(input);
    validate_initial_identity_sets(input);
    assert!(input.order_integrity.admits(&input.order));
    validate_initial_reconstruction(input);
}

pub(super) fn validate_delta(input: &UiMountedPresentationDeltaInput) {
    assert_ne!(input.predecessor, input.successor);
    let change_identities = input
        .changes
        .iter()
        .flat_map(|change| match change {
            UiMountedPaintCommandChange::Insert(command) => vec![command.identity()],
            UiMountedPaintCommandChange::Replace {
                predecessor,
                successor,
            } if *predecessor != successor.identity() => {
                vec![*predecessor, successor.identity()]
            }
            UiMountedPaintCommandChange::Replace { predecessor, .. }
            | UiMountedPaintCommandChange::Remove(predecessor) => vec![*predecessor],
        })
        .collect::<HashSet<_>>();
    let order_identities = input
        .order
        .iter()
        .map(|edit| edit.identity().command())
        .collect::<HashSet<_>>();
    let node_identities = input
        .nodes
        .iter()
        .map(|change| change.mounted_instance())
        .collect::<HashSet<_>>();
    let expected_change_identities = input
        .changes
        .iter()
        .map(|change| match change {
            UiMountedPaintCommandChange::Replace {
                predecessor,
                successor,
            } if *predecessor != successor.identity() => 2,
            _ => 1,
        })
        .sum::<usize>();
    assert_eq!(change_identities.len(), expected_change_identities);
    assert_eq!(order_identities.len(), input.order.len());
    assert_eq!(node_identities.len(), input.nodes.len());
    assert!(
        !input.changes.is_empty()
            || !input.nodes.is_empty()
            || !input.order.is_empty()
            || input.auxiliary.is_some(),
        "an empty transition must use unchanged work"
    );
}

pub(super) fn validate_reconstruction(input: &UiMountedPresentationReconstructionInput) {
    assert_eq!(input.projection.frame(), input.successor);
    assert_eq!(input.projection.surface(), input.surface);
    assert_eq!(input.projection.binding(), input.binding);
    assert_eq!(input.projection.content_generation(), input.content);
    let command_identities = input
        .commands
        .iter()
        .map(|command| command.identity())
        .collect::<HashSet<_>>();
    let order_identities = input
        .order
        .iter()
        .map(|identity| identity.command())
        .collect::<HashSet<_>>();
    assert_eq!(command_identities.len(), input.commands.len());
    assert_eq!(command_identities, order_identities);
    assert!(input.order_integrity.admits(&input.order));
    let command_map = input
        .commands
        .iter()
        .cloned()
        .map(|command| (command.identity(), command))
        .collect::<HashMap<_, _>>();
    let reconstructed =
        UiMountedPresentationAuxiliaryState::from_runtime_mounting(&input.projection)
            .reconstruct(&command_map)
            .expect("reconstruction work must rebuild its complete projection");
    assert_eq!(reconstructed, input.projection);
}

pub(super) fn validate_sample(input: &UiMountedPresentationSampleInput) {
    let identities = input
        .changes
        .iter()
        .map(|change| change.command())
        .collect::<HashSet<_>>();
    assert_eq!(identities.len(), input.changes.len());
    assert!(!input.changes.is_empty());
    // Logical allocation damage can be empty for visible overhanging text.
    // Actual image damage is finalized by the host presentation owner.
}

pub(super) fn validate_unchanged(input: &UiMountedPresentationUnchangedInput) {
    assert_ne!(input.predecessor, input.successor);
}

fn validate_initial_affinity(input: &UiMountedPresentationInitialInput) {
    assert_eq!(input.projection.frame(), input.successor);
    assert_eq!(input.projection.surface(), input.surface);
    assert_eq!(input.projection.binding(), input.binding);
    assert_eq!(input.projection.content_generation(), input.content);
}

fn validate_initial_identity_sets(input: &UiMountedPresentationInitialInput) {
    let command_identities = input
        .commands
        .iter()
        .map(|command| command.identity())
        .collect::<HashSet<_>>();
    let order_identities = input
        .order
        .iter()
        .map(|identity| identity.command())
        .collect::<HashSet<_>>();
    assert_eq!(command_identities.len(), input.commands.len());
    assert_eq!(order_identities.len(), input.order.len());
    assert_eq!(command_identities, order_identities);
}

fn validate_initial_reconstruction(input: &UiMountedPresentationInitialInput) {
    let command_map = input
        .commands
        .iter()
        .cloned()
        .map(|command| (command.identity(), command))
        .collect::<HashMap<_, _>>();
    let reconstructed =
        UiMountedPresentationAuxiliaryState::from_runtime_mounting(&input.projection)
            .reconstruct(&command_map)
            .expect("initial presentation commands must reconstruct their projection tables");
    assert_eq!(reconstructed, input.projection);
}
