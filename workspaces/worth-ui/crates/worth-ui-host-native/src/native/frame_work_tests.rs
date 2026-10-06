//! A thread keeps at most one work log, which holds exactly the frames
//! submitted while it lives and none of the work counted before it began.

use worth_ui_host_contract::{record_presentation_glyphs, UiPresentationWorkStage};

use super::{charge_submitted, UiNativeFrameWorkLog};

#[test]
fn a_log_keeps_only_frames_submitted_while_it_lives() {
    charge_submitted(1, [8, 8]);
    record_presentation_glyphs(UiPresentationWorkStage::Pins, 5);
    let log = UiNativeFrameWorkLog::begin().expect("no log is kept yet");
    record_presentation_glyphs(UiPresentationWorkStage::Pins, 2);
    charge_submitted(2, [16, 8]);
    let frames = log.take();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].frame(), 2);
    assert_eq!(frames[0].extent(), [16, 8]);
    assert_eq!(frames[0].work().glyphs(UiPresentationWorkStage::Pins), 2);
    assert!(log.take().is_empty());
    drop(log);
    charge_submitted(3, [8, 8]);
    let log = UiNativeFrameWorkLog::begin().expect("the dropped log ended");
    assert!(log.take().is_empty());
}

#[test]
fn a_thread_keeps_one_log_at_a_time() {
    let log = UiNativeFrameWorkLog::begin().expect("no log is kept yet");
    assert!(UiNativeFrameWorkLog::begin().is_none());
    drop(log);
    assert!(UiNativeFrameWorkLog::begin().is_some());
}
