use winit::event_loop::ActiveEventLoop;

impl<Client: super::UiNativeEventLoopClient> super::UiNativeEventLoopApplication<Client> {
    pub(super) fn finalize_presentation_retry_round(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> bool {
        let finalization = self
            .shared
            .borrow_mut()
            .lifecycle
            .finalize_presentation_retry_round(std::time::Instant::now());
        if let Some(denial) = terminal_denial(finalization) {
            self.fail(event_loop, denial);
            return true;
        }
        self.schedule_presentation_retry(event_loop);
        false
    }

    pub(super) fn schedule_presentation_retry(&mut self, event_loop: &ActiveEventLoop) {
        let wake = self.shared.borrow().lifecycle.presentation_retry_wake();
        match wake {
            Some(crate::native::UiNativePresentationRetryWake::Timeout(deadline)) => {
                event_loop.set_control_flow(super::physical_clock::tighten_deadline(
                    event_loop.control_flow(),
                    deadline,
                ));
            }
            // Visibility has no timed wake of its own. Preserve independent
            // application, physical-signal, and confirmation scheduling.
            Some(crate::native::UiNativePresentationRetryWake::Visibility) => {}
            None => {}
        }
    }

    pub(super) fn progress_due_presentation_retry(&mut self, event_loop: &ActiveEventLoop) {
        if self
            .shared
            .borrow_mut()
            .lifecycle
            .consume_due_presentation_timeout(std::time::Instant::now())
        {
            self.commit_readiness(event_loop);
        }
    }

    pub(super) fn commit_visible_surface_readiness(&mut self, event_loop: &ActiveEventLoop) {
        if !self
            .shared
            .borrow()
            .lifecycle
            .presentation_readiness_allowed()
        {
            return;
        }
        let retry_wake = self.shared.borrow().lifecycle.presentation_retry_wake();
        match retry_wake {
            Some(crate::native::UiNativePresentationRetryWake::Timeout(_)) => return,
            Some(crate::native::UiNativePresentationRetryWake::Visibility) => {
                if !self
                    .shared
                    .borrow_mut()
                    .lifecycle
                    .consume_presentation_visibility()
                {
                    return self.fail(
                        event_loop,
                        super::UiNativeEventLoopRunDenial::ApplicationDriver,
                    );
                }
            }
            None => {}
        }
        self.commit_readiness(event_loop);
    }

    pub(super) fn awaits_presentation_visibility(&self) -> bool {
        self.shared.borrow().lifecycle.presentation_retry_wake()
            == Some(crate::native::UiNativePresentationRetryWake::Visibility)
    }
}

pub(super) const fn terminal_denial(
    finalization: crate::native::UiNativePresentationRetryFinalization,
) -> Option<super::UiNativeEventLoopRunDenial> {
    match finalization {
        crate::native::UiNativePresentationRetryFinalization::DeadlineExpired => {
            Some(super::UiNativeEventLoopRunDenial::PresentationDeadlineExpired)
        }
        crate::native::UiNativePresentationRetryFinalization::Unchanged
        | crate::native::UiNativePresentationRetryFinalization::Wake(_) => None,
    }
}
