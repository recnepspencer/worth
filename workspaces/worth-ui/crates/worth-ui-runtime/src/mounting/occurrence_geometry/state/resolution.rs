use std::collections::BTreeMap;

use worth_ui_host_contract::{
    UiMountIncarnation, UiMountedCanonicalBox, UiMountedCanonicalBoxInput,
    UiMountedCoordinateSpace, UiMountedInstanceIdentity, UiSemanticSurfaceIdentity,
};

use super::{UiMountedOccurrenceGeometryRow, UiMountedSurfaceGeometry};
use crate::mounting::{
    UiMountedOccurrenceGeometry, UiMountedOccurrenceGeometryDenial, UiMountedOccurrencePlacement,
};

pub(super) fn exact_region_bounds(
    regions: &BTreeMap<
        (
            UiMountedInstanceIdentity,
            worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
        ),
        UiMountedCanonicalBox,
    >,
    owner: UiMountedInstanceIdentity,
    declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
) -> Result<UiMountedCanonicalBox, UiMountedOccurrenceGeometryDenial> {
    regions
        .get(&(owner, declaration))
        .copied()
        .ok_or(UiMountedOccurrenceGeometryDenial::MissingMosaicRegionGeometry)
}

pub(super) fn validate_placement(
    identity: &crate::mounting::UiMountedIdentityState,
    surface: UiSemanticSurfaceIdentity,
    occurrence: UiMountedOccurrenceGeometry,
) -> Result<(), UiMountedOccurrenceGeometryDenial> {
    let Some(parent) = occurrence.parent() else {
        return (occurrence.bounds().coordinate_space() == UiMountedCoordinateSpace::HostSurface)
            .then_some(())
            .ok_or(UiMountedOccurrenceGeometryDenial::SurfaceCoordinateSpaceMismatch);
    };
    if occurrence.bounds().coordinate_space() != UiMountedCoordinateSpace::GraphNodeLocal {
        return Err(UiMountedOccurrenceGeometryDenial::ParentCoordinateSpaceMismatch);
    }
    let parent = identity
        .projection_instance(parent)
        .ok_or(UiMountedOccurrenceGeometryDenial::UnknownParentOccurrence)?;
    if parent.basis().semantic_surface_identity() != surface {
        return Err(UiMountedOccurrenceGeometryDenial::ParentSurfaceMismatch);
    }
    Ok(())
}

pub(super) fn resolve_surface_geometry(
    inputs: &BTreeMap<UiMountedInstanceIdentity, (UiMountedOccurrenceGeometry, UiMountIncarnation)>,
) -> Result<
    BTreeMap<UiMountedInstanceIdentity, UiMountedCanonicalBox>,
    UiMountedOccurrenceGeometryDenial,
> {
    let mut children = BTreeMap::<UiMountedInstanceIdentity, Vec<_>>::new();
    let mut pending = Vec::new();
    for (instance, (occurrence, _)) in inputs {
        match occurrence.placement() {
            UiMountedOccurrencePlacement::Surface(bounds) => pending.push((*instance, bounds)),
            UiMountedOccurrencePlacement::ParentRelative { parent, .. } => {
                children.entry(parent).or_default().push(*instance);
            }
        }
    }
    let mut resolved = BTreeMap::new();
    while let Some((instance, bounds)) = pending.pop() {
        resolved.insert(instance, bounds);
        for child in children.remove(&instance).unwrap_or_default() {
            let UiMountedOccurrencePlacement::ParentRelative { bounds: local, .. } =
                inputs[&child].0.placement()
            else {
                unreachable!("only parent-relative placements enter the child index")
            };
            let translated = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                x: bounds.x() + local.x(),
                y: bounds.y() + local.y(),
                width: local.width(),
                height: local.height(),
                coordinate_space: UiMountedCoordinateSpace::HostSurface,
            })
            .map_err(|_| UiMountedOccurrenceGeometryDenial::ParentCoordinateSpaceMismatch)?;
            pending.push((child, translated));
        }
    }
    if resolved.len() != inputs.len() {
        return Err(UiMountedOccurrenceGeometryDenial::ParentCycle);
    }
    Ok(resolved)
}

pub(super) fn changed_instances(
    current: Option<&UiMountedSurfaceGeometry>,
    viewport: UiMountedCanonicalBox,
    successor: &BTreeMap<UiMountedInstanceIdentity, UiMountedOccurrenceGeometryRow>,
) -> Vec<UiMountedInstanceIdentity> {
    successor
        .keys()
        .copied()
        .into_iter()
        .filter(|instance| {
            current.is_none_or(|surface| surface.viewport != viewport)
                || current.and_then(|surface| surface.occurrences.get(instance))
                    != successor.get(instance)
        })
        .collect()
}
