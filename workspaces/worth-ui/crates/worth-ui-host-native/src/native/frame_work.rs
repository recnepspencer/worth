//! Each submitted frame's presentation work. At submission the work counted
//! since the previous submission is charged to the frame: written to the
//! resize trace when one is kept, and logged in process while an offscreen
//! session keeps a log on this thread. A thread without a log pays nothing.

use std::cell::RefCell;
use std::marker::PhantomData;

use worth_ui_host_contract::UiPresentationWorkCounts;

thread_local! {
    static LOG: RefCell<Option<Vec<UiNativeSubmittedFrameWork>>> = const { RefCell::new(None) };
}

/// One submitted frame and the presentation work charged to it, attempts
/// that never submitted included.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeSubmittedFrameWork {
    frame: u64,
    extent: [u32; 2],
    work: UiPresentationWorkCounts,
}

impl UiNativeSubmittedFrameWork {
    pub const fn frame(&self) -> u64 {
        self.frame
    }

    pub const fn extent(&self) -> [u32; 2] {
        self.extent
    }

    pub const fn work(&self) -> UiPresentationWorkCounts {
        self.work
    }
}

/// Charges the work counted since the previous submission to `frame`, a
/// frame of `extent` just handed to the surface. Taking the counts even when
/// nothing keeps them starts each frame afresh.
pub(crate) fn charge_submitted(frame: u64, extent: [u32; 2]) {
    let work = worth_ui_host_contract::take_presentation_work();
    super::resize_trace::work(frame, work);
    LOG.with_borrow_mut(|log| {
        if let Some(log) = log {
            log.push(UiNativeSubmittedFrameWork {
                frame,
                extent,
                work,
            });
        }
    });
}

/// This thread's work log, kept while the value lives.
pub(crate) struct UiNativeFrameWorkLog {
    /// The log belongs to the thread that presents.
    _thread: PhantomData<*const ()>,
}

impl UiNativeFrameWorkLog {
    /// Starts this thread's log, empty, with work counted before it
    /// discarded. `None` when the thread already keeps one: two logs would
    /// each see the other's frames.
    pub(crate) fn begin() -> Option<Self> {
        LOG.with_borrow_mut(|log| {
            if log.is_some() {
                return None;
            }
            let _ = worth_ui_host_contract::take_presentation_work();
            *log = Some(Vec::new());
            Some(Self {
                _thread: PhantomData,
            })
        })
    }

    /// Takes the frames logged since the last take.
    pub(crate) fn take(&self) -> Vec<UiNativeSubmittedFrameWork> {
        LOG.with_borrow_mut(|log| log.as_mut().map(std::mem::take).unwrap_or_default())
    }
}

impl Drop for UiNativeFrameWorkLog {
    fn drop(&mut self) {
        LOG.with_borrow_mut(|log| *log = None);
    }
}

#[cfg(test)]
#[path = "frame_work_tests.rs"]
mod tests;
