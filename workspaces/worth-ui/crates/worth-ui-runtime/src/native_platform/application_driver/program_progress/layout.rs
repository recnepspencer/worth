use crate::facade::entry::UiNativeMountedComponentLayoutInput;
use crate::facade::WorthUiNativeApplicationShell;
use crate::mounting::UiMountedSurfaceGeometryBatch;

/// Completes the exact occurrence geometry for the declarative frame-program
/// driver. Applications with a live runtime remain responsible for their own
/// layout; this owner covers the bounded program path used without one.
pub(in super::super) fn complete_program_layout(
    shell: &mut WorthUiNativeApplicationShell,
) -> Result<(), ()> {
    if !shell.native_program_layout_required() {
        return Ok(());
    }
    let viewport = shell.native_layout_viewport().ok_or(())?;
    let (occurrences, regions) = UiNativeMountedComponentLayoutInput::resolve_layout(
        viewport,
        &shell.native_component_layout_inputs(),
        &shell.native_region_layout_inputs(),
    )
    .map_err(|_| ())?
    .into_parts();
    let basis = shell.native_layout_basis().map_err(|_| ())?;
    let revision = shell.next_native_layout_revision().map_err(|_| ())?;
    shell
        .complete_native_layout(
            UiMountedSurfaceGeometryBatch::new(basis, revision, viewport, occurrences)
                .with_regions(regions),
        )
        .map_err(|_| ())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::complete_program_layout;
    use crate::certification_support::ScriptedPresentationHost;
    use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host_and_viewport_allocation;

    #[test]
    fn same_binding_viewport_resize_recompletes_program_owned_geometry() {
        let host = ScriptedPresentationHost::native_display();
        let mut shell = source_backed_component_app_with_host_and_viewport_allocation(host)
            .launch_native_surface()
            .expect("native viewport shell should launch");
        shell.observe_native_viewport_readiness([800, 600], 1_000, false);
        assert!(shell.native_program_layout_required());
        complete_program_layout(&mut shell).expect("initial program layout should complete");
        assert!(!shell.native_program_layout_required());

        shell.observe_native_viewport_readiness([960, 720], 1_000, false);
        assert!(
            shell.native_program_layout_required(),
            "same-binding viewport change invalidates program-owned geometry"
        );
        complete_program_layout(&mut shell).expect("resized program layout should complete");
        assert!(!shell.native_program_layout_required());
    }
}
