//! Lowers one surface's pointer row change into its appearance fragment.

use super::RetainedPointer;
use worth_ui_host_contract::*;

pub(super) fn lower_fragment(
    frame: &super::super::UiMountedProjectionFrame,
    presentation: UiMountedPresentationAttemptIdentity,
    binding: UiMountedSurfaceBindingRequirement,
    previous: Option<&RetainedPointer>,
    successor: Option<UiMountedPointerAffordanceMechanic>,
    reconstruction: bool,
) -> Result<UiUnpublishedAppearanceFragment, super::super::UiMountedAppearanceOutputDenial> {
    let denial = || super::super::UiMountedAppearanceOutputDenial::PointerLowering;
    let old = previous.map(|row| UiMountedAppearanceMechanic::Pointer(row.mechanic));
    let new = successor.map(UiMountedAppearanceMechanic::Pointer);
    let mut changes = Vec::new();
    match (&old, &new) {
        (Some(old), Some(new)) if old.identity() == new.identity() => {
            changes.push(
                UiMountedAppearanceMechanicChange::replacement(old.identity(), new.clone())
                    .ok_or_else(denial)?,
            );
        }
        _ => {
            if let Some(old) = &old {
                changes.push(UiMountedAppearanceMechanicChange::Remove(old.identity()));
            }
            if let Some(new) = &new {
                changes.push(UiMountedAppearanceMechanicChange::Insert(new.clone()));
            }
        }
    }
    let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        binding.semantic_surface(),
        presentation,
        0,
        0,
        [],
    )
    .map_err(|_| denial())?;
    let output = UiMountedAppearanceFrame::from_runtime_mounting(
        frame.frame_identity(),
        binding.semantic_surface(),
        new,
        order,
    )
    .map_err(|_| denial())?;
    let manifest = previous
        .map(|_| {
            UiMountedAppearancePredecessorManifest::from_runtime_mounting(
                old.iter().map(UiMountedAppearanceMechanic::identity),
                [],
            )
            .ok_or_else(denial)
        })
        .transpose()?;
    let posture = if previous.is_none() {
        UiMountedAppearanceWorkPosture::Initial
    } else if reconstruction {
        UiMountedAppearanceWorkPosture::Reconstruction
    } else {
        UiMountedAppearanceWorkPosture::Delta
    };
    let predecessor = previous.map(|row| row.frame);
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        posture,
        predecessor,
        manifest,
        output,
        changes,
        [],
        previous.is_none(),
    )
    .ok_or_else(denial)?;
    let affinity = UiMountedPresentationAffinity::from_runtime_mounting(
        predecessor,
        frame.frame_identity(),
        binding,
        frame.content_generation(),
        None,
    );
    let pointer = successor
        .or(previous.map(|row| row.mechanic))
        .ok_or_else(denial)?
        .pointer();
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
            surface: binding.semantic_surface(),
            pointer,
        },
        work,
        [],
        binding,
        affinity,
    )
    .map_err(super::super::UiMountedAppearanceOutputDenial::Transport)
}
