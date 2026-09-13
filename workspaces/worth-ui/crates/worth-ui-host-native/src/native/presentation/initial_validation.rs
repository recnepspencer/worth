use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedFrameConsumptionView, UiMountedPaintCommand,
    UiMountedPresentationWorkView,
};

use super::{UiNativePresentationAccess, UiNativePresentationFailure, UiNativeRasterOperation};

pub(super) struct ValidatedInitial {
    commands: Box<[UiMountedPaintCommand]>,
}

pub(super) fn validate_initial(
    view: &UiMountedFrameConsumptionView<'_>,
    retained: &super::UiNativeRetainedDrawList,
) -> Result<ValidatedInitial, UiHostSurfacePresentationDenial> {
    let UiMountedPresentationWorkView::Initial(initial) = view.presentation_work() else {
        return Err(UiHostSurfacePresentationDenial::AdapterDeclined);
    };
    if (initial.commands().is_empty() && !retained.has_appearance())
        || initial.order().len() != initial.commands().len()
        || !initial.order_integrity().admits(initial.order())
    {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    let commands = initial
        .commands()
        .iter()
        .map(|command| match command {
            UiMountedPaintCommand::SemanticText { identity, mechanic }
                if *identity
                    == worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text(
                        mechanic,
                    )
                    && initial
                        .projection()
                        .semantic_text()
                        .rows()
                        .contains(mechanic) =>
            {
                Ok((*identity, command.clone()))
            }
            UiMountedPaintCommand::PortalOverlay { identity, mechanic }
                if *identity
                    == worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(
                        mechanic,
                    )
                    && initial
                        .projection()
                        .portal_overlays()
                        .rows()
                        .contains(mechanic) =>
            {
                Ok((*identity, command.clone()))
            }
            _ => Err(UiHostSurfacePresentationDenial::MalformedProjection),
        })
        .collect::<Result<std::collections::HashMap<_, _>, _>>()?;
    if commands.len() != initial.commands().len() {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    let ordered = initial
        .order()
        .iter()
        .map(|order| {
            commands
                .get(&order.command())
                .cloned()
                .ok_or(UiHostSurfacePresentationDenial::MalformedProjection)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ValidatedInitial {
        commands: ordered.into_boxed_slice(),
    })
}

pub(super) fn initial_operations(
    retained: &super::UiNativeRetainedDrawList,
    graphics: &UiNativePresentationAccess,
    atlas: &crate::native::text_atlas::UiNativeTextAtlas,
    initial: &ValidatedInitial,
) -> Result<Vec<UiNativeRasterOperation>, UiNativePresentationFailure> {
    if retained.has_appearance() {
        return retained
            .complete_appearance_operations(
                super::raster::UiNativeRasterBasis::from_presentation_access(graphics),
                atlas,
            )
            .map_err(|_| before_effects_malformed());
    }
    let mut operations = Vec::new();
    for command in &initial.commands {
        match command {
            UiMountedPaintCommand::PortalOverlay { mechanic, .. } => {
                if let Some(operation) = retained
                    .appearance_portal_surface_operation(mechanic.owner(), graphics.extent())
                    .map_err(|_| before_effects_malformed())?
                {
                    operations.push(operation);
                    continue;
                }
                let rect = super::raster::raster_portal_overlay(*mechanic, graphics)
                    .map_err(|_| before_effects_malformed())?;
                operations.push(UiNativeRasterOperation::FilledRect {
                    rect,
                    source_rgba8: mechanic.color().channels(),
                });
            }
            UiMountedPaintCommand::SemanticText { identity, .. } => operations.extend(
                retained
                    .plan_text_commands(
                        *identity,
                        atlas,
                        super::raster::UiNativeRasterBasis::from_presentation_access(graphics),
                    )
                    .map_err(|_| before_effects_malformed())?
                    .iter()
                    .copied()
                    .map(UiNativeRasterOperation::Glyph),
            ),
        }
    }
    Ok(operations)
}

fn before_effects_malformed() -> UiNativePresentationFailure {
    UiNativePresentationFailure::BeforeEffects(UiHostSurfacePresentationDenial::MalformedProjection)
}
