//! Close drains retained work through the existing event lifecycle, admitting no new input.
use super::*;
impl PlatformPulseApplicationRuntime {
    pub(in crate::native_application) fn request_query_close(&mut self) {
        if self.external_close_requested { return; }
        self.external_close_requested = true;
        // One wake belongs to this close request, not a retry executor. Subsequent
        // physical progress/readiness comes from the actual outstanding publication.
        if !self.query_close_ready() && let Some(port) = &self.query_readiness {
            let _ = port.signal();
        }
    }
    pub(in crate::native_application) fn query_close_ready(&self) -> bool {
        self.pending_frame_presentation.is_none()
            && self.pending_managed_rebind.is_none()
            && self.pending_query_request.is_none()
            && self.pending_query_actions.is_empty()
            && self.pending_query_cancellation.is_none()
            && self.pending_native_publications.is_empty()
            && self.pending_intent_execution_transitions.is_empty()
            && !self.query_lifecycle.as_ref().is_some_and(|query| query.has_pending_mutation() || query.has_pending_publication())
    }
    pub(in crate::native_application) fn advance_closing_query_turn(&mut self) {
        if self.pending_managed_rebind.is_some() || self.pending_frame_presentation.is_some() { return; }
        let mut shell = self.take_runtime_shell();
        self.advance_pending_native_publications(&mut shell);
        self.shell = Some(shell);
        if self.pending_managed_rebind.is_some() || self.pending_frame_presentation.is_some() { return; }
        let retained = self.pending_query_request.is_some()
            || self.pending_query_cancellation.is_some()
            || !self.pending_query_actions.is_empty()
            || !self.pending_intent_execution_transitions.is_empty()
            || self.query_lifecycle.as_ref().is_some_and(|query| query.has_pending_mutation() || query.has_pending_publication());
        if !retained {
            // A rebind may already have admitted its Query fact while the actual
            // shell still owes its frame. Present existing work; never start ingress.
            if self.startup_ready { self.present(); }
            return;
        }
        if self.query_lifecycle.as_ref().is_some_and(|query| query.has_pending_mutation()) {
            if self.pending_query_request.is_none() {
                // An unexecuted source command can close without running its effect.
                // Committed/recovery custody refuses this and continues as itself.
                let cancelled = self.query_lifecycle.as_mut().expect("retained Query owner").cancel_pending_mutation();
                if cancelled.is_err() { self.resume_query_mutation(); }
            } else {
                self.resume_query_mutation();
            }
        }
        // This polls existing Intent executions and delivers their actual completion;
        // it never drains watchers or accepts another product-port command.
        self.advance_intent_execution();
        self.present();
    }
}
