use worth_signal::facade::TemporalDuration;

use super::PhysicalMutationIdempotencyKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalMutationDeadline(TemporalDuration);

impl PhysicalMutationDeadline {
    pub fn after_milliseconds(milliseconds: u64) -> Option<Self> {
        TemporalDuration::temporal_duration(milliseconds)
            .ok()
            .map(Self)
    }

    pub const fn at(deadline: TemporalDuration) -> Self {
        Self(deadline)
    }

    pub const fn signal_deadline(self) -> TemporalDuration {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::PhysicalMutationDeadline;

    #[test]
    fn millisecond_deadline_rejects_zero_and_preserves_positive_duration() {
        assert_eq!(PhysicalMutationDeadline::after_milliseconds(0), None);
        assert_eq!(
            PhysicalMutationDeadline::after_milliseconds(1_000)
                .unwrap()
                .signal_deadline()
                .get(),
            1_000
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalMutationDurabilityRequest {
    PlatformDurable,
}

/// Caller request for one platform-durable physical mutation.
///
/// Deadline is derived Signal lifecycle input and is deliberately absent from
/// canonical request equivalence. The complete idempotency key remains a
/// separate exact-retry identity.
#[derive(Debug)]
pub struct PhysicalMutationRequest {
    idempotency: PhysicalMutationIdempotencyKey,
    deadline: PhysicalMutationDeadline,
    durability: PhysicalMutationDurabilityRequest,
    interruption: Option<std::sync::Arc<worth_execution::ExecutionInterruptionView>>,
}

impl PhysicalMutationRequest {
    pub const fn platform_durable(
        idempotency: PhysicalMutationIdempotencyKey,
        deadline: PhysicalMutationDeadline,
    ) -> Self {
        Self {
            idempotency,
            deadline,
            durability: PhysicalMutationDurabilityRequest::PlatformDurable,
            interruption: None,
        }
    }

    pub fn with_interruption_view(
        mut self,
        view: worth_execution::ExecutionInterruptionView,
    ) -> Self {
        self.interruption = Some(std::sync::Arc::new(view));
        self
    }
    pub(in crate::physical_runtime) fn with_shared_interruption(
        mut self,
        view: Option<std::sync::Arc<worth_execution::ExecutionInterruptionView>>,
    ) -> Self {
        self.interruption = view;
        self
    }

    pub const fn idempotency_key(&self) -> &PhysicalMutationIdempotencyKey {
        &self.idempotency
    }

    pub const fn deadline(&self) -> PhysicalMutationDeadline {
        self.deadline
    }

    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        PhysicalMutationIdempotencyKey,
        PhysicalMutationDeadline,
        PhysicalMutationDurabilityRequest,
        Option<std::sync::Arc<worth_execution::ExecutionInterruptionView>>,
    ) {
        (
            self.idempotency,
            self.deadline,
            self.durability,
            self.interruption,
        )
    }
}
