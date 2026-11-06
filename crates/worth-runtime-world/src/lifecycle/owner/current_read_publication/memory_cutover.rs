//! Borrow one original component cutover before acquiring its World guard.
use crate::branch::ProductBranchObservation;
use std::time::Instant;
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
        Option<Instant>,
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
            Option<Instant>,
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

    /// Add a narrowing cutoff to the original Native request and candidate.
    /// Native checks it after reservation waits before assigning a position.
    pub fn perform_before(self, deadline: Instant) {
        (self.perform)(self.native, Some(deadline));
    }
}

pub(crate) type RelationalCutoverHandoff<'handoff> =
    dyn for<'action, 'native, 'policy, 'request> FnMut(
            PreparedWorldRelationalCutover<'action, 'native, 'policy, 'request>,
        ) + 'handoff;
