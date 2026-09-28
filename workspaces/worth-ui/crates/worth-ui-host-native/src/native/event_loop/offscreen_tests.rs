//! The offscreen pump, driven without a platform window or event loop.
//!
//! Starting a session resumes the host at once, negotiating a real graphics
//! device. A denied resume also stops the host, which could pass an exit or
//! idle assertion for the wrong reason, so each test that depends on the
//! resume first asserts that the client saw its surface.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::UiNativeOffscreenSettle;
use crate::native::event_loop::window_port::UiNativeOffscreenWindow;
use crate::native::event_loop::{
    UiNativeClientPresentationAttribution, UiNativeEventLoopClient, UiNativeEventLoopClientClose,
    UiNativeEventLoopClientDenial, UiNativeEventLoopDirective, UiNativeEventLoopRunDenial,
    UiNativeObservationClock, UiNativeObservationReadinessGrant, UiNativeObservationTimeProgress,
    UiNativeReadinessGrant,
};
use crate::{UiNativeWindowConfiguration, WorthUiPreparedNativeHost};

/// A client callback this file records, with the one detail that
/// distinguishes its turn: the extent a surface or redraw grant carried.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RecordedCallback {
    NativeSurfaceReady([u32; 2]),
    RedrawReady([u32; 2]),
    ExternalCloseRequested,
}

/// A client whose every fallible callback is a no-op that records itself,
/// configurable for the few turns these tests bend: closing as soon as the
/// surface is ready, refusing an external close, and re-arming a redraw's
/// own extent so a turn's work never runs out on its own.
struct RecordingClient {
    log: Rc<RefCell<Vec<RecordedCallback>>>,
    closed: Rc<Cell<bool>>,
    honor_close: bool,
    close_on_native_surface_ready: bool,
    redraw_rearm: Rc<RefCell<Option<Rc<UiNativeOffscreenWindow>>>>,
    rearm_toggle: Cell<bool>,
}

impl RecordingClient {
    fn new() -> Self {
        Self {
            log: Rc::new(RefCell::new(Vec::new())),
            closed: Rc::new(Cell::new(false)),
            honor_close: true,
            close_on_native_surface_ready: false,
            redraw_rearm: Rc::new(RefCell::new(None)),
            rearm_toggle: Cell::new(true),
        }
    }
}

impl UiNativeEventLoopClient for RecordingClient {
    fn install_observation_clock(
        &mut self,
        _clock: UiNativeObservationClock,
    ) -> Result<(), UiNativeEventLoopClientDenial> {
        Ok(())
    }

    fn observation_time_ready(
        &mut self,
    ) -> Result<UiNativeObservationTimeProgress, UiNativeEventLoopClientDenial> {
        Ok(UiNativeObservationTimeProgress::new(
            None,
            UiNativeEventLoopDirective::Continue,
        ))
    }

    fn native_surface_ready(
        &mut self,
        grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientDenial> {
        self.log
            .borrow_mut()
            .push(RecordedCallback::NativeSurfaceReady(
                grant.client_physical_size(),
            ));
        Ok(if self.close_on_native_surface_ready {
            UiNativeEventLoopDirective::Close
        } else {
            UiNativeEventLoopDirective::Continue
        })
    }

    fn redraw_ready(
        &mut self,
        grant: UiNativeReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientDenial> {
        self.log
            .borrow_mut()
            .push(RecordedCallback::RedrawReady(grant.client_physical_size()));
        // A border drag never runs out of extents on its own; re-requesting
        // one here is the same resize a real drag keeps reporting, so the
        // host's own commit-and-redraw path stays exact instead of a probe
        // bypassing it.
        if let Some(window) = self.redraw_rearm.borrow().as_ref() {
            let toggled = self.rearm_toggle.get();
            self.rearm_toggle.set(!toggled);
            window.resize(if toggled { [201, 150] } else { [200, 150] });
        }
        Ok(UiNativeEventLoopDirective::Continue)
    }

    fn native_observations_ready(
        &mut self,
        _grant: UiNativeObservationReadinessGrant,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientDenial> {
        Ok(UiNativeEventLoopDirective::Continue)
    }

    fn external_close_requested(
        &mut self,
    ) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopClientDenial> {
        self.log
            .borrow_mut()
            .push(RecordedCallback::ExternalCloseRequested);
        Ok(if self.honor_close {
            UiNativeEventLoopDirective::Close
        } else {
            UiNativeEventLoopDirective::Continue
        })
    }

    fn presentation_attribution(
        &self,
        _observed: &crate::native::UiNativeRetainedFrameObservation,
    ) -> Option<UiNativeClientPresentationAttribution> {
        None
    }

    fn close(self) -> UiNativeEventLoopClientClose {
        self.closed.set(true);
        UiNativeEventLoopClientClose::Complete
    }
}

/// Asserts the resume reached the client with a surface of `extent`.
fn assert_resumed(log: &RefCell<Vec<RecordedCallback>>, extent: [u32; 2]) {
    assert_eq!(
        log.borrow().first(),
        Some(&RecordedCallback::NativeSurfaceReady(extent)),
        "the resume negotiated a device and reached the client"
    );
}

fn qualified_offscreen_host(
    title: &'static str,
) -> crate::native::event_loop::WorthUiNativeEventLoop {
    let (_mechanics, event_loop) = WorthUiPreparedNativeHost::prepare_qualified()
        .into_parts(UiNativeWindowConfiguration::qualified(title, [200, 150]));
    event_loop
}

#[test]
fn resize_reaches_the_client_through_the_next_turns_redraw() {
    let client = RecordingClient::new();
    let log = Rc::clone(&client.log);
    let mut session = qualified_offscreen_host("offscreen-resize")
        .start_offscreen(client, [200, 150], 1.0)
        .expect("an offscreen session resumes without a platform window");
    assert_resumed(&log, [200, 150]);
    // Only the resize's turn is under test below.
    log.borrow_mut().clear();

    session.resize([400, 300]);
    let settle = session.run_until_idle(8);

    assert!(matches!(settle, UiNativeOffscreenSettle::Idle { .. }));
    assert_eq!(
        *log.borrow(),
        vec![RecordedCallback::RedrawReady([400, 300])]
    );
}

#[test]
fn run_until_idle_settles_idle_once_the_bootstrap_turn_quiets() {
    let client = RecordingClient::new();
    let log = Rc::clone(&client.log);
    let mut session = qualified_offscreen_host("offscreen-idle")
        .start_offscreen(client, [200, 150], 1.0)
        .expect("an offscreen session resumes without a platform window");
    assert_resumed(&log, [200, 150]);

    assert!(matches!(
        session.run_until_idle(8),
        UiNativeOffscreenSettle::Idle { .. }
    ));
}

/// The client's own `Close` directive from `native_surface_ready` exits the
/// loop inside the bootstrap turn itself, before `start_offscreen` returns.
#[test]
fn run_until_idle_reports_exited_once_the_client_closes() {
    let client = RecordingClient {
        close_on_native_surface_ready: true,
        ..RecordingClient::new()
    };
    let log = Rc::clone(&client.log);
    let mut session = qualified_offscreen_host("offscreen-exit")
        .start_offscreen(client, [200, 150], 1.0)
        .expect("start_offscreen always returns a session, even one that already exited");
    assert_resumed(&log, [200, 150]);

    assert_eq!(
        session.run_until_idle(8),
        UiNativeOffscreenSettle::Exited { turns: 0 }
    );
}

/// Each turn's redraw re-arms the next one, so the pump stops on the
/// caller's turn limit rather than waiting for the client.
#[test]
fn run_until_idle_reports_busy_when_the_client_keeps_requesting_redraws() {
    let client = RecordingClient::new();
    let log = Rc::clone(&client.log);
    let rearm = Rc::clone(&client.redraw_rearm);
    let mut session = qualified_offscreen_host("offscreen-busy")
        .start_offscreen(client, [200, 150], 1.0)
        .expect("an offscreen session resumes without a platform window");
    assert_resumed(&log, [200, 150]);
    *rearm.borrow_mut() = Some(Rc::clone(&session.window));
    // Each re-armed extent differs from the one before it.
    session.resize([200, 151]);

    assert_eq!(session.run_until_idle(3), UiNativeOffscreenSettle::Busy);
}

#[test]
fn close_names_close_unhonored_when_the_client_keeps_running() {
    let client = RecordingClient {
        honor_close: false,
        ..RecordingClient::new()
    };
    let log = Rc::clone(&client.log);
    let session = qualified_offscreen_host("offscreen-close")
        .start_offscreen(client, [200, 150], 1.0)
        .expect("an offscreen session resumes without a platform window");
    assert_resumed(&log, [200, 150]);

    let stopped = session
        .close(4)
        .expect_err("a client that keeps running past close is stopped, not exited");
    assert_eq!(stopped.cause, UiNativeEventLoopRunDenial::CloseUnhonored);
    assert!(log
        .borrow()
        .contains(&RecordedCallback::ExternalCloseRequested));
}

/// The frame work log is claimed before any graphics negotiation and
/// released only when the session finishes, by close or by drop.
#[test]
fn a_second_offscreen_session_on_the_same_thread_is_refused_until_the_first_finishes() {
    let first = qualified_offscreen_host("offscreen-nested-a")
        .start_offscreen(RecordingClient::new(), [100, 100], 1.0)
        .expect("the first session on this thread starts");

    let refused = qualified_offscreen_host("offscreen-nested-b").start_offscreen(
        RecordingClient::new(),
        [100, 100],
        1.0,
    );
    match refused {
        Err(report) => assert_eq!(report.cause, UiNativeEventLoopRunDenial::EventLoopCreation),
        Ok(_) => panic!("a live session's frame work log must refuse a second one on this thread"),
    }

    drop(first);

    assert!(qualified_offscreen_host("offscreen-nested-c")
        .start_offscreen(RecordingClient::new(), [100, 100], 1.0)
        .is_ok());
}

/// Finishing always closes the client, whether reached through `close` or
/// through `Drop`.
#[test]
fn dropping_a_session_without_close_still_closes_the_client() {
    let client = RecordingClient::new();
    let closed = Rc::clone(&client.closed);
    let session = qualified_offscreen_host("offscreen-drop")
        .start_offscreen(client, [100, 100], 1.0)
        .expect("the session starts");

    assert!(!closed.get());
    drop(session);
    assert!(closed.get());
}
