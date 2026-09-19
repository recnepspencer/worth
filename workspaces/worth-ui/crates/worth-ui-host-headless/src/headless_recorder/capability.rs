pub(super) fn report(
    recorder: &super::WorthUiHeadlessRecorder,
) -> worth_ui_host_contract::WorthUiHostCapabilityReport {
    let mut capabilities = vec![
        worth_ui_host_contract::WorthUiHostCapability::MountedFrameRecording,
        worth_ui_host_contract::WorthUiHostCapability::SemanticFocusPlacement,
    ];
    recorder
        .state
        .borrow()
        .measurement
        .append_capabilities(&mut capabilities);
    worth_ui_host_contract::WorthUiHostCapabilityReport::available(capabilities)
        .with_appearance_profile(reference_appearance_profile())
}

fn reference_appearance_profile() -> worth_ui_host_contract::UiHostAppearanceProfileContract {
    let geometry = worth_ui_host_contract::UiHostAppearanceGeometryQualification::admit([
        worth_ui_host_contract::UiHostAppearanceScaleGeometryQualification::new(
            1_000,
            1,
            worth_ui_host_contract::UiAppearanceLogicalLength::new(1_000)
                .expect("one logical point is valid"),
            worth_ui_host_contract::UiHostAppearanceGeometryQualificationBasis::AnalyticSignedDistancePixelCenter,
        ),
    ])
    .expect("the reference geometry qualification is canonical");
    worth_ui_host_contract::UiHostAppearanceProfileContract::admit(
        "worth-ui-headless-reference-v1",
        1,
        worth_ui_host_contract::UiHostAppearanceMechanicFamily::ALL,
        None,
        geometry,
    )
    .expect("the reference appearance profile is complete")
}
