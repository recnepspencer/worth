//! A prepared platform run offscreen: the same application, driver and host
//! as [`UiPreparedNativePlatform::run`], pumped by its owner instead of a
//! platform event loop. Frames render into an offscreen target, so the run
//! exercises every stage before present and shows nothing on screen.

use worth_ui_host_native::{
    UiNativeOffscreenSettle, UiNativeSubmittedFrameWork, WorthUiNativeOffscreenSession,
};

use super::super::application_driver::UiNativeApplicationDriver;
use super::super::{UiNativeApplicationDefinition, UiNativePlatformOutcome};
use super::UiPreparedNativePlatform;

/// An application running offscreen. The host session is boxed: it holds
/// the whole application and host, tens of kilobytes.
pub struct UiNativeOffscreenPlatformSession {
    session: Box<WorthUiNativeOffscreenSession<UiNativeApplicationDriver>>,
}

#[must_use]
pub enum UiNativeOffscreenStart {
    Started(UiNativeOffscreenPlatformSession),
    /// The application or its host stopped before the session started.
    Ended(UiNativePlatformOutcome),
}

impl UiPreparedNativePlatform {
    /// Starts `application` offscreen, with a client area of `extent`
    /// physical pixels at `scale_factor`.
    pub fn start_offscreen<Application>(
        self,
        application: Application,
        extent: [u32; 2],
        scale_factor: f64,
    ) -> UiNativeOffscreenStart
    where
        Application: UiNativeApplicationDefinition,
    {
        let (driver, event_loop) = match self.prepare_driver(application) {
            Ok(prepared) => prepared,
            Err(denial) => {
                return UiNativeOffscreenStart::Ended(
                    UiNativePlatformOutcome::ApplicationPreparationDenied(denial),
                );
            }
        };
        match event_loop.start_offscreen(driver, extent, scale_factor) {
            Ok(session) => UiNativeOffscreenStart::Started(UiNativeOffscreenPlatformSession {
                session: Box::new(session),
            }),
            Err(report) => {
                UiNativeOffscreenStart::Ended(UiNativePlatformOutcome::from_native(Err(report)))
            }
        }
    }
}

impl UiNativeOffscreenPlatformSession {
    /// Resizes the client area, as one step of a drag would; the next
    /// [`Self::run_until_idle`] delivers the resize.
    pub fn resize(&mut self, extent: [u32; 2]) {
        self.session.resize(extent);
    }

    /// Runs until the host is quiet, for at most `max_turns` turns. A quiet
    /// host has no resize, wake, redraw or deadline left; work the
    /// application holds on its own threads is invisible to it.
    pub fn run_until_idle(&mut self, max_turns: u32) -> UiNativeOffscreenSettle {
        self.session.run_until_idle(max_turns)
    }

    /// Waits up to `within` for a wake posted from the application's own
    /// threads, leaving it for the next [`Self::run_until_idle`]; whether
    /// one arrived.
    pub fn await_wake(&mut self, within: std::time::Duration) -> bool {
        self.session.await_wake(within)
    }

    /// Takes each frame submitted since the last take, with the presentation
    /// work charged to it.
    pub fn take_frame_work(&mut self) -> Vec<UiNativeSubmittedFrameWork> {
        self.session.take_frame_work()
    }

    /// Requests the application close and finishes the run. An application
    /// that does not exit within `max_turns` turns is stopped.
    pub fn close(self, max_turns: u32) -> UiNativePlatformOutcome {
        UiNativePlatformOutcome::from_native(self.session.close(max_turns))
    }
}
