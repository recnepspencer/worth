use winit::event_loop::{ActiveEventLoop, ControlFlow};

use super::{UiNativeEventLoopApplication, UiNativeEventLoopClient, UiNativeEventLoopRunDenial};

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    pub(super) fn restore_wait_before_observation_deadline(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) {
        // new_events runs before any client/retry callback in the next batch.
        // Restore the carried predecessor, never infer ownership from equal Instants.
        event_loop.set_control_flow(
            self.observation_wait
                .begin_event_batch(event_loop.control_flow()),
        );
    }

    pub(super) fn close_observation_time_and_schedule(&mut self, event_loop: &ActiveEventLoop) {
        let before_close = self.physical_clock.current_tick();
        let Some(client) = self.client.as_mut() else {
            return;
        };
        let progress = match client.observation_time_ready() {
            Ok(progress) => progress,
            Err(_) => return self.fail(event_loop, UiNativeEventLoopRunDenial::ApplicationDriver),
        };
        match progress.directive() {
            super::UiNativeEventLoopDirective::Continue => {}
            super::UiNativeEventLoopDirective::WaitUntil(deadline) => event_loop.set_control_flow(
                super::physical_clock::tighten_deadline(event_loop.control_flow(), deadline),
            ),
            super::UiNativeEventLoopDirective::Close => {
                self.apply_client_directive(event_loop, progress.directive());
                return;
            }
        }
        if self.finalize_presentation_retry_round(event_loop) {
            return;
        }
        self.request_physical_signal_redraw();
        self.schedule_physical_signal_deadline(event_loop);
        let deadline = match progress.deadline() {
            None => return,
            Some(tick) if tick > before_close => self.physical_clock.deadline(tick),
            _ => None,
        };
        let Some(deadline) = deadline else {
            return self.fail(event_loop, UiNativeEventLoopRunDenial::ApplicationDriver);
        };
        // Apply last: client callbacks and presentation retry may have changed
        // ControlFlow. A confirmation deadline must not erase their earlier wake.
        event_loop.set_control_flow(
            self.observation_wait
                .schedule(event_loop.control_flow(), deadline),
        );
    }
}

#[derive(Default)]
pub(super) struct UiNativeObservationWait {
    predecessor: Option<ControlFlow>,
}

impl UiNativeObservationWait {
    fn schedule(&mut self, current: ControlFlow, deadline: std::time::Instant) -> ControlFlow {
        self.predecessor = Some(current);
        super::physical_clock::tighten_deadline(current, deadline)
    }

    fn begin_event_batch(&mut self, current: ControlFlow) -> ControlFlow {
        self.predecessor.take().unwrap_or(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn observation_deadline_preserves_independent_waits_including_equal_expiry() {
        let now = Instant::now();
        let expiry = now + Duration::from_millis(8);
        for previous in [
            ControlFlow::Wait,
            ControlFlow::Poll,
            ControlFlow::WaitUntil(now + Duration::from_millis(3)),
            ControlFlow::WaitUntil(expiry),
            ControlFlow::WaitUntil(now + Duration::from_millis(13)),
        ] {
            let mut wait = UiNativeObservationWait::default();
            let during = wait.schedule(previous, expiry);
            match previous {
                ControlFlow::Poll => assert_eq!(during, ControlFlow::Poll),
                ControlFlow::WaitUntil(earlier) if earlier < expiry => assert_eq!(during, previous),
                _ => assert_eq!(during, ControlFlow::WaitUntil(expiry)),
            }
            // Next event batch restores the independent owner's wait before it
            // processes expiry/cancellation. No deadline equality implies ownership.
            assert_eq!(wait.begin_event_batch(during), previous);
            assert_eq!(wait.begin_event_batch(ControlFlow::Wait), ControlFlow::Wait);
        }
    }
}
