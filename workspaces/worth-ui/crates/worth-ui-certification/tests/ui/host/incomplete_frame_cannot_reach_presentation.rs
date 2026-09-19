use worth_ui_runtime::facade::mounted::{
    UiMountedPresentationAdmission, UiMountedPresentationAttempt,
};

fn skip_required_preparation(
    admission: UiMountedPresentationAdmission,
) -> UiMountedPresentationAttempt {
    admission.into_attempt()
}

fn skip_semantic_resolution(
    frame: <worth_ui_runtime::facade::mounted::UiPreparedMountedFrame as std::ops::Deref>::Target,
) -> worth_ui_runtime::facade::mounted::UiPreparedMountedFrame {
    frame
}
