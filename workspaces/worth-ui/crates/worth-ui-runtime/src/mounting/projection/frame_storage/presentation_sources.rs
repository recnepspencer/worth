use worth_ui_host_contract::{
    UiMountedDrawableReference, UiMountedNodeProjectionView, UiMountedPaintCommand,
    UiMountedPaintCommandIdentity, UiMountedPaintOrderIdentity, UiMountedSemanticTextMechanic,
};

pub(crate) fn compile(
    nodes: &[UiMountedNodeProjectionView],
    portal_overlays: &[worth_ui_host_contract::UiMountedPortalOverlayMechanic],
    semantic_text: &[UiMountedSemanticTextMechanic],
) -> (Vec<UiMountedPaintCommand>, Vec<UiMountedPaintOrderIdentity>) {
    let mut commands = Vec::new();
    let mut sources = Vec::new();
    for (node_ordinal, node) in nodes.iter().enumerate() {
        for (local_ordinal, reference) in node.drawables().iter().copied().enumerate() {
            let command = command_for(reference, portal_overlays, semantic_text);
            sources.push((
                command.layer_semantic_order(),
                node_ordinal,
                local_ordinal,
                command.identity(),
            ));
            commands.push(command);
        }
    }
    sources.sort_by_key(|source| (source.0, source.1, source.2));
    let order = sources
        .into_iter()
        .map(|source| UiMountedPaintOrderIdentity::for_command(source.3))
        .collect();
    (commands, order)
}

fn command_for(
    reference: UiMountedDrawableReference,
    portal_overlays: &[worth_ui_host_contract::UiMountedPortalOverlayMechanic],
    semantic_text: &[UiMountedSemanticTextMechanic],
) -> UiMountedPaintCommand {
    match reference {
        UiMountedDrawableReference::SemanticText(reference) => {
            let mechanic = semantic_text[usize::from(reference.index())].clone();
            UiMountedPaintCommand::SemanticText {
                identity: UiMountedPaintCommandIdentity::semantic_text(&mechanic),
                mechanic,
            }
        }
        UiMountedDrawableReference::PortalOverlay(reference) => {
            let mechanic = portal_overlays[usize::from(reference.index())];
            UiMountedPaintCommand::PortalOverlay {
                identity: UiMountedPaintCommandIdentity::portal_overlay(&mechanic),
                mechanic,
            }
        }
    }
}
