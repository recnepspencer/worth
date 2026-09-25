use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
};

use super::{UiNativeMountedComponentLayoutInput, UiNativeMountedRegionLayoutInput};
use crate::mounting::{UiMountedMosaicRegionGeometry, UiMountedOccurrenceGeometry};
use crate::runtime::mosaic::layout::{
    resolve_component_layout, UiMosaicLayoutBox, UiNativeComponentLayoutDenial,
};

/// One viewport's resolved layout: an occurrence per component, in the order
/// given, and each mounted Mosaic region local to the component that owns it.
#[derive(Clone, Debug)]
pub struct UiNativeResolvedLayout {
    occurrences: Vec<UiMountedOccurrenceGeometry>,
    regions: Vec<UiMountedMosaicRegionGeometry>,
}

impl UiNativeResolvedLayout {
    pub fn occurrences(&self) -> &[UiMountedOccurrenceGeometry] {
        &self.occurrences
    }

    pub fn regions(&self) -> &[UiMountedMosaicRegionGeometry] {
        &self.regions
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<UiMountedOccurrenceGeometry>,
        Vec<UiMountedMosaicRegionGeometry>,
    ) {
        (self.occurrences, self.regions)
    }
}

impl UiNativeMountedComponentLayoutInput {
    /// Lays out every mounted component for one logical viewport: viewport
    /// contracts against the viewport, Portal children relative to their
    /// owner, and layout members within the cell their container assigns at
    /// this viewport width. Each mounted region takes the placement its owner
    /// declares, within the box the owner's own allocation resolved against.
    /// Initial, resized, rebound, and replacement layout all use this one
    /// formula.
    pub fn resolve_layout(
        viewport: UiMountedCanonicalBox,
        components: &[Self],
        regions: &[UiNativeMountedRegionLayoutInput],
    ) -> Result<UiNativeResolvedLayout, UiNativeComponentLayoutDenial> {
        let nodes = components.iter().map(Self::layout_node).collect::<Vec<_>>();
        let placed = resolve_component_layout(viewport.width(), viewport.height(), &nodes)?;
        let occurrences = placed
            .iter()
            .map(|placed| {
                let Some(parent) = placed.parent else {
                    let bounds = canonical(placed.bounds, UiMountedCoordinateSpace::HostSurface)?;
                    return Ok(UiMountedOccurrenceGeometry::surface(
                        placed.instance,
                        bounds,
                    ));
                };
                let bounds = canonical(placed.bounds, UiMountedCoordinateSpace::GraphNodeLocal)?;
                Ok(UiMountedOccurrenceGeometry::parent_relative(
                    placed.instance,
                    parent,
                    bounds,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let regions = regions
            .iter()
            .map(|region| {
                let (owner, input) = placed
                    .iter()
                    .zip(components)
                    .find(|(owner, _)| owner.instance == region.owner())
                    .ok_or(UiNativeComponentLayoutDenial::MissingRegionOwner)?;
                let placement = input
                    .region_allocation(region.region_kind())
                    .ok_or(UiNativeComponentLayoutDenial::MissingRegionAllocation)?;
                let bounds = canonical(
                    owner.region_box(placement),
                    UiMountedCoordinateSpace::GraphNodeLocal,
                )?;
                Ok(region.geometry(bounds))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(UiNativeResolvedLayout {
            occurrences,
            regions,
        })
    }
}

fn canonical(
    bounds: UiMosaicLayoutBox,
    coordinate_space: UiMountedCoordinateSpace,
) -> Result<UiMountedCanonicalBox, UiNativeComponentLayoutDenial> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
        coordinate_space,
    })
    .map_err(|_| UiNativeComponentLayoutDenial::NonCanonicalBounds)
}

#[cfg(test)]
#[path = "component_layout_tests.rs"]
mod tests;
