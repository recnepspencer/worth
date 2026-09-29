//! The offscreen pump: runs a host without a platform window or event loop.
//!
//! The pump replays ordinary platform loop turns: each turn starts new
//! events, dispatches a pending `Resized` or other window event, the posted
//! wakes, then a requested redraw, and ends about to wait. It does not replay
//! a platform's modal border drag, where the loop neither starts events nor
//! waits, so about-to-wait work runs in each offscreen step that a Windows
//! drag would defer. Frames render into an offscreen target of the
//! swapchain's extent, format and usage, so every stage before present runs
//! unchanged. What the pump cannot show is what happens after present:
//! pacing, composition and when a frame reaches the screen.

use std::rc::Rc;
use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;

use super::loop_control::{UiNativeLoopControl, UiNativeOffscreenLoopControl};
use super::resume::UiNativeWindowSource;
use super::window_port::UiNativeOffscreenWindow;
use super::{
    UiNativeEventLoopApplication, UiNativeEventLoopClient, UiNativeEventLoopRunDenial,
    UiNativeEventLoopRunReport, UiNativeEventLoopStopReport, WorthUiNativeEventLoop,
};
use crate::native::frame_work::{UiNativeFrameWorkLog, UiNativeSubmittedFrameWork};
use crate::native::readiness::{UiNativeOffscreenWakes, UiNativeWakeSender};

/// A host running offscreen, advanced only by its owner. Dropping a session
/// without [`Self::close`] stops the host and finishes it unreported.
#[must_use = "a session runs only while its owner advances it, and reports only when closed"]
pub struct WorthUiNativeOffscreenSession<Client: UiNativeEventLoopClient> {
    /// Present until the session finishes, by close or by drop.
    application: Option<UiNativeEventLoopApplication<Client>>,
    control: UiNativeOffscreenLoopControl,
    window: Rc<UiNativeOffscreenWindow>,
    wakes: Arc<UiNativeOffscreenWakes>,
    log: UiNativeFrameWorkLog,
}

/// How [`WorthUiNativeOffscreenSession::run_until_idle`] stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeOffscreenSettle {
    /// After `turns` turns the host was quiet: no resize, wake, redraw or
    /// deadline remained. Work the application holds on its own threads,
    /// whose wake has not yet been posted, is invisible to the host.
    Idle { turns: u32 },
    /// The host asked to exit after `turns` turns.
    Exited { turns: u32 },
    /// Work remained after the turn limit.
    Busy,
}

impl WorthUiNativeEventLoop {
    /// Starts the host offscreen, with a client area of `extent` physical
    /// pixels at `scale_factor`, and resumes it as a platform loop would.
    /// A thread runs one offscreen session at a time.
    pub fn start_offscreen<Client: UiNativeEventLoopClient>(
        self,
        client: Client,
        extent: [u32; 2],
        scale_factor: f64,
    ) -> Result<WorthUiNativeOffscreenSession<Client>, UiNativeEventLoopStopReport> {
        let Some(log) = UiNativeFrameWorkLog::begin() else {
            let cause = UiNativeEventLoopRunDenial::EventLoopCreation;
            return Err(super::run::stop_before_callbacks(self.state, client, cause));
        };
        let wakes = Arc::new(UiNativeOffscreenWakes::default());
        let application = self.start(client, UiNativeWakeSender::Offscreen(Arc::clone(&wakes)))?;
        let mut session = WorthUiNativeOffscreenSession {
            application: Some(application),
            control: UiNativeOffscreenLoopControl::new(),
            window: Rc::new(UiNativeOffscreenWindow::new(extent, scale_factor)),
            wakes,
            log,
        };
        let source = UiNativeWindowSource::Offscreen(Rc::clone(&session.window));
        session.turn(Some(UiNativeOffscreenEvent::Resumed(source)), 0);
        Ok(session)
    }
}

/// The event a turn dispatches before the pending resize and the wakes.
enum UiNativeOffscreenEvent {
    Resumed(UiNativeWindowSource<'static>),
    Window(WindowEvent),
}

impl<Client: UiNativeEventLoopClient> WorthUiNativeOffscreenSession<Client> {
    /// Resizes the client area, as one step of a drag would. The host reads
    /// the new extent at once and receives its `Resized` event in the next
    /// turn [`Self::run_until_idle`] runs.
    pub fn resize(&mut self, extent: [u32; 2]) {
        self.window.resize(extent);
    }

    /// Runs turns until the host is quiet, waiting for a deadline or a wake
    /// as a platform loop would, for at most `max_turns` turns.
    pub fn run_until_idle(&mut self, max_turns: u32) -> UiNativeOffscreenSettle {
        for turns in 0..max_turns {
            if self.control.exiting() {
                return UiNativeOffscreenSettle::Exited { turns };
            }
            let mut wakes = self.wakes.take(None);
            if wakes == 0 && !self.window.redraw_pending() && !self.window.resize_pending() {
                match self.control.control_flow() {
                    ControlFlow::Wait => return UiNativeOffscreenSettle::Idle { turns },
                    ControlFlow::WaitUntil(deadline) => wakes = self.wakes.take(Some(deadline)),
                    ControlFlow::Poll => {}
                }
            }
            self.turn(None, wakes);
        }
        if self.control.exiting() {
            UiNativeOffscreenSettle::Exited { turns: max_turns }
        } else {
            UiNativeOffscreenSettle::Busy
        }
    }

    /// Waits up to `within` for a wake posted from the application's own
    /// threads, leaving it for the next [`Self::run_until_idle`]; whether
    /// one arrived. A quiet host whose application posts nothing within a
    /// window longer than its threads' latency has settled.
    pub fn await_wake(&mut self, within: std::time::Duration) -> bool {
        self.wakes.await_posted(std::time::Instant::now() + within)
    }

    /// Takes each frame submitted since the last take, with the presentation
    /// work charged to it.
    pub fn take_frame_work(&mut self) -> Vec<UiNativeSubmittedFrameWork> {
        self.log.take()
    }

    /// Requests the host close, runs it until it exits or `max_turns` pass,
    /// and finishes it as the platform loop does when its run returns. A
    /// host that settles quiet or busy instead of exiting is stopped, and
    /// the report names [`UiNativeEventLoopRunDenial::CloseUnhonored`].
    pub fn close(
        mut self,
        max_turns: u32,
    ) -> Result<UiNativeEventLoopRunReport, UiNativeEventLoopStopReport> {
        if !self.control.exiting() {
            let close = UiNativeOffscreenEvent::Window(WindowEvent::CloseRequested);
            self.turn(Some(close), 0);
            let settle = self.run_until_idle(max_turns);
            if !matches!(settle, UiNativeOffscreenSettle::Exited { .. }) {
                if let Some(application) = self.application.as_mut() {
                    application
                        .failure
                        .get_or_insert(UiNativeEventLoopRunDenial::CloseUnhonored);
                }
            }
        }
        self.finish()
            .expect("a session finishes only once, when closed or dropped")
    }

    /// One ordinary turn: new events, `event`, a pending `Resized`, `wakes`
    /// wakes, a requested redraw, then about to wait.
    fn turn(&mut self, event: Option<UiNativeOffscreenEvent>, wakes: u64) {
        let control = &self.control;
        let application = self
            .application
            .as_mut()
            .expect("a session turns only until it finishes");
        application.on_new_events(control);
        match event {
            Some(UiNativeOffscreenEvent::Resumed(source)) => {
                application.on_resumed(control, source)
            }
            Some(UiNativeOffscreenEvent::Window(event)) => {
                application.on_window_event(control, event);
            }
            None => {}
        }
        if let Some([width, height]) = self.window.take_resize() {
            let resized = WindowEvent::Resized(PhysicalSize::new(width, height));
            application.on_window_event(control, resized);
        }
        for _ in 0..wakes {
            application.on_wake(control);
        }
        if self.window.take_redraw() {
            application.on_window_event(control, WindowEvent::RedrawRequested);
        }
        application.on_about_to_wait(control);
    }

    /// Stops the host, refuses later wakes, and finishes it as the platform
    /// loop does when its run returns; `None` once finished.
    fn finish(
        &mut self,
    ) -> Option<Result<UiNativeEventLoopRunReport, UiNativeEventLoopStopReport>> {
        let application = self.application.take()?;
        self.control.exit();
        self.wakes.close();
        Some(application.finish())
    }
}

impl<Client: UiNativeEventLoopClient> Drop for WorthUiNativeOffscreenSession<Client> {
    fn drop(&mut self) {
        // An unclosed session still closes its client and cleans up; only
        // the report is lost.
        drop(self.finish());
    }
}

#[cfg(test)]
#[path = "offscreen_tests.rs"]
mod tests;
