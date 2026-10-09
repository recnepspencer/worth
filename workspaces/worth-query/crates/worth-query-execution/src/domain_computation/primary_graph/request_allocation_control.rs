use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Wake, Waker},
};
use worth_execution::{CancellationSource, ExecutionAllocationPolicy, ExecutionResourceLease};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[cfg(test)]
mod tests;

/// Attempt-local stop custody; it never becomes mutation or commit authority.
pub(super) struct RequestAllocationControl<'authority> {
    storage: ControlStorage<'authority>,
}

enum ControlStorage<'authority> {
    SystemAllocation,
    Execution {
        lease: ExecutionResourceLease<'authority>,
        _registration: CancellationRegistration,
    },
}

impl<'authority> RequestAllocationControl<'authority> {
    pub(super) fn new(
        request: &WorthQueryRequestScope,
        policy: ExecutionAllocationPolicy<'_, 'authority>,
    ) -> Self {
        match policy {
            ExecutionAllocationPolicy::SystemAllocation => Self {
                storage: ControlStorage::SystemAllocation,
            },
            ExecutionAllocationPolicy::Execution(parent) => {
                let registration = CancellationRegistration::new(request);
                let lease = parent
                    .controlled_child(registration.cancellation.token(), Some(request.deadline()));
                Self {
                    storage: ControlStorage::Execution {
                        lease,
                        _registration: registration,
                    },
                }
            }
        }
    }

    pub(super) fn policy(&self) -> ExecutionAllocationPolicy<'_, 'authority> {
        match &self.storage {
            ControlStorage::SystemAllocation => ExecutionAllocationPolicy::SystemAllocation,
            ControlStorage::Execution { lease, .. } => ExecutionAllocationPolicy::Execution(lease),
        }
    }
}

struct CancellationRegistration {
    cancellation: CancellationSource,
    _future: Pin<Box<dyn Future<Output = ()> + Send>>,
}

struct CancelAllocation(CancellationSource);

impl Wake for CancelAllocation {
    fn wake(self: Arc<Self>) {
        self.0.cancel();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.cancel();
    }
}

impl CancellationRegistration {
    fn new(request: &WorthQueryRequestScope) -> Self {
        let cancellation = CancellationSource::new();
        let waker = Waker::from(Arc::new(CancelAllocation(cancellation.clone())));
        let token = request.cancellation().clone();
        let mut future = Box::pin(async move { token.cancelled().await });
        if future
            .as_mut()
            .poll(&mut Context::from_waker(&waker))
            .is_ready()
        {
            cancellation.cancel();
        }
        Self {
            cancellation,
            _future: future,
        }
    }
}
