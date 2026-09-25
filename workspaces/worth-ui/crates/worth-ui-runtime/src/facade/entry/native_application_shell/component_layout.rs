use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
};

use super::UiNativeMountedComponentLayoutInput;
use crate::mounting::UiMountedOccurrenceGeometry;
use crate::runtime::mosaic::layout::{resolve_component_layout, UiNativeComponentLayoutDenial};

impl UiNativeMountedComponentLayoutInput {
    /// Lays out every mounted component for one logical viewport: viewport
    /// contracts against the viewport, Portal children relative to their
    /// owner, and layout members within the cell their container assigns at
    /// this viewport width. Initial, resized, rebound, and replacement layout
    /// all use this one formula. Returns one occurrence per component, in
    /// the order given.
    pub fn resolve_occurrences(
        viewport: UiMountedCanonicalBox,
        components: &[Self],
    ) -> Result<Box<[UiMountedOccurrenceGeometry]>, UiNativeComponentLayoutDenial> {
        let nodes = components.iter().map(Self::layout_node).collect::<Vec<_>>();
        resolve_component_layout(viewport.width(), viewport.height(), &nodes)?
            .into_iter()
            .map(|placed| {
                let coordinate_space = if placed.parent.is_some() {
                    UiMountedCoordinateSpace::GraphNodeLocal
                } else {
                    UiMountedCoordinateSpace::HostSurface
                };
                let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                    x: placed.bounds.x,
                    y: placed.bounds.y,
                    width: placed.bounds.width,
                    height: placed.bounds.height,
                    coordinate_space,
                })
                .map_err(|_| UiNativeComponentLayoutDenial::NonCanonicalBounds)?;
                Ok(match placed.parent {
                    Some(parent) => UiMountedOccurrenceGeometry::parent_relative(
                        placed.instance,
                        parent,
                        bounds,
                    ),
                    None => UiMountedOccurrenceGeometry::surface(placed.instance, bounds),
                })
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "component_layout_tests.rs"]
mod tests;
