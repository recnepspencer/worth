use worth_ui::facade::app::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
    UiNativeMountedComponentLayoutInput, WorthUiNativeApplicationShell,
};
use worth_ui_platform_pulse::product_world::{dashboard_elements, PlatformPulseMosaicRegion};

mod scroll_region_bounds;

pub(super) fn publish_native_layout(
    shell: &mut WorthUiNativeApplicationShell,
) -> Result<bool, String> {
    let Some(viewport) = shell.native_layout_viewport() else {
        return Ok(false);
    };
    let components = shell.native_component_layout_inputs();
    let regions = shell.native_region_layout_inputs();
    let basis = shell
        .native_layout_basis()
        .map_err(|denial| format!("native-layout-basis:{denial:?}"))?;
    let revision = shell
        .next_native_layout_revision()
        .map_err(|denial| format!("native-layout-revision:{denial:?}"))?;
    let batch = prepare_native_layout_batch(viewport, basis, revision, &components, &regions)?;
    shell
        .complete_native_layout(batch)
        .map_err(|denial| format!("native-layout-completion:{denial:?}"))?;
    Ok(true)
}

pub(super) fn prepare_replacement_native_layout(
    input: worth_ui::facade::app::UiNativeReplacementLayoutInput,
) -> Option<UiMountedSurfaceGeometryBatch> {
    prepare_native_layout_batch(
        input.viewport(),
        input.basis(),
        input.revision(),
        input.components(),
        input.regions(),
    )
    .ok()
}

fn prepare_native_layout_batch(
    viewport: UiMountedCanonicalBox,
    basis: worth_ui::facade::app::UiMountedLayoutBasis,
    revision: worth_ui::facade::app::UiMountedLayoutRevision,
    components: &[UiNativeMountedComponentLayoutInput],
    region_inputs: &[worth_ui::facade::app::UiNativeMountedRegionLayoutInput],
) -> Result<UiMountedSurfaceGeometryBatch, String> {
    let scroll_panels = dashboard_elements()
        .into_iter()
        .filter_map(|element| {
            let panel = element.scroll_panel?;
            Some((format!("component:{}", element.component_id()), panel))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let instances = components
        .iter()
        .map(|component| (component.authored_semantic_identity(), component.instance()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let occurrences =
        UiNativeMountedComponentLayoutInput::resolve_occurrences(viewport, components)
            .map_err(|denial| format!("native-layout-components:{denial:?}"))?
            .into_vec()
            .into_iter()
            .zip(components)
            .map(|(occurrence, component)| {
                let Some(panel) = scroll_panels.get(component.authored_semantic_identity()) else {
                    return Ok(occurrence);
                };
                let owner = format!("component:platform.pulse.component.{}", panel.owner());
                let owner = instances
                    .get(owner.as_str())
                    .ok_or("native-layout-scroll-owner-missing")?;
                scrolled_into_panel(occurrence, component, *owner)
            })
            .collect::<Result<Vec<_>, String>>()?;
    let occurrence_index = occurrences
        .iter()
        .copied()
        .map(|occurrence| (occurrence.instance(), occurrence))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut host_bounds = std::collections::BTreeMap::new();
    let regions = region_inputs
        .iter()
        .map(|region| {
            let owner = resolve_host_bounds(
                region.owner(),
                &occurrence_index,
                &mut host_bounds,
                occurrence_index.len(),
            )?;
            region_bounds(region.region_kind(), viewport, owner)
                .map(|bounds| region.geometry(bounds))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(
        UiMountedSurfaceGeometryBatch::new(basis, revision, viewport, occurrences)
            .with_regions(regions),
    )
}

/// Places a scrolled child relative to the panel owner that scrolls it. Its
/// allocation is already content-local: the product authors it from its
/// panel's content origin, so layout subtracts nothing here and owns no
/// scroll geometry. A layout-cell member is refused: its bounds are relative
/// to its container, so reparenting them would silently drop that offset.
fn scrolled_into_panel(
    occurrence: UiMountedOccurrenceGeometry,
    component: &UiNativeMountedComponentLayoutInput,
    owner: worth_ui::facade::app::UiMountedInstanceIdentity,
) -> Result<UiMountedOccurrenceGeometry, String> {
    if component.layout_container().is_some() {
        return Err(format!(
            "native-layout-scrolled-layout-member:{}",
            component.authored_semantic_identity()
        ));
    }
    let bounds = occurrence.bounds();
    Ok(UiMountedOccurrenceGeometry::parent_relative(
        occurrence.instance(),
        owner,
        canonical_box(
            bounds.x(),
            bounds.y(),
            bounds.width(),
            bounds.height(),
            UiMountedCoordinateSpace::GraphNodeLocal,
        )?,
    ))
}

fn region_bounds(
    region_kind: &str,
    viewport: UiMountedCanonicalBox,
    owner: UiMountedCanonicalBox,
) -> Result<UiMountedCanonicalBox, String> {
    let (x, y, region_width, region_height) =
        match scroll_region_bounds::scroll_region_surface_bounds(region_kind)? {
            Some(bounds) => (bounds.x(), bounds.y(), bounds.width(), bounds.height()),
            None => surface_region_bounds(region_kind, viewport, owner)?,
        };
    canonical_box(
        x - owner.x(),
        y - owner.y(),
        region_width,
        region_height,
        UiMountedCoordinateSpace::GraphNodeLocal,
    )
}

/// Host-surface rectangles for the regions the surface itself allocates.
fn surface_region_bounds(
    region_kind: &str,
    viewport: UiMountedCanonicalBox,
    owner: UiMountedCanonicalBox,
) -> Result<(f32, f32, f32, f32), String> {
    let width = viewport.width();
    let height = viewport.height();
    let bounds = match region_kind {
        kind if kind == PlatformPulseMosaicRegion::Viewport.id() => (0.0, 0.0, width, height),
        kind if kind == PlatformPulseMosaicRegion::Masthead.id() => {
            (235.0, 0.0, (width - 235.0).max(0.0), 58.0)
        }
        kind if kind == PlatformPulseMosaicRegion::EvidenceRail.id() => (0.0, 0.0, 235.0, height),
        kind if kind == PlatformPulseMosaicRegion::ServiceStage.id() => (
            235.0,
            58.0,
            (width - 235.0).max(0.0),
            (height - 58.0).max(0.0),
        ),
        kind if kind == PlatformPulseMosaicRegion::StatusBand.id() => {
            (0.0, 930.0, 235.0, (height - 930.0).max(0.0))
        }
        kind if kind == PlatformPulseMosaicRegion::ServiceTile.id()
            || kind == PlatformPulseMosaicRegion::NativeTile.id() =>
        {
            (owner.x(), owner.y(), owner.width(), owner.height())
        }
        _ => return Err(format!("native-layout-unknown-mosaic-region:{region_kind}")),
    };
    Ok(bounds)
}

fn resolve_host_bounds(
    instance: worth_ui::facade::app::UiMountedInstanceIdentity,
    occurrence_index: &std::collections::BTreeMap<
        worth_ui::facade::app::UiMountedInstanceIdentity,
        UiMountedOccurrenceGeometry,
    >,
    resolved: &mut std::collections::BTreeMap<
        worth_ui::facade::app::UiMountedInstanceIdentity,
        UiMountedCanonicalBox,
    >,
    remaining_depth: usize,
) -> Result<UiMountedCanonicalBox, String> {
    if let Some(bounds) = resolved.get(&instance) {
        return Ok(*bounds);
    }
    if remaining_depth == 0 {
        return Err("native-layout-occurrence-cycle".into());
    }
    let occurrence = occurrence_index
        .get(&instance)
        .ok_or_else(|| "native-layout-region-owner-missing".to_owned())?;
    let bounds = match occurrence.parent() {
        None => occurrence.bounds(),
        Some(parent) => {
            let parent =
                resolve_host_bounds(parent, occurrence_index, resolved, remaining_depth - 1)?;
            let local = occurrence.bounds();
            canonical_box(
                parent.x() + local.x(),
                parent.y() + local.y(),
                local.width(),
                local.height(),
                UiMountedCoordinateSpace::HostSurface,
            )?
        }
    };
    resolved.insert(instance, bounds);
    Ok(bounds)
}

fn canonical_box(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    coordinate_space: UiMountedCoordinateSpace,
) -> Result<UiMountedCanonicalBox, String> {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space,
    })
    .map_err(|denial| format!("native-layout-geometry:{denial:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resized_surface_regions_are_local_to_their_owner() {
        let viewport = canonical_box(
            0.0,
            0.0,
            1_120.0,
            700.0,
            UiMountedCoordinateSpace::HostSurface,
        )
        .unwrap();
        let service_region = region_bounds(
            PlatformPulseMosaicRegion::ServiceStage.id(),
            viewport,
            viewport,
        )
        .unwrap();
        assert_eq!(
            [
                service_region.x(),
                service_region.y(),
                service_region.width(),
                service_region.height()
            ],
            [235.0, 58.0, 885.0, 642.0]
        );
        assert_eq!(viewport.x() + service_region.x(), 235.0);
        assert_eq!(viewport.y() + service_region.y(), 58.0);
    }
}
