use worth_ui::facade::app::{
    UiMountedCanonicalBox, UiMountedSurfaceGeometryBatch, UiNativeMountedComponentLayoutInput,
    WorthUiNativeApplicationShell,
};

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
    let (occurrences, regions) =
        UiNativeMountedComponentLayoutInput::resolve_layout(viewport, components, region_inputs)
            .map_err(|denial| format!("native-layout-components:{denial:?}"))?
            .into_parts();
    Ok(
        UiMountedSurfaceGeometryBatch::new(basis, revision, viewport, occurrences)
            .with_regions(regions),
    )
}
