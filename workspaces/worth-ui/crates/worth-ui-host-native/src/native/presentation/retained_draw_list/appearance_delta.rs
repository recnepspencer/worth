//! Reversible non-text appearance mutations joined to the ordinary delta transaction.
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::appearance::{
    UiNativeAppearanceCommand, UiNativeAppearanceCommandIdentity,
};
use worth_ui_host_contract::{
    UiMountedAppearanceMechanic as Mechanic, UiMountedAppearanceMechanicChange as Change,
    UiMountedAppearanceMechanicIdentity as Identity, UiUnpublishedAppearanceFragment,
};

impl UiNativeRetainedDrawList {
    pub(in crate::native) fn stage_nontext_appearance_fragment(
        &mut self,
        fragment: &UiUnpublishedAppearanceFragment,
    ) -> Result<Vec<crate::native::presentation::appearance::UiNativeAppearanceCommandUndo>, Denial>
    {
        let affinity = fragment.presentation_affinity();
        if fragment.surface_binding().semantic_surface() != self.surface
            || fragment.surface_binding().binding() != self.binding
            || affinity.successor() != self.frame
            || affinity.surface() != self.surface
            || affinity.binding() != self.binding
            || affinity.content() != self.content
        {
            return Err(Denial::AffinityMismatch);
        }
        let surface = fragment.surface_binding().semantic_surface();
        let appearance = &mut self
            .staged_appearance
            .as_mut()
            .ok_or(Denial::CommandMismatch)?
            .1;
        let mut undo = Vec::new();
        let staging = (|| {
            for change in fragment.work().changes() {
                let staged = match change {
                    Change::Insert(successor) if !is_text(successor) => {
                        let predecessor = appearance_predecessor(appearance, surface)?;
                        let (_, staged) = appearance
                            .stage_command_insert(native_command(successor)?, predecessor)
                            .map_err(|_| Denial::CommandMismatch)?;
                        Some(staged)
                    }
                    Change::Replace {
                        predecessor,
                        successor,
                    } if !is_text(successor) => {
                        if successor.identity() != *predecessor {
                            return Err(Denial::CommandMismatch);
                        }
                        let key = appearance
                            .key_for_identity(&native_identity(predecessor, surface)?)
                            .ok_or(Denial::CommandMismatch)?;
                        Some(
                            appearance
                                .stage_command_replace(key, native_command(successor)?)
                                .map_err(|_| Denial::CommandMismatch)?,
                        )
                    }
                    Change::Remove(identity)
                        if !matches!(identity, Identity::TextForeground { .. }) =>
                    {
                        let key = appearance
                            .key_for_identity(&native_identity(identity, surface)?)
                            .ok_or(Denial::CommandMismatch)?;
                        Some(
                            appearance
                                .stage_command_remove(key)
                                .map_err(|_| Denial::CommandMismatch)?,
                        )
                    }
                    _ => None,
                };
                if let Some(staged) = staged {
                    undo.push(staged);
                }
            }
            Ok(())
        })();
        if let Err(denial) = staging {
            for staged in undo.drain(..).rev() {
                appearance
                    .rollback_text(staged)
                    .expect("partial appearance staging must restore its predecessor");
            }
            return Err(denial);
        }
        Ok(undo)
    }

    pub(in crate::native) fn stage_appearance_overlay(
        &mut self,
        overlay: &worth_ui_host_contract::UiMountedOverlayOrderMechanic,
    ) -> Result<crate::native::presentation::appearance::UiNativeAppearanceCommandUndo, Denial>
    {
        let appearance = &mut self
            .staged_appearance
            .as_mut()
            .ok_or(Denial::CommandMismatch)?
            .1;
        let identity = UiNativeAppearanceCommandIdentity::OverlayOrder {
            surface: overlay.semantic_surface(),
        };
        let key = appearance
            .key_for_identity(&identity)
            .ok_or(Denial::CommandMismatch)?;
        appearance
            .stage_command_replace(
                key,
                UiNativeAppearanceCommand::OverlayOrder(overlay.clone()),
            )
            .map_err(|_| Denial::CommandMismatch)
    }

    pub(in crate::native) fn rollback_appearance_commands(
        &mut self,
        undo: impl IntoIterator<
            Item = crate::native::presentation::appearance::UiNativeAppearanceCommandUndo,
        >,
    ) -> Result<(), Denial> {
        let appearance = &mut self
            .staged_appearance
            .as_mut()
            .ok_or(Denial::CommandMismatch)?
            .1;
        for staged in undo {
            appearance
                .rollback_text(staged)
                .map_err(|_| Denial::CommandMismatch)?;
        }
        Ok(())
    }
}

fn appearance_predecessor(
    appearance: &crate::native::presentation::appearance::UiNativeAppearanceRetained,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> Result<Option<crate::native::presentation::appearance::UiNativeAppearanceCommandKey>, Denial> {
    let overlay = appearance
        .key_for_identity(&UiNativeAppearanceCommandIdentity::OverlayOrder { surface })
        .ok_or(Denial::CommandMismatch)?;
    appearance
        .predecessor(overlay)
        .map_err(|_| Denial::CommandMismatch)
}

fn native_command(mechanic: &Mechanic) -> Result<UiNativeAppearanceCommand, Denial> {
    match mechanic {
        Mechanic::Surface(value) => Ok(UiNativeAppearanceCommand::Surface(value.clone())),
        Mechanic::PortalSurface(value) => {
            Ok(UiNativeAppearanceCommand::PortalSurface(value.clone()))
        }
        Mechanic::Outline(value) => Ok(UiNativeAppearanceCommand::Outline(value.clone())),
        Mechanic::Pointer(value) => Ok(UiNativeAppearanceCommand::PointerAffordance(*value)),
        Mechanic::Backdrop(value) => Ok(UiNativeAppearanceCommand::Backdrop(value.clone())),
        Mechanic::TextForeground(_) => Err(Denial::CommandMismatch),
    }
}

fn native_identity(
    identity: &Identity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> Result<UiNativeAppearanceCommandIdentity, Denial> {
    match identity {
        Identity::Surface(instance) => Ok(UiNativeAppearanceCommandIdentity::Surface(*instance)),
        Identity::PortalSurface(instance) => {
            Ok(UiNativeAppearanceCommandIdentity::PortalSurface(*instance))
        }
        Identity::Outline(instance) => Ok(UiNativeAppearanceCommandIdentity::Outline(*instance)),
        Identity::Pointer {
            pointer,
            surface,
            target,
        } => Ok(UiNativeAppearanceCommandIdentity::PointerAffordance {
            pointer: *pointer,
            surface: *surface,
            target: *target,
        }),
        Identity::Backdrop(identity) => Ok(UiNativeAppearanceCommandIdentity::Backdrop {
            surface,
            identity: identity.clone(),
        }),
        Identity::TextForeground { .. } => Err(Denial::CommandMismatch),
    }
}

fn is_text(mechanic: &Mechanic) -> bool {
    matches!(mechanic, Mechanic::TextForeground(_))
}
