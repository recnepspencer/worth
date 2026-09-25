use std::collections::BTreeMap;

use worth_ui_host_contract::UiMountedInstanceIdentity;

use super::track_allocation::{allocate_axis, UiMosaicAxisSpan};
use crate::capability::{
    ComponentAllocationMeasurementContract, ComponentId, ComponentViewportAxisPlacement,
    ComponentViewportRegion, MosaicLayoutCell, MosaicLayoutContract, MosaicResponsiveLayout,
};

/// A logical-point box, relative to the viewport or to a parent component.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiMosaicLayoutBox {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

/// What a component's box is placed relative to.
#[derive(Clone, Copy, Debug)]
pub(crate) enum UiMosaicLayoutParent<'a> {
    Viewport,
    /// A Portal child: authored against the viewport, placed relative to its
    /// owner.
    Portal(UiMountedInstanceIdentity),
    /// A layout member: placed within the cell its container assigns it.
    Container {
        container: UiMountedInstanceIdentity,
        member: &'a ComponentId,
    },
}

/// One mounted component's layout inputs.
#[derive(Clone, Copy, Debug)]
pub(crate) struct UiMosaicLayoutNode<'a> {
    pub(crate) instance: UiMountedInstanceIdentity,
    pub(crate) allocation: Option<ComponentAllocationMeasurementContract>,
    pub(crate) layout: Option<&'a MosaicResponsiveLayout>,
    pub(crate) parent: UiMosaicLayoutParent<'a>,
}

/// One component's resolved box, relative to its parent when it has one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiMosaicPlacedComponent {
    pub(crate) instance: UiMountedInstanceIdentity,
    pub(crate) parent: Option<UiMountedInstanceIdentity>,
    pub(crate) bounds: UiMosaicLayoutBox,
}

/// Why mounted components cannot be laid out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeComponentLayoutDenial {
    /// A mounted component declares no allocation contract.
    MissingAllocation,
    /// A layout-cell component has no mounted container with a layout.
    MissingLayoutContainer,
    /// The selected layout variant assigns the member no cell.
    UnplacedMember,
    /// A container is laid out inside itself.
    ContainerCycle,
    /// A resolved box is not a canonical finite box.
    NonCanonicalBounds,
}

/// Resolves every component's box for one logical viewport. Containers
/// select their layout variant by the viewport width, allocate their tracks
/// across their own box, and place each member within its assigned cell.
pub(crate) fn resolve_component_layout(
    viewport_width: f32,
    viewport_height: f32,
    nodes: &[UiMosaicLayoutNode<'_>],
) -> Result<Vec<UiMosaicPlacedComponent>, UiNativeComponentLayoutDenial> {
    let viewport = UiMosaicLayoutBox {
        x: 0.0,
        y: 0.0,
        width: viewport_width,
        height: viewport_height,
    };
    let mut resolution = Resolution {
        viewport,
        nodes: nodes.iter().map(|node| (node.instance, node)).collect(),
        resolved: BTreeMap::new(),
    };
    nodes
        .iter()
        .map(|node| {
            Ok(UiMosaicPlacedComponent {
                instance: node.instance,
                parent: match node.parent {
                    UiMosaicLayoutParent::Viewport => None,
                    UiMosaicLayoutParent::Portal(owner) => Some(owner),
                    UiMosaicLayoutParent::Container { container, .. } => Some(container),
                },
                bounds: resolution.resolve(node.instance, 0)?,
            })
        })
        .collect()
}

struct Resolution<'n, 'a> {
    viewport: UiMosaicLayoutBox,
    nodes: BTreeMap<UiMountedInstanceIdentity, &'n UiMosaicLayoutNode<'a>>,
    resolved: BTreeMap<UiMountedInstanceIdentity, UiMosaicLayoutBox>,
}

impl Resolution<'_, '_> {
    fn resolve(
        &mut self,
        instance: UiMountedInstanceIdentity,
        depth: usize,
    ) -> Result<UiMosaicLayoutBox, UiNativeComponentLayoutDenial> {
        if let Some(bounds) = self.resolved.get(&instance) {
            return Ok(*bounds);
        }
        if depth > self.nodes.len() {
            return Err(UiNativeComponentLayoutDenial::ContainerCycle);
        }
        let node = *self
            .nodes
            .get(&instance)
            .ok_or(UiNativeComponentLayoutDenial::MissingLayoutContainer)?;
        let contract = node
            .allocation
            .ok_or(UiNativeComponentLayoutDenial::MissingAllocation)?;
        let bounds = match (contract, node.parent) {
            (
                ComponentAllocationMeasurementContract::LayoutCell(region),
                UiMosaicLayoutParent::Container { container, member },
            ) => {
                let container_box = self.resolve(container, depth + 1)?;
                let layout = self.nodes[&container]
                    .layout
                    .ok_or(UiNativeComponentLayoutDenial::MissingLayoutContainer)?
                    .select(self.viewport.width);
                let cell = layout
                    .member_cell(member)
                    .ok_or(UiNativeComponentLayoutDenial::UnplacedMember)?;
                region_box(region, cell_box(layout, cell, container_box))
            }
            (ComponentAllocationMeasurementContract::LayoutCell(_), _) => {
                return Err(UiNativeComponentLayoutDenial::MissingLayoutContainer);
            }
            (_, UiMosaicLayoutParent::Container { .. }) => {
                return Err(UiNativeComponentLayoutDenial::UnplacedMember);
            }
            (contract, _) => viewport_box(contract, self.viewport),
        };
        self.resolved.insert(instance, bounds);
        Ok(bounds)
    }
}

/// A cell's box relative to its container's origin.
fn cell_box(
    layout: &MosaicLayoutContract,
    cell: MosaicLayoutCell,
    container: UiMosaicLayoutBox,
) -> UiMosaicLayoutBox {
    let columns = allocate_axis(
        layout.column_tracks(),
        f32::from(layout.column_gap_logical_points()),
        container.width,
    );
    let rows = allocate_axis(
        layout.row_tracks(),
        f32::from(layout.row_gap_logical_points()),
        container.height,
    );
    let horizontal = span(&columns, cell.column(), cell.column_span());
    let vertical = span(&rows, cell.row(), cell.row_span());
    UiMosaicLayoutBox {
        x: horizontal.start,
        y: vertical.start,
        width: horizontal.extent,
        height: vertical.extent,
    }
}

/// The start and extent covered by `count` tracks from `first`, including the
/// gaps between them.
fn span(tracks: &[UiMosaicAxisSpan], first: u16, count: u16) -> UiMosaicAxisSpan {
    let start = tracks[usize::from(first)].start;
    let last = tracks[usize::from(first) + usize::from(count) - 1];
    UiMosaicAxisSpan {
        start,
        extent: last.start + last.extent - start,
    }
}

fn region_box(region: ComponentViewportRegion, reference: UiMosaicLayoutBox) -> UiMosaicLayoutBox {
    let horizontal = resolve_axis(region.horizontal(), reference.width);
    let vertical = resolve_axis(region.vertical(), reference.height);
    UiMosaicLayoutBox {
        x: reference.x + horizontal.start,
        y: reference.y + vertical.start,
        width: horizontal.extent,
        height: vertical.extent,
    }
}

fn viewport_box(
    contract: ComponentAllocationMeasurementContract,
    viewport: UiMosaicLayoutBox,
) -> UiMosaicLayoutBox {
    match contract {
        ComponentAllocationMeasurementContract::FillViewport => viewport,
        ComponentAllocationMeasurementContract::ViewportInset(inset) => {
            let horizontal = f32::from(inset.horizontal_logical_points());
            let vertical = f32::from(inset.vertical_logical_points());
            UiMosaicLayoutBox {
                x: horizontal,
                y: vertical,
                width: (viewport.width - 2.0 * horizontal).max(0.0),
                height: (viewport.height - 2.0 * vertical).max(0.0),
            }
        }
        ComponentAllocationMeasurementContract::ViewportRegion(region) => {
            region_box(region, viewport)
        }
        ComponentAllocationMeasurementContract::LayoutCell(_) => {
            unreachable!("a layout cell is placed by its container, not the viewport")
        }
        ComponentAllocationMeasurementContract::FixedLogicalSize { width, height } => {
            UiMosaicLayoutBox {
                x: 0.0,
                y: 0.0,
                width: f32::from(width),
                height: f32::from(height),
            }
        }
    }
}

/// The span one placement takes along an axis `available` points long.
fn resolve_axis(axis: ComponentViewportAxisPlacement, available: f32) -> UiMosaicAxisSpan {
    match axis {
        ComponentViewportAxisPlacement::FixedFromStart {
            start_logical_points,
            extent_logical_points,
        } => UiMosaicAxisSpan {
            start: f32::from(start_logical_points),
            extent: f32::from(extent_logical_points),
        },
        ComponentViewportAxisPlacement::StretchBetween {
            start_logical_points,
            end_logical_points,
        } => {
            let start = f32::from(start_logical_points);
            UiMosaicAxisSpan {
                start,
                extent: (available - start - f32::from(end_logical_points)).max(0.0),
            }
        }
        ComponentViewportAxisPlacement::FixedFromEnd {
            end_logical_points,
            extent_logical_points,
        } => {
            let extent = f32::from(extent_logical_points);
            UiMosaicAxisSpan {
                start: (available - f32::from(end_logical_points) - extent).max(0.0),
                extent,
            }
        }
    }
}

#[cfg(test)]
#[path = "component_layout_tests.rs"]
mod tests;
