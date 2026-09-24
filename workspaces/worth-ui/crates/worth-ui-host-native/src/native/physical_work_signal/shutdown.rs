use worth_signal::facade::adapters::RuntimeTelemetry;

use super::worker::UiNativePhysicalSignalWorker;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativePhysicalSignalShutdown {
    Disposed,
    RetainedObligations { active_requests: usize },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativePhysicalSignalLifecycle {
    Running,
    Draining,
    Disposed,
}

/// The signal runtime is owned until shutdown retires it; a retired runtime
/// keeps only the totals it reached.
pub(super) enum UiNativePhysicalSignalRuntime {
    Owned(UiNativePhysicalSignalWorker),
    Retired(UiNativePhysicalSignalRetiredRuntime),
}

#[derive(Clone, Copy)]
pub(super) struct UiNativePhysicalSignalRetiredRuntime {
    telemetry: RuntimeTelemetry,
    performed_transitions: u64,
    performed_nodes: u64,
}

impl UiNativePhysicalSignalRuntime {
    pub(super) const fn worker(&self) -> Option<&UiNativePhysicalSignalWorker> {
        match self {
            Self::Owned(worker) => Some(worker),
            Self::Retired(_) => None,
        }
    }

    pub(super) fn worker_mut(&mut self) -> Option<&mut UiNativePhysicalSignalWorker> {
        match self {
            Self::Owned(worker) => Some(worker),
            Self::Retired(_) => None,
        }
    }

    pub(super) fn telemetry(&self) -> RuntimeTelemetry {
        match self {
            Self::Owned(worker) => worker.telemetry(),
            Self::Retired(retired) => retired.telemetry,
        }
    }

    pub(super) fn performed_transitions(&self) -> u64 {
        match self {
            Self::Owned(worker) => worker.performed_transitions(),
            Self::Retired(retired) => retired.performed_transitions,
        }
    }

    pub(super) fn performed_nodes(&self) -> u64 {
        match self {
            Self::Owned(worker) => worker.performed_nodes(),
            Self::Retired(retired) => retired.performed_nodes,
        }
    }

    fn retire(&mut self) {
        if let Self::Owned(worker) = self {
            *self = Self::Retired(UiNativePhysicalSignalRetiredRuntime {
                telemetry: worker.telemetry(),
                performed_transitions: worker.performed_transitions(),
                performed_nodes: worker.performed_nodes(),
            });
        }
    }
}

impl super::UiNativePhysicalSignalOwner {
    pub(crate) fn shutdown(&mut self) -> UiNativePhysicalSignalShutdown {
        if self.lifecycle == UiNativePhysicalSignalLifecycle::Running {
            self.lifecycle = UiNativePhysicalSignalLifecycle::Draining;
        }
        let active_requests = self
            .runtime
            .worker()
            .map_or(0, UiNativePhysicalSignalWorker::active_requests);
        if active_requests != 0 {
            return UiNativePhysicalSignalShutdown::RetainedObligations { active_requests };
        }
        self.runtime.retire();
        self.route.clear();
        self.wake.clear();
        self.transition_observations.clear();
        self.lifecycle = UiNativePhysicalSignalLifecycle::Disposed;
        UiNativePhysicalSignalShutdown::Disposed
    }
}
