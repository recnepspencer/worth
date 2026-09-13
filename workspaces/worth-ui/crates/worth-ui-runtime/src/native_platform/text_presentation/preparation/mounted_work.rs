use worth_ui_host_contract::{
    UiMountedLogicalDamage, UiMountedPaintCommand, UiMountedPaintCommandChange,
    UiMountedPaintCommandIdentity, UiMountedPresentationWorkView, UiMountedSemanticTextMechanic,
};

pub(crate) type MountedSemanticTextCommand<'work> = (
    UiMountedPaintCommandIdentity,
    &'work UiMountedSemanticTextMechanic,
);

pub(crate) struct MountedSemanticTextWork<'work> {
    pub(crate) mechanics: Vec<MountedSemanticTextCommand<'work>>,
    pub(crate) removals: Vec<UiMountedPaintCommandIdentity>,
    pub(crate) complete: bool,
}

pub(crate) fn mounted_semantic_text(
    work: UiMountedPresentationWorkView<'_>,
) -> MountedSemanticTextWork<'_> {
    match work {
        UiMountedPresentationWorkView::Initial(initial) => complete_text_work(initial.commands()),
        UiMountedPresentationWorkView::Reconstruction(reconstruction) => {
            complete_text_work(reconstruction.commands())
        }
        UiMountedPresentationWorkView::Delta(delta) => delta_text_work(delta.changes()),
        UiMountedPresentationWorkView::Sample(_) | UiMountedPresentationWorkView::Unchanged(_) => {
            MountedSemanticTextWork {
                mechanics: Vec::new(),
                removals: Vec::new(),
                complete: false,
            }
        }
    }
}

fn delta_text_work(changes: &[UiMountedPaintCommandChange]) -> MountedSemanticTextWork<'_> {
    let mut mechanics = Vec::new();
    let mut removals = Vec::new();
    for change in changes {
        match change {
            UiMountedPaintCommandChange::Insert(command) => {
                if let Some(mechanic) = semantic_text_mechanic(command) {
                    mechanics.push((command.identity(), mechanic));
                }
            }
            UiMountedPaintCommandChange::Replace {
                predecessor,
                successor,
            } => {
                if *predecessor != successor.identity()
                    && predecessor.semantic_text_identity_parts().is_some()
                {
                    removals.push(*predecessor);
                }
                if let Some(mechanic) = semantic_text_mechanic(successor) {
                    mechanics.push((successor.identity(), mechanic));
                }
            }
            UiMountedPaintCommandChange::Remove(identity)
                if identity.semantic_text_identity_parts().is_some() =>
            {
                removals.push(*identity);
            }
            UiMountedPaintCommandChange::Remove(_) => {}
        }
    }
    MountedSemanticTextWork {
        mechanics,
        removals,
        complete: false,
    }
}

fn complete_text_work(commands: &[UiMountedPaintCommand]) -> MountedSemanticTextWork<'_> {
    MountedSemanticTextWork {
        mechanics: commands
            .iter()
            .filter_map(|command| {
                semantic_text_mechanic(command).map(|mechanic| (command.identity(), mechanic))
            })
            .collect(),
        removals: Vec::new(),
        complete: true,
    }
}

fn semantic_text_mechanic(
    command: &UiMountedPaintCommand,
) -> Option<&UiMountedSemanticTextMechanic> {
    match command {
        UiMountedPaintCommand::SemanticText { mechanic, .. } => Some(mechanic),
        UiMountedPaintCommand::PortalOverlay { .. } => None,
    }
}

pub(super) fn logical_damage(work: UiMountedPresentationWorkView<'_>) -> &[UiMountedLogicalDamage] {
    match work {
        UiMountedPresentationWorkView::Initial(initial) => initial.damage(),
        UiMountedPresentationWorkView::Delta(delta) => delta.damage(),
        UiMountedPresentationWorkView::Reconstruction(reconstruction) => reconstruction.damage(),
        UiMountedPresentationWorkView::Sample(sample) => sample.damage(),
        UiMountedPresentationWorkView::Unchanged(_) => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::{
        UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostSurfaceIdentity,
        UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
        UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptIssuer,
        UiMountedPortalInputShielding, UiMountedPortalOverlayCompletionInput,
        UiMountedPortalOverlayLifecyclePosture, UiMountedPortalOverlayMechanic, UiMountedRgba8,
        UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    };

    #[test]
    fn non_text_delta_changes_do_not_forge_semantic_text_removals() {
        let first = portal_overlay_command(0.0);
        let second = portal_overlay_command(40.0);
        let changes = [
            UiMountedPaintCommandChange::Insert(first.clone()),
            UiMountedPaintCommandChange::replacement(first.identity(), second.clone()),
            UiMountedPaintCommandChange::Remove(second.identity()),
        ];

        let work = delta_text_work(&changes);

        assert!(work.mechanics.is_empty());
        assert!(work.removals.is_empty());
    }

    fn portal_overlay_command(x: f32) -> UiMountedPaintCommand {
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x,
            y: 0.0,
            width: 32.0,
            height: 24.0,
            coordinate_space: UiMountedCoordinateSpace::Viewport,
        })
        .unwrap();
        let mechanic = UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(
            UiMountedPortalOverlayCompletionInput {
                frame,
                surface,
                binding,
                owner: instance,
                owner_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
                    .unwrap()
                    .receipt_for(instance),
                portal_identity: instance.diagnostic_value(),
                anchor_presentation: UiHostObservationPresentationBasis::new(
                    UiHostSurfaceIdentity::mint_unbound().unwrap(),
                    frame,
                    binding,
                    UiHostPresentationEpoch::issued_by_host(1),
                ),
                anchor_bounds: bounds,
                bounds,
                color: UiMountedRgba8::new(47, 129, 247, 255),
                layer_semantic_order: 0,
                layer_depth: 0,
                clip_bounds: bounds,
                lifecycle: UiMountedPortalOverlayLifecyclePosture::Visible,
                shielding: UiMountedPortalInputShielding::ContentBounds,
            },
        )
        .unwrap();
        UiMountedPaintCommand::PortalOverlay {
            identity: UiMountedPaintCommandIdentity::portal_overlay(&mechanic),
            mechanic,
        }
    }
}
