use std::collections::HashSet;

use super::{
    UiHeadlessMountedFrameTranscript, UiHeadlessSemanticTextMechanic,
    UiHeadlessTranscriptSuccessorIdentity,
};
use worth_ui_host_contract::{UiMountedLogicalDamage, UiMountedPaintOrderIdentity};

impl UiHeadlessMountedFrameTranscript {
    pub(crate) fn successor_recorded_delta(
        &self,
        identity: UiHeadlessTranscriptSuccessorIdentity,
        changes: &[worth_ui_host_contract::UiMountedPaintCommandChange],
        order_edits: &[worth_ui_host_contract::UiMountedPaintOrderEdit],
        order_integrity: worth_ui_host_contract::UiMountedPaintOrderIntegrity,
        damage: &[UiMountedLogicalDamage],
        node_changes: &[worth_ui_host_contract::UiMountedPresentationNodeChange],
        semantic_snapshots: &[(
            worth_ui_host_contract::UiMountedPaintCommandIdentity,
            UiHeadlessSemanticTextMechanic,
        )],
    ) -> Result<Self, worth_ui_host_contract::UiHostSurfacePresentationDenial> {
        let mut order = self.paint_order.to_vec();
        apply_recorded_order_edits(&mut order, order_edits)?;
        if !order_integrity.admits(&order) {
            return Err(malformed());
        }
        let mut successor = self.successor_recorded_identity(identity);
        apply_mechanic_changes(&mut successor, changes, semantic_snapshots)?;
        apply_node_changes(&mut successor, node_changes)?;
        refresh_native_paint_effect(&mut successor)?;
        successor.paint_order = order.into_boxed_slice();
        successor.logical_damage = damage.into();
        Ok(successor)
    }

    fn successor_recorded_identity(&self, identity: UiHeadlessTranscriptSuccessorIdentity) -> Self {
        let mut successor = self.clone();
        successor.host_session_identity = identity.host_session_identity;
        successor.protocol = identity.protocol;
        successor.attempt = identity.attempt;
        successor.frame = identity.frame;
        successor.binding = identity.binding;
        successor.appearance_work = None;
        successor
    }

    pub(crate) fn successor_appearance(
        &self,
        identity: UiHeadlessTranscriptSuccessorIdentity,
        appearance: super::appearance::UiHeadlessAppearancePresentationTranscript,
    ) -> Result<Self, worth_ui_host_contract::UiHostSurfacePresentationDenial> {
        let mut successor = self.successor_recorded_identity(identity);
        successor.replace_appearance_work(appearance)?;
        Ok(successor)
    }
}

fn apply_node_changes(
    successor: &mut UiHeadlessMountedFrameTranscript,
    changes: &[worth_ui_host_contract::UiMountedPresentationNodeChange],
) -> Result<(), worth_ui_host_contract::UiHostSurfacePresentationDenial> {
    let mut affected = HashSet::with_capacity(changes.len());
    let mut upserts = Vec::new();
    for change in changes {
        let instance = change.mounted_instance();
        if !affected.insert(instance) {
            return Err(malformed());
        }
        let current = successor
            .nodes
            .iter()
            .position(|node| node.mounted_instance() == instance);
        match change {
            worth_ui_host_contract::UiMountedPresentationNodeChange::Remove(_) => {
                current.ok_or_else(malformed)?;
            }
            worth_ui_host_contract::UiMountedPresentationNodeChange::Upsert(state) => {
                upserts.push(translate_node_state(successor, *state)?);
            }
        }
    }
    let mut nodes = successor
        .nodes
        .iter()
        .filter(|node| !affected.contains(&node.mounted_instance()))
        .cloned()
        .collect::<Vec<_>>();
    let mut positions = nodes
        .iter()
        .map(|node| node.authored_position())
        .collect::<HashSet<_>>();
    for node in upserts {
        if !positions.insert(node.authored_position()) {
            return Err(malformed());
        }
        let insertion = nodes
            .partition_point(|existing| existing.authored_position() < node.authored_position());
        nodes.insert(insertion, node);
    }
    successor.nodes = nodes.into_boxed_slice();
    refresh_node_effects(successor)
}

fn translate_node_state(
    successor: &UiHeadlessMountedFrameTranscript,
    state: worth_ui_host_contract::UiMountedPresentationNodeState,
) -> Result<super::UiHeadlessNodeMechanic, worth_ui_host_contract::UiHostSurfacePresentationDenial>
{
    use worth_ui_host_contract::UiMountedPresentationNodePaint;
    let paint = match state.paint() {
        UiMountedPresentationNodePaint::Command(identity) => {
            let _ = identity;
            return Err(malformed());
        }
        UiMountedPresentationNodePaint::CountOnlyBatch(index) => {
            if usize::from(index) >= successor.paint_batches.len() {
                return Err(malformed());
            }
            super::UiHeadlessNodePaintMechanic::CountOnlyBatch(index)
        }
        UiMountedPresentationNodePaint::Omitted(reason) => {
            super::UiHeadlessNodePaintMechanic::Omitted(reason)
        }
    };
    Ok(super::UiHeadlessNodeMechanic::new(
        super::UiHeadlessNodeMechanicInput {
            mounted_instance: state.mounted_instance(),
            authored_position: state.authored_position(),
            role: state.role(),
            participation: state.participation(),
            allocation: state.allocation(),
            preview: state.preview(),
            paint,
            accessibility: state.accessibility(),
            motion: state.motion(),
            diagnostic: state.diagnostic(),
        },
    ))
}

fn refresh_native_paint_effect(
    successor: &mut UiHeadlessMountedFrameTranscript,
) -> Result<(), worth_ui_host_contract::UiHostSurfacePresentationDenial> {
    let portal_overlay_count =
        u32::try_from(successor.portal_overlays.len()).map_err(|_| malformed())?;
    let semantic_text_count =
        u32::try_from(successor.semantic_text.len()).map_err(|_| malformed())?;
    let preview_node_count = node_count(&successor.nodes, |node| {
        matches!(
            node.preview(),
            worth_ui_host_contract::UiMountedPreviewProjection::Resize { .. }
        )
    })?;
    let effect = successor
        .unperformed_effects
        .iter_mut()
        .find_map(|effect| match effect {
            super::UiHeadlessUnperformedEffect::NativePaint {
                appearance_mechanic_count,
                portal_overlay_count,
                semantic_text_count,
                preview_node_count,
            } => Some((
                appearance_mechanic_count,
                portal_overlay_count,
                semantic_text_count,
                preview_node_count,
            )),
            _ => None,
        })
        .ok_or_else(malformed)?;
    *effect.0 = successor
        .appearance_work
        .as_ref()
        .map_or(Ok(0), |appearance| appearance.mechanic_count())?;
    *effect.1 = portal_overlay_count;
    *effect.2 = semantic_text_count;
    *effect.3 = preview_node_count;
    Ok(())
}

fn refresh_node_effects(
    successor: &mut UiHeadlessMountedFrameTranscript,
) -> Result<(), worth_ui_host_contract::UiHostSurfacePresentationDenial> {
    use worth_ui_host_contract::UiMountedAccessibilityProjection;
    successor.unperformed_effects = successor
        .unperformed_effects
        .iter()
        .copied()
        .filter(|effect| {
            !matches!(
                effect,
                super::UiHeadlessUnperformedEffect::Accessibility { .. }
                    | super::UiHeadlessUnperformedEffect::Diagnostic { .. }
            )
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let predicates = [
        node_count(&successor.nodes, |node| {
            matches!(
                node.accessibility(),
                UiMountedAccessibilityProjection::Admitted(_)
            )
        })?,
        node_count(&successor.nodes, |node| {
            matches!(
                node.diagnostic(),
                worth_ui_host_contract::UiMountedDiagnosticProjection::Reference(_)
                    | worth_ui_host_contract::UiMountedDiagnosticProjection::IdentityOverlay(_)
            )
        })?,
    ];
    let mut effects = successor.unperformed_effects.to_vec();
    if predicates[0] > 0 {
        effects.push(super::UiHeadlessUnperformedEffect::Accessibility {
            node_count: predicates[0],
        });
    }
    if predicates[1] > 0 {
        effects.push(super::UiHeadlessUnperformedEffect::Diagnostic {
            node_count: predicates[1],
        });
    }
    successor.unperformed_effects = effects.into_boxed_slice();
    Ok(())
}

fn node_count(
    nodes: &[super::UiHeadlessNodeMechanic],
    predicate: impl Fn(&super::UiHeadlessNodeMechanic) -> bool,
) -> Result<u32, worth_ui_host_contract::UiHostSurfacePresentationDenial> {
    u32::try_from(nodes.iter().filter(|node| predicate(node)).count()).map_err(|_| malformed())
}

fn apply_mechanic_changes(
    successor: &mut UiHeadlessMountedFrameTranscript,
    changes: &[worth_ui_host_contract::UiMountedPaintCommandChange],
    semantic_snapshots: &[(
        worth_ui_host_contract::UiMountedPaintCommandIdentity,
        UiHeadlessSemanticTextMechanic,
    )],
) -> Result<(), worth_ui_host_contract::UiHostSurfacePresentationDenial> {
    let mut portal_overlays = std::mem::take(&mut successor.portal_overlays).into_vec();
    let mut semantic_text = std::mem::take(&mut successor.semantic_text).into_vec();
    remove_changed_commands(&mut portal_overlays, &mut semantic_text, changes)?;
    insert_changed_commands(
        &mut portal_overlays,
        &mut semantic_text,
        changes,
        semantic_snapshots,
    )?;
    successor.portal_overlays = portal_overlays.into_boxed_slice();
    successor.semantic_text = semantic_text.into_boxed_slice();
    Ok(())
}

fn apply_recorded_order_edits(
    order: &mut Vec<UiMountedPaintOrderIdentity>,
    edits: &[worth_ui_host_contract::UiMountedPaintOrderEdit],
) -> Result<(), worth_ui_host_contract::UiHostSurfacePresentationDenial> {
    for edit in edits {
        let identity = edit.identity();
        if let Some(index) = order.iter().position(|current| *current == identity) {
            order.remove(index);
        } else if edit.is_removal() {
            return Err(malformed());
        }
        if edit.is_removal() {
            continue;
        }
        let index = match edit.predecessor() {
            None => 0,
            Some(predecessor) => order
                .iter()
                .position(|current| *current == predecessor)
                .and_then(|index| index.checked_add(1))
                .ok_or_else(malformed)?,
        };
        order.insert(index, identity);
    }
    Ok(())
}

fn remove_changed_commands(
    portal_overlays: &mut Vec<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
    semantic_text: &mut Vec<UiHeadlessSemanticTextMechanic>,
    changes: &[worth_ui_host_contract::UiMountedPaintCommandChange],
) -> Result<(), worth_ui_host_contract::UiHostSurfacePresentationDenial> {
    for change in changes {
        let identity = match change {
            worth_ui_host_contract::UiMountedPaintCommandChange::Insert(_) => continue,
            worth_ui_host_contract::UiMountedPaintCommandChange::Replace {
                predecessor, ..
            } => *predecessor,
            worth_ui_host_contract::UiMountedPaintCommandChange::Remove(identity) => *identity,
        };
        if let Some(index) = semantic_text
            .iter()
            .position(|mechanic| mechanic.command_identity() == identity)
        {
            semantic_text.remove(index);
        } else if let Some(index) = portal_overlays.iter().position(|mechanic| {
            worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(mechanic)
                == identity
        }) {
            portal_overlays.remove(index);
        } else {
            return Err(malformed());
        }
    }
    Ok(())
}

fn insert_changed_commands(
    portal_overlays: &mut Vec<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
    semantic_text: &mut Vec<UiHeadlessSemanticTextMechanic>,
    changes: &[worth_ui_host_contract::UiMountedPaintCommandChange],
    semantic_snapshots: &[(
        worth_ui_host_contract::UiMountedPaintCommandIdentity,
        UiHeadlessSemanticTextMechanic,
    )],
) -> Result<(), worth_ui_host_contract::UiHostSurfacePresentationDenial> {
    for command in changes.iter().filter_map(changed_command) {
        match command {
            worth_ui_host_contract::UiMountedPaintCommand::SemanticText { mechanic, .. } => {
                let snapshot = semantic_snapshots
                    .iter()
                    .find(|(identity, _)| *identity == command.identity())
                    .map(|(_, snapshot)| snapshot.clone())
                    .ok_or_else(malformed)?;
                debug_assert_eq!(snapshot.semantic_digest(), mechanic.semantic_digest());
                semantic_text.push(snapshot)
            }
            worth_ui_host_contract::UiMountedPaintCommand::PortalOverlay { mechanic, .. } => {
                portal_overlays.push(*mechanic)
            }
        }
    }
    Ok(())
}

fn changed_command(
    change: &worth_ui_host_contract::UiMountedPaintCommandChange,
) -> Option<&worth_ui_host_contract::UiMountedPaintCommand> {
    match change {
        worth_ui_host_contract::UiMountedPaintCommandChange::Insert(command)
        | worth_ui_host_contract::UiMountedPaintCommandChange::Replace {
            successor: command, ..
        } => Some(command),
        worth_ui_host_contract::UiMountedPaintCommandChange::Remove(_) => None,
    }
}

fn malformed() -> worth_ui_host_contract::UiHostSurfacePresentationDenial {
    worth_ui_host_contract::UiHostSurfacePresentationDenial::MalformedProjection
}
