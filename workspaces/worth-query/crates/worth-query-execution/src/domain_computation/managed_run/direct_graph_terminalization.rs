use super::{
    WorthQueryActiveDirectGraphExecution, WorthQueryDirectGraphStepOutcome,
    WorthQueryDirectRunTerminal, WorthQueryManagedRunTerminalKind,
};

impl WorthQueryActiveDirectGraphExecution {
    pub(super) fn interrupted_terminal(
        mut self,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> WorthQueryDirectGraphStepOutcome {
        self.running.provider_work_mut().interrupt_step_call();
        self.into_terminal_outcome(kind)
    }

    pub(super) fn abandoned_terminal(
        mut self,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> WorthQueryDirectGraphStepOutcome {
        self.running.provider_work_mut().abandon();
        self.into_terminal_outcome(kind)
    }

    pub(super) fn settled_terminal(
        self,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> WorthQueryDirectGraphStepOutcome {
        self.into_terminal_outcome(kind)
    }

    fn into_terminal_outcome(
        mut self,
        kind: WorthQueryManagedRunTerminalKind,
    ) -> WorthQueryDirectGraphStepOutcome {
        let output_retained_bytes = self.execution.output_retained_bytes();
        let _ = self
            .running
            .provider_work_mut()
            .release_output_bytes(output_retained_bytes);
        let release = self.execution.release_provider_execution();
        let (release_evidence, memory) = release.into_parts();
        self.running
            .provider_work_mut()
            .record_provider_execution_release(&release_evidence);
        self.running
            .provider_work_mut()
            .retain_provider_memory(memory);
        terminal_outcome(self.running.terminal(kind), kind)
    }
}

pub(super) fn terminal_outcome(
    terminal: WorthQueryDirectRunTerminal,
    kind: WorthQueryManagedRunTerminalKind,
) -> WorthQueryDirectGraphStepOutcome {
    match kind {
        WorthQueryManagedRunTerminalKind::Cancelled => {
            WorthQueryDirectGraphStepOutcome::Cancelled(terminal)
        }
        WorthQueryManagedRunTerminalKind::TimedOut => {
            WorthQueryDirectGraphStepOutcome::TimedOut(terminal)
        }
        WorthQueryManagedRunTerminalKind::Exhausted => {
            WorthQueryDirectGraphStepOutcome::Exhausted(terminal)
        }
        WorthQueryManagedRunTerminalKind::Degraded => {
            WorthQueryDirectGraphStepOutcome::Degraded(terminal)
        }
        WorthQueryManagedRunTerminalKind::Failed => {
            WorthQueryDirectGraphStepOutcome::Failed(terminal)
        }
        WorthQueryManagedRunTerminalKind::Completed => {
            unreachable!("provider completion returns a completion authority")
        }
    }
}
