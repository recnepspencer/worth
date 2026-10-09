use worth_query_declaration::facade::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryPartialEffectPosture,
    WorthQueryResourceDimension,
};

use super::WorthQueryExecutionResourceEnvelope;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledBoundedStepContract {
    safe_point_family: WorthQueryCancellationSafePointFamily,
    max_work_units_per_step: u64,
    queue_depth_ceiling: u64,
    chunk_width_ceiling: u64,
    scratch_bytes_ceiling: u64,
    retained_bytes_ceiling: u64,
    deadline_nanos: u64,
    partial_effect_posture: WorthQueryPartialEffectPosture,
}

impl WorthQueryInstalledBoundedStepContract {
    pub(super) fn derive(
        envelope: &WorthQueryExecutionResourceEnvelope,
    ) -> Result<Self, &'static str> {
        if envelope.boundary() != worth_query_declaration::facade::domain_computation::WorthQueryExecutionBoundary::BoundedStep {
            return Err("atomic-execution-is-not-bounded-step");
        }
        let max_work_units_per_step = envelope
            .optional_resource_ceiling(WorthQueryResourceDimension::CancellationPollingInterval)
            .ok_or("incomplete-bounded-step-envelope")?;
        if max_work_units_per_step == 0 {
            return Err("zero-cancellation-polling-interval");
        }
        let queue_depth_ceiling = envelope
            .optional_resource_ceiling(WorthQueryResourceDimension::QueueDepth)
            .ok_or("incomplete-bounded-step-envelope")?;
        if queue_depth_ceiling == 0 {
            return Err("zero-bounded-step-queue-depth");
        }
        let chunk_width_ceiling = envelope
            .optional_resource_ceiling(WorthQueryResourceDimension::ChunkWidth)
            .ok_or("incomplete-bounded-step-envelope")?;
        if chunk_width_ceiling == 0 {
            return Err("zero-bounded-step-chunk-width");
        }
        if chunk_width_ceiling > queue_depth_ceiling {
            return Err("bounded-step-chunk-exceeds-queue-depth");
        }
        Ok(Self {
            safe_point_family: envelope.cancellation_safe_point().clone(),
            max_work_units_per_step,
            queue_depth_ceiling,
            chunk_width_ceiling,
            scratch_bytes_ceiling: envelope
                .optional_resource_ceiling(WorthQueryResourceDimension::ScratchBytes)
                .ok_or("incomplete-bounded-step-envelope")?,
            retained_bytes_ceiling: envelope
                .optional_resource_ceiling(WorthQueryResourceDimension::RetainedBytes)
                .ok_or("incomplete-bounded-step-envelope")?,
            deadline_nanos: envelope
                .optional_resource_ceiling(WorthQueryResourceDimension::DeadlineNanos)
                .ok_or("incomplete-bounded-step-envelope")?,
            partial_effect_posture: envelope.partial_effect_posture(),
        })
    }

    pub fn safe_point_family(&self) -> &WorthQueryCancellationSafePointFamily {
        &self.safe_point_family
    }

    pub const fn max_work_units_per_step(&self) -> u64 {
        self.max_work_units_per_step
    }

    pub const fn queue_depth_ceiling(&self) -> u64 {
        self.queue_depth_ceiling
    }

    pub const fn chunk_width_ceiling(&self) -> u64 {
        self.chunk_width_ceiling
    }

    pub const fn scratch_bytes_ceiling(&self) -> u64 {
        self.scratch_bytes_ceiling
    }

    pub const fn retained_bytes_ceiling(&self) -> u64 {
        self.retained_bytes_ceiling
    }

    pub const fn deadline_nanos(&self) -> u64 {
        self.deadline_nanos
    }

    pub const fn partial_effect_posture(&self) -> WorthQueryPartialEffectPosture {
        self.partial_effect_posture
    }

    pub const fn partial_effects_may_remain(&self) -> bool {
        matches!(
            self.partial_effect_posture,
            WorthQueryPartialEffectPosture::PartialEffectsMayRemain
        )
    }
}

#[cfg(test)]
mod tests {
    use worth_query_declaration::facade::domain_computation::{
        WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode,
        WorthQueryResourceDimension, WorthQueryResourceLimitRequest,
        WorthQuerySemanticScaleRequest,
    };

    use super::WorthQueryExecutionResourceEnvelope;

    #[test]
    fn bounded_step_contract_is_derived_from_the_exact_installed_envelope() {
        let envelope = WorthQueryExecutionResourceEnvelope::bounded(
            5,
            7,
            WorthQueryExecutionMode::Asynchronous,
            WorthQueryCancellationSafePointFamily::new("chunk-boundary").unwrap(),
        );

        let contract = envelope.bounded_step_contract().unwrap();
        assert_eq!(contract.safe_point_family().as_str(), "chunk-boundary");
        assert_eq!(contract.max_work_units_per_step(), 7);
        assert_eq!(contract.queue_depth_ceiling(), 7);
        assert_eq!(contract.chunk_width_ceiling(), 7);
        assert_eq!(contract.scratch_bytes_ceiling(), 7);
        assert_eq!(contract.retained_bytes_ceiling(), 7);
    }

    #[test]
    fn zero_queue_or_chunk_width_cannot_claim_bounded_execution() {
        for dimension in [
            WorthQueryResourceDimension::QueueDepth,
            WorthQueryResourceDimension::ChunkWidth,
        ] {
            let envelope = WorthQueryExecutionResourceEnvelope::new(
                WorthQuerySemanticScaleRequest::bounded(1),
                WorthQueryResourceLimitRequest::bounded(1).with(dimension, 0),
                WorthQueryExecutionMode::Asynchronous,
                None,
                WorthQueryCancellationSafePointFamily::new("chunk-boundary").unwrap(),
            );
            assert!(envelope.bounded_step_contract().is_err());
        }
    }

    #[test]
    fn chunk_width_cannot_exceed_the_bound_queue_depth() {
        let envelope = WorthQueryExecutionResourceEnvelope::new(
            WorthQuerySemanticScaleRequest::bounded(8),
            WorthQueryResourceLimitRequest::bounded(8)
                .with(WorthQueryResourceDimension::QueueDepth, 1)
                .with(WorthQueryResourceDimension::ChunkWidth, 8),
            WorthQueryExecutionMode::Asynchronous,
            None,
            WorthQueryCancellationSafePointFamily::new("chunk-boundary").unwrap(),
        );

        assert_eq!(
            envelope.bounded_step_contract(),
            Err("bounded-step-chunk-exceeds-queue-depth")
        );
    }
}
