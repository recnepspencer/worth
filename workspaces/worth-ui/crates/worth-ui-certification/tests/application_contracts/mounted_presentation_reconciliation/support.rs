use worth_ui_runtime::facade::mounted::{
    UiMountedFrameOutcome, UiMountedFramePublicationReceipt, UiMountedFrameRequest,
    UiMountedFrameReuse,
};
use worth_ui_test_support::{
    WorthUiFrameworkTurnCertificationExt, WorthUiMountedFrameExecutionCertificationExt,
};

pub(super) fn prepared_frame(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiPreparedMountedFrame {
    super::prepared(session)
}

pub(super) fn prepared_with_request(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    request: &UiMountedFrameRequest,
) -> worth_ui_runtime::facade::mounted::UiPreparedMountedFrame {
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    session
        .execute_framework_turn(|_| {})
        .unwrap()
        .into_execution()
        .unwrap_or_else(|_| panic!("empty source turn permits mounted preparation"))
        .prepare_mounted_frame(request.clone())
        .unwrap()
}

pub(super) fn classify_reuse(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
    request: &UiMountedFrameRequest,
) -> UiMountedFrameReuse {
    session
        .execute_framework_turn(|_| {})
        .unwrap()
        .into_execution()
        .unwrap_or_else(|_| panic!("empty source turn carries mounted reuse authority"))
        .classify_mounted_frame_reuse(request)
}

pub(super) fn expect_published(outcome: UiMountedFrameOutcome) -> UiMountedFramePublicationReceipt {
    match outcome {
        UiMountedFrameOutcome::Published(receipt) => receipt,
        _ => panic!("scripted predecessor presentation must publish"),
    }
}
