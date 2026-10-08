//! Execution attribution stays with a Ready suspended for its prerequisite.
use super::SelectedReadyReadmission;

/// A caller successor belongs to the caller even while an upstream is resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FrameRole {
    Reached,
    Successor,
    CallerSuccessor,
}

pub(super) struct RequiredWaveFrame<Ready = SelectedReadyReadmission> {
    pub(super) ready: Ready,
    pub(super) role: FrameRole,
    pub(super) contacts: usize,
}
impl<Ready> RequiredWaveFrame<Ready> {
    pub(super) fn into_parts(self) -> (Ready, FrameRole, usize) {
        (self.ready, self.role, self.contacts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_suspended_caller_successor_retains_its_execution_attribution() {
        // The Ready payload is opaque to this custody transfer: its execution
        // role and accrued contacts must survive resolving a prerequisite.
        let saved = RequiredWaveFrame {
            ready: (),
            role: FrameRole::CallerSuccessor,
            contacts: 2,
        };
        let (_, role, contacts) = saved.into_parts();
        assert_eq!(role, FrameRole::CallerSuccessor);
        assert_eq!(contacts, 2);
    }
}
