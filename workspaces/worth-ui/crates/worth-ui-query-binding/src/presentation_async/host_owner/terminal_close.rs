use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiPresentationAsyncCloseReceipt {
    closed_query_resources: u64,
    transitions: Box<[WorthUiPresentationTransitionObservation]>,
    transition_trace_complete: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiPresentationAsyncCloseDenial {
    ActiveAdmissions,
    QueryClose,
}

impl WorthUiPresentationAsyncCloseReceipt {
    pub const fn closed_query_resources(&self) -> u64 {
        self.closed_query_resources
    }

    pub fn transitions(&self) -> &[WorthUiPresentationTransitionObservation] {
        &self.transitions
    }

    pub const fn transition_trace_complete(&self) -> bool {
        self.transition_trace_complete
    }
}

impl WorthUiPresentationAsyncOwner {
    pub fn close_terminal_resources(
        &mut self,
    ) -> Result<WorthUiPresentationAsyncCloseReceipt, WorthUiPresentationAsyncCloseDenial> {
        if !self.pending.is_empty()
            || !self.settling.is_empty()
            || !self.superseded_pending.is_empty()
            || !self.superseded_awaiting_completion.is_empty()
            || !self.runtime_cleanups.is_empty()
        {
            return Err(WorthUiPresentationAsyncCloseDenial::ActiveAdmissions);
        }
        self.stage_terminal_resources();
        while let Some(key) = self.terminal_closing.keys().next().copied() {
            let closing = self
                .terminal_closing
                .remove(&key)
                .expect("selected terminal resource remains retained");
            if closing
                .admission
                .close_query_live_view(&mut self.workspace)
                .is_err()
            {
                self.terminal_closing.insert(key, closing);
                return Err(WorthUiPresentationAsyncCloseDenial::QueryClose);
            }
            self.active_keys.remove(&key);
            self.terminal_closed_resources = self.terminal_closed_resources.saturating_add(1);
            self.record_transition(WorthUiPresentationTransitionKind::TerminalClosed, key);
        }
        self.retained.clear();
        Ok(WorthUiPresentationAsyncCloseReceipt {
            closed_query_resources: self.terminal_closed_resources,
            transitions: self.transition_trace.clone().into_boxed_slice(),
            transition_trace_complete: !self.transition_trace_overflowed,
        })
    }

    fn stage_terminal_resources(&mut self) {
        for (_, (key, _, admission)) in self.current.drain() {
            self.terminal_closing
                .insert(key, PendingTerminalClose { admission });
        }
        for (key, unresolved) in self.unresolved.drain() {
            self.terminal_closing.insert(
                key,
                PendingTerminalClose {
                    admission: unresolved.admission,
                },
            );
        }
    }
}
