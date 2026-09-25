use worth_ui::facade::app::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
    UiNativeMountedComponentLayoutInput, WorthUiNativeApplicationShell,
};
use worth_ui_platform_pulse::product_world::dashboard_elements;

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
    let (occurrences, regions) =
        UiNativeMountedComponentLayoutInput::resolve_layout(viewport, components, region_inputs)
            .map_err(|denial| format!("native-layout-components:{denial:?}"))?
            .into_parts();
    let occurrences = occurrences
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
