use std::collections::BTreeMap;

use worth_ui_dsl::{
    UiBackdropDeclaration, UiBackdropExtentBasis, UiBackdropIdentity, UiBackdropMotionBasis,
    UiBackdropPresenceBasis, UiBackdropScope, UiPortalDeclarationId,
};

use super::extent::{
    UiOverlayMotionBinding, UiOverlayMotionSnapshot, UiOverlaySurfaceExtentSnapshot,
};
use super::planner::{
    UiOverlayCapacityProfile, UiOverlayCompositionDenial, UiOverlayCompositionInput,
    UiOverlayReservation,
};
use super::snapshot::{
    UiBackdropInstanceIdentity, UiOverlayBackdropInstanceScope, UiOverlayBackdropRow,
    UiOverlayExtent, UiOverlayPortalRow,
};

pub(super) fn current_portals(
    input: &UiOverlayCompositionInput<'_>,
    capacity: UiOverlayCapacityProfile,
) -> Result<Vec<UiOverlayPortalRow>, UiOverlayCompositionDenial> {
    let mut bindings = BTreeMap::new();
    for binding in input.portal_bindings {
        let Some(row) = input
            .portal_snapshot
            .rows()
            .iter()
            .find(|row| row.portal() == binding.portal())
        else {
            return Err(UiOverlayCompositionDenial::MissingPortalSnapshotRow(
                binding.portal(),
            ));
        };
        if row.surface() != input.extent.runtime_surface() {
            continue;
        }
        if bindings
            .insert(binding.portal(), binding.declaration())
            .is_some()
        {
            return Err(UiOverlayCompositionDenial::DuplicatePortalBinding);
        }
    }

    let mut portals = Vec::new();
    let mut previous_ordinal = None;
    for row in input.portal_snapshot.rows() {
        if row.surface() != input.extent.runtime_surface() {
            continue;
        }
        let declaration = bindings.get(&row.portal()).copied().ok_or(
            UiOverlayCompositionDenial::MissingPortalDeclarationBinding(row.portal()),
        )?;
        if previous_ordinal.is_some_and(|ordinal| ordinal >= row.ordinal()) {
            return Err(UiOverlayCompositionDenial::Cycle);
        }
        previous_ordinal = Some(row.ordinal());
        portals.push(UiOverlayPortalRow::new(
            declaration,
            row.portal(),
            row.parent(),
            row.ordinal(),
            row.lifecycle(),
        ));
    }
    if portals.len() > capacity.max_portal_rows {
        return Err(UiOverlayCompositionDenial::PortalRowCapacityExceeded {
            observed: portals.len(),
            maximum: capacity.max_portal_rows,
        });
    }
    Ok(portals)
}

pub(super) fn materialize_backdrops(
    declarations: &[UiBackdropDeclaration],
    portals: &[UiOverlayPortalRow],
    extent: &UiOverlaySurfaceExtentSnapshot,
    motion: Option<&UiOverlayMotionSnapshot>,
    capacity: UiOverlayCapacityProfile,
) -> Result<Vec<UiOverlayBackdropRow>, UiOverlayCompositionDenial> {
    let mut rows = Vec::new();
    for declaration in declarations {
        let materialized = materialize_one(declaration, portals, extent, motion)?;
        rows.extend(materialized);
        ensure_backdrop_capacity(rows.len(), capacity)?;
    }
    Ok(rows)
}

pub(super) fn materialize_one(
    declaration: &UiBackdropDeclaration,
    portals: &[UiOverlayPortalRow],
    extent: &UiOverlaySurfaceExtentSnapshot,
    motion: Option<&UiOverlayMotionSnapshot>,
) -> Result<Vec<UiOverlayBackdropRow>, UiOverlayCompositionDenial> {
    let scopes = match declaration.scope() {
        UiBackdropScope::SurfaceSingleton => {
            if !present(declaration.presence(), portals) {
                return Ok(Vec::new());
            }
            vec![UiOverlayBackdropInstanceScope::SurfaceSingleton]
        }
        UiBackdropScope::PerPortalInstance(portal) => portals
            .iter()
            .filter(|row| row.declaration() == portal)
            .map(|row| UiOverlayBackdropInstanceScope::Portal(row.portal()))
            .collect(),
    };
    scopes
        .into_iter()
        .map(|scope| {
            let identity = UiBackdropInstanceIdentity::new(declaration.identity(), scope);
            let extent_value = resolve_extent(declaration, extent)?;
            let motion_value = resolve_motion(declaration, scope, portals, motion)?;
            Ok(UiOverlayBackdropRow::new(
                identity,
                declaration,
                extent_value,
                motion_value,
            ))
        })
        .collect()
}

fn present(presence: UiBackdropPresenceBasis, portals: &[UiOverlayPortalRow]) -> bool {
    match presence {
        UiBackdropPresenceBasis::Always => true,
        UiBackdropPresenceBasis::WhilePortalPresented(portal) => {
            portals.iter().any(|row| row.declaration() == portal)
        }
    }
}

fn resolve_extent(
    declaration: &UiBackdropDeclaration,
    extent: &UiOverlaySurfaceExtentSnapshot,
) -> Result<UiOverlayExtent, UiOverlayCompositionDenial> {
    match declaration.extent() {
        UiBackdropExtentBasis::SurfaceViewport(surface)
            if surface == extent.declaration_surface() =>
        {
            Ok(UiOverlayExtent::SurfaceViewport {
                basis: declaration.extent(),
                bounds: extent.viewport(),
            })
        }
        UiBackdropExtentBasis::PresentedMosaicRegion { surface, region }
            if surface == extent.declaration_surface() =>
        {
            extent
                .region(region)
                .map(|region_extent| UiOverlayExtent::PresentedMosaicRegion {
                    basis: declaration.extent(),
                    bounds: region_extent.bounds(),
                })
                .ok_or(UiOverlayCompositionDenial::MissingRegionExtent {
                    backdrop: declaration.identity(),
                    region,
                })
        }
        _ => Err(UiOverlayCompositionDenial::ForeignSurfaceExtent {
            backdrop: declaration.identity(),
        }),
    }
}

fn resolve_motion(
    declaration: &UiBackdropDeclaration,
    scope: UiOverlayBackdropInstanceScope,
    portals: &[UiOverlayPortalRow],
    motion: Option<&UiOverlayMotionSnapshot>,
) -> Result<Option<UiOverlayMotionBinding>, UiOverlayCompositionDenial> {
    let UiBackdropMotionBasis::PortalPresentation(target) = declaration.motion() else {
        return Ok(None);
    };
    let target = matching_portals(declaration.identity(), target, scope, portals)?;
    let binding = motion.ok_or(UiOverlayCompositionDenial::MissingMotionBasis {
        backdrop: declaration.identity(),
        portal: target.declaration(),
    })?;
    binding
        .lookup(target.declaration(), target.portal())
        .map(Some)
        .ok_or(UiOverlayCompositionDenial::MissingMotionBinding {
            backdrop: declaration.identity(),
            portal: target.declaration(),
        })
}

fn matching_portals(
    backdrop: UiBackdropIdentity,
    target: UiPortalDeclarationId,
    scope: UiOverlayBackdropInstanceScope,
    portals: &[UiOverlayPortalRow],
) -> Result<UiOverlayPortalRow, UiOverlayCompositionDenial> {
    let mut matches = portals.iter().filter(|row| row.declaration() == target);
    if let UiOverlayBackdropInstanceScope::Portal(portal) = scope {
        return matches.find(|row| row.portal() == portal).cloned().ok_or(
            UiOverlayCompositionDenial::MissingPortalAnchor {
                backdrop,
                portal: target,
            },
        );
    }
    let first = matches
        .next()
        .cloned()
        .ok_or(UiOverlayCompositionDenial::MissingPortalAnchor {
            backdrop,
            portal: target,
        })?;
    if matches.next().is_some() {
        return Err(UiOverlayCompositionDenial::AmbiguousPortalAnchor {
            backdrop,
            portal: target,
        });
    }
    Ok(first)
}

pub(super) fn reserve(
    portal_rows: usize,
    backdrop_rows: usize,
    order_rows: usize,
    relation_edges: usize,
    capacity: UiOverlayCapacityProfile,
) -> Result<UiOverlayReservation, UiOverlayCompositionDenial> {
    if portal_rows > capacity.max_portal_rows {
        return Err(UiOverlayCompositionDenial::PortalRowCapacityExceeded {
            observed: portal_rows,
            maximum: capacity.max_portal_rows,
        });
    }
    ensure_backdrop_capacity(backdrop_rows, capacity)?;
    if order_rows > capacity.max_order_rows {
        return Err(UiOverlayCompositionDenial::OverlayOrderCapacityExceeded {
            observed: order_rows,
            maximum: capacity.max_order_rows,
        });
    }
    if relation_edges > capacity.max_relation_edges {
        return Err(UiOverlayCompositionDenial::RelationEdgeCapacityExceeded {
            observed: relation_edges,
            maximum: capacity.max_relation_edges,
        });
    }
    Ok(UiOverlayReservation {
        portal_rows,
        backdrop_rows,
        order_rows,
        relation_edges,
    })
}

pub(super) fn ensure_backdrop_capacity(
    observed: usize,
    capacity: UiOverlayCapacityProfile,
) -> Result<(), UiOverlayCompositionDenial> {
    if observed > capacity.max_backdrop_rows {
        return Err(UiOverlayCompositionDenial::BackdropRowCapacityExceeded {
            observed,
            maximum: capacity.max_backdrop_rows,
        });
    }
    Ok(())
}
