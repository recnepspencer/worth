use worth_ui::facade::app::WorthUi;

fn main() {
    let app = WorthUi::app()
        .with_change_profile(worth_ui::facade::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .map(
            worth_ui_runtime::facade::entry::WorthUiCertificationApplicationTransition::activate_headless,
        )
        .expect("fixed certification host should prepare");
    let mut session = app.launch().expect("fixed host should launch");
    // Ordinary callers enter through the operation that owns preparation and
    // presentation; assembled frames cannot be promoted by the caller.
    let _ = session.execute_mounted_frame(
        worth_ui_runtime::facade::mounted::UiMountedFrameRequest::all_bound_surfaces(),
        worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
        1,
        |_| {},
    );
    let _ = session.shutdown();
}
