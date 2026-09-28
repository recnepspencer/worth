use crate::capability::{
    MosaicPlacementPolicyDescriptor, MosaicPlacementSource, MosaicPlacementTarget,
    MosaicRegionKindId, MosaicRegionRole, MosaicScrollOwnership, MosaicSizingBehavior,
    MosaicSizingKind, MosaicStateOwnerIdentity, MosaicStateSlotDescriptor, MosaicStateSlotKind,
    SurfaceDescriptor, SurfaceId,
};
use crate::source::WorthUiStructuralLegalityDiagnosticCode;

pub(crate) fn sizing_contract_matches_region(
    behavior: &MosaicSizingBehavior,
    kind: &MosaicSizingKind,
) -> bool {
    match behavior {
        MosaicSizingBehavior::FillsAvailableSpace => matches!(
            kind,
            MosaicSizingKind::Fill | MosaicSizingKind::MinMax | MosaicSizingKind::GrowThenScroll
        ),
        MosaicSizingBehavior::ContentDriven => matches!(
            kind,
            MosaicSizingKind::Hug | MosaicSizingKind::ContentMeasured | MosaicSizingKind::Bounded
        ),
        MosaicSizingBehavior::ViewportBounded => matches!(
            kind,
            MosaicSizingKind::ViewportRelative
                | MosaicSizingKind::Bounded
                | MosaicSizingKind::Fixed
        ),
        MosaicSizingBehavior::OverlayAnchored => matches!(
            kind,
            MosaicSizingKind::Fixed
                | MosaicSizingKind::ViewportRelative
                | MosaicSizingKind::ContentMeasured
        ),
        MosaicSizingBehavior::MissingForDiagnostics => false,
    }
}

pub(crate) fn placement_policy_matches_mount(
    surface: &SurfaceDescriptor,
    region_role: &MosaicRegionRole,
    policy: &MosaicPlacementPolicyDescriptor,
) -> bool {
    policy
        .source()
        .is_some_and(|source| placement_source_matches_surface(source, surface))
        && policy
            .target()
            .is_some_and(|target| placement_target_matches_region(target, region_role))
}

pub(crate) fn region_state_slot_is_legal(
    region_id: &MosaicRegionKindId,
    region_role: &MosaicRegionRole,
    scroll_ownership: &MosaicScrollOwnership,
    state_slot: &MosaicStateSlotDescriptor,
) -> Result<(), WorthUiStructuralLegalityDiagnosticCode> {
    if state_slot.owner_identity()
        != Some(&MosaicStateOwnerIdentity::mosaic_region_kind(
            region_id.clone(),
        ))
    {
        return Err(WorthUiStructuralLegalityDiagnosticCode::IllegalRegionStateOwner);
    }
    match state_slot.kind() {
        MosaicStateSlotKind::ScrollPosition => {
            if !matches!(scroll_ownership, MosaicScrollOwnership::RegionOwned) {
                return Err(WorthUiStructuralLegalityDiagnosticCode::IllegalRegionOwnedScrollState);
            }
        }
        MosaicStateSlotKind::PinnedPosture => {
            if !matches!(
                region_role,
                MosaicRegionRole::Overlay | MosaicRegionRole::Floating | MosaicRegionRole::Modal
            ) {
                return Err(
                    WorthUiStructuralLegalityDiagnosticCode::IllegalPinnedStateSlotForRegionRole,
                );
            }
        }
        MosaicStateSlotKind::SplitterPosition
        | MosaicStateSlotKind::ActiveStackItem
        | MosaicStateSlotKind::RegionVisibility
        | MosaicStateSlotKind::CollapsedPosture
        | MosaicStateSlotKind::FocusedRegion
        | MosaicStateSlotKind::ActivePrimarySurface
        | MosaicStateSlotKind::ActiveAuxiliarySurface
        | MosaicStateSlotKind::SelectionToken
        | MosaicStateSlotKind::DraftInputState => {
            if matches!(scroll_ownership, MosaicScrollOwnership::SurfaceOwned)
                && matches!(state_slot.kind(), MosaicStateSlotKind::FocusedRegion)
            {
                return Err(
                    WorthUiStructuralLegalityDiagnosticCode::IllegalSurfaceOwnedScrollState,
                );
            }
        }
    }
    Ok(())
}

pub(crate) fn mount_state_slot_is_legal(
    surface_id: &SurfaceId,
    surface: &SurfaceDescriptor,
    state_slot: &MosaicStateSlotDescriptor,
) -> Result<(), WorthUiStructuralLegalityDiagnosticCode> {
    if state_slot.owner_identity() != Some(&MosaicStateOwnerIdentity::surface(surface_id.clone())) {
        return Err(WorthUiStructuralLegalityDiagnosticCode::IllegalMountStateOwner);
    }
    match state_slot.kind() {
        MosaicStateSlotKind::ActivePrimarySurface
            if surface.placement_class()
                != &crate::capability::SurfacePlacementClass::primary_region() =>
        {
            Err(WorthUiStructuralLegalityDiagnosticCode::IllegalMountStateSlotKind)
        }
        MosaicStateSlotKind::ActiveAuxiliarySurface
            if surface.placement_class()
                != &crate::capability::SurfacePlacementClass::auxiliary_region() =>
        {
            Err(WorthUiStructuralLegalityDiagnosticCode::IllegalMountStateSlotKind)
        }
        MosaicStateSlotKind::SplitterPosition
        | MosaicStateSlotKind::ActiveStackItem
        | MosaicStateSlotKind::RegionVisibility
        | MosaicStateSlotKind::CollapsedPosture
        | MosaicStateSlotKind::PinnedPosture
        | MosaicStateSlotKind::ScrollPosition
        | MosaicStateSlotKind::FocusedRegion
        | MosaicStateSlotKind::ActivePrimarySurface
        | MosaicStateSlotKind::ActiveAuxiliarySurface
        | MosaicStateSlotKind::SelectionToken
        | MosaicStateSlotKind::DraftInputState => Ok(()),
    }
}

fn placement_source_matches_surface(
    source: &MosaicPlacementSource,
    surface: &SurfaceDescriptor,
) -> bool {
    match source {
        MosaicPlacementSource::SurfaceClass(surface_class) => {
            surface_class == surface.placement_class()
        }
        MosaicPlacementSource::RegionRole(_)
        | MosaicPlacementSource::PluginImperativeMutationForDiagnostics
        | MosaicPlacementSource::MissingForDiagnostics => false,
    }
}

fn placement_target_matches_region(
    target: &MosaicPlacementTarget,
    region_role: &MosaicRegionRole,
) -> bool {
    match target {
        MosaicPlacementTarget::RegionRole(target_role)
        | MosaicPlacementTarget::RegionStack(target_role) => target_role == region_role,
        MosaicPlacementTarget::PluginImperativeMutationForDiagnostics
        | MosaicPlacementTarget::MissingForDiagnostics => false,
    }
}
