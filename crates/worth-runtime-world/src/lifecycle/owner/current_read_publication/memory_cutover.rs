//! Borrow one original component cutover before acquiring its World guard.
use crate::branch::ProductBranchObservation;
use crate::lifecycle::RuntimeWorldInstant;
use worth_relational::facade::mvcc::PreparedMemoryRelationalCutover;

/// One-use World receiver for Native's prepared memory publication. The
/// callback may acquire its own liveness guards before consuming this token.
/// Native custody and World progression remain outside that callback.
pub struct PreparedWorldRelationalCutover<'action, 'native, 'policy, 'request> {
    native: PreparedMemoryRelationalCutover<'native, 'policy, 'request>,
    current: &'action ProductBranchObservation,
    target: &'action ProductBranchObservation,
    perform: &'action mut dyn FnMut(
        PreparedMemoryRelationalCutover<'native, 'policy, 'request>,
        Option<RuntimeWorldInstant>,
    ),
}

impl<'action, 'native, 'policy, 'request>
    PreparedWorldRelationalCutover<'action, 'native, 'policy, 'request>
{
    pub(super) fn new(
        native: PreparedMemoryRelationalCutover<'native, 'policy, 'request>,
        current: &'action ProductBranchObservation,
        target: &'action ProductBranchObservation,
        perform: &'action mut dyn FnMut(
            PreparedMemoryRelationalCutover<'native, 'policy, 'request>,
            Option<RuntimeWorldInstant>,
        ),
    ) -> Self {
        Self {
            native,
            current,
            target,
            perform,
        }
    }

    pub fn current_read(&self) -> &ProductBranchObservation {
        self.current
    }

    pub fn expected_target(&self) -> &ProductBranchObservation {
        self.target
    }

    pub fn perform(self) {
        (self.perform)(self.native, None);
    }

    /// Narrow the original request with a deadline from the installed World
    /// clock. World checks it again after Native's reservation waits, before
    /// Native assigns a position.
    pub fn perform_before(self, deadline: RuntimeWorldInstant) {
        (self.perform)(self.native, Some(deadline));
    }
}

pub(crate) type RelationalCutoverHandoff<'handoff> =
    dyn for<'action, 'native, 'policy, 'request> FnMut(
            PreparedWorldRelationalCutover<'action, 'native, 'policy, 'request>,
        ) + 'handoff;
