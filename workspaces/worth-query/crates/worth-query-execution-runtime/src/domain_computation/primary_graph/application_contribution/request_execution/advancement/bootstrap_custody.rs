use super::ExecutionRequest;

/// Installation custody cannot fund an already installed runtime's public doors.
/// Only an installation callback can borrow this phase; it cannot escape the call.
/// World has not issued an installation identity when this callback opens.
/// A host-built raw request cannot readmit an installed provider plan.
pub struct WorthQueryBootstrapAdvancementPhase<'scope> {
    request: ExecutionRequest<'scope, 'scope>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl<'scope> WorthQueryBootstrapAdvancementPhase<'scope> {
    pub(super) const fn new(request: ExecutionRequest<'scope, 'scope>) -> Self {
        Self {
            request,
            _thread: std::marker::PhantomData,
        }
    }

    /// Borrows the same installation request at an execution owner's boundary.
    pub fn execution_request(&self) -> ExecutionRequest<'_, '_> {
        self.request
    }
}
