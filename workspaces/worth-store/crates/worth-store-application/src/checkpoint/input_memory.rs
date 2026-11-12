//! Original allocation tickets follow the moved publication input.
use worth_execution::ExecutionMemoryReservation;

pub(super) struct CheckpointInputMemory {
    _payload: Option<ExecutionMemoryReservation>,
    _members: Option<ExecutionMemoryReservation>,
}

impl CheckpointInputMemory {
    #[cfg(feature = "certification-test-authority")]
    pub(super) fn bytes(&self) -> Option<u64> {
        self._payload
            .as_ref()?
            .bytes()
            .checked_add(self._members.as_ref()?.bytes())
    }
    pub(super) fn new(
        payload: Option<ExecutionMemoryReservation>,
        members: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            _payload: payload,
            _members: members,
        }
    }
}
