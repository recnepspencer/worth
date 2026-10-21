use std::sync::Arc;

use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BridgeManagedExecutionStepContractIdentity(Arc<str>);

impl BridgeManagedExecutionStepContractIdentity {
    pub(super) fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeManagedExecutionPartialEffectPosture {
    None,
    MayRemain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeManagedExecutionStepLimits {
    max_work_units_per_step: u64,
    queue_depth_ceiling: u64,
    chunk_width_ceiling: u64,
    scratch_bytes_ceiling: u64,
    retained_bytes_ceiling: u64,
    deadline_nanos: Option<u64>,
}

impl BridgeManagedExecutionStepLimits {
    pub const fn new(
        max_work_units_per_step: u64,
        queue_depth_ceiling: u64,
        chunk_width_ceiling: u64,
    ) -> Self {
        Self {
            max_work_units_per_step,
            queue_depth_ceiling,
            chunk_width_ceiling,
            scratch_bytes_ceiling: 0,
            retained_bytes_ceiling: 0,
            deadline_nanos: None,
        }
    }

    pub const fn with_memory_ceilings(
        mut self,
        scratch_bytes_ceiling: u64,
        retained_bytes_ceiling: u64,
    ) -> Self {
        self.scratch_bytes_ceiling = scratch_bytes_ceiling;
        self.retained_bytes_ceiling = retained_bytes_ceiling;
        self
    }

    pub const fn with_deadline_nanos(mut self, deadline_nanos: u64) -> Self {
        self.deadline_nanos = Some(deadline_nanos);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeManagedExecutionStepContract {
    identity: BridgeManagedExecutionStepContractIdentity,
    safe_point_family: Arc<str>,
    queue_depth_ceiling: u64,
    step_limits: Option<BridgeManagedExecutionStepLimits>,
    deadline_nanos: Option<u64>,
    partial_effects_may_remain: bool,
}

impl BridgeManagedExecutionStepContract {
    pub fn new(
        safe_point_family: impl Into<Arc<str>>,
        limits: BridgeManagedExecutionStepLimits,
        partial_effect_posture: BridgeManagedExecutionPartialEffectPosture,
    ) -> Result<Self, &'static str> {
        let safe_point_family = safe_point_family.into();
        if safe_point_family.trim().is_empty()
            || safe_point_family.trim() != safe_point_family.as_ref()
            || safe_point_family.chars().any(char::is_control)
        {
            return Err("invalid-bridge-managed-safe-point-family");
        }
        if limits.max_work_units_per_step == 0 {
            return Err("zero-bridge-managed-step-work-limit");
        }
        if limits.queue_depth_ceiling == 0 {
            return Err("zero-bridge-managed-queue-depth");
        }
        if limits.chunk_width_ceiling == 0 {
            return Err("zero-bridge-managed-chunk-width");
        }
        if limits.deadline_nanos == Some(0) {
            return Err("zero-bridge-managed-deadline");
        }
        let partial_effects_may_remain = matches!(
            partial_effect_posture,
            BridgeManagedExecutionPartialEffectPosture::MayRemain
        );
        let canonical = format!(
            "bridge-managed-step-contract-v2|safe-point={safe_point_family}|work={}|queue={}|chunk={}|scratch={}|retained={}|deadline={:?}|partial-effects={partial_effects_may_remain}",
            limits.max_work_units_per_step,
            limits.queue_depth_ceiling,
            limits.chunk_width_ceiling,
            limits.scratch_bytes_ceiling,
            limits.retained_bytes_ceiling,
            limits.deadline_nanos,
        );
        let digest = Sha256::digest(canonical.as_bytes());
        Ok(Self {
            identity: BridgeManagedExecutionStepContractIdentity(Arc::from(format!(
                "bridge-managed-step-contract:sha256:{digest:x}"
            ))),
            safe_point_family,
            queue_depth_ceiling: limits.queue_depth_ceiling,
            step_limits: Some(limits),
            deadline_nanos: limits.deadline_nanos,
            partial_effects_may_remain,
        })
    }

    /// A live queue retains managed request and occupancy custody, without
    /// granting a bounded provider step or accepting work/memory estimates.
    pub fn queue(
        safe_point_family: impl Into<Arc<str>>,
        capacity: u64,
    ) -> Result<Self, &'static str> {
        let safe_point_family = safe_point_family.into();
        if safe_point_family.trim().is_empty()
            || safe_point_family.trim() != safe_point_family.as_ref()
            || safe_point_family.chars().any(char::is_control)
        {
            return Err("invalid-bridge-managed-safe-point-family");
        }
        if capacity == 0 {
            return Err("zero-bridge-managed-queue-depth");
        }
        let canonical = format!(
            "bridge-managed-queue-contract-v1|safe-point={safe_point_family}|queue={capacity}"
        );
        let digest = Sha256::digest(canonical.as_bytes());
        Ok(Self {
            identity: BridgeManagedExecutionStepContractIdentity(Arc::from(format!(
                "bridge-managed-queue-contract:sha256:{digest:x}"
            ))),
            safe_point_family,
            queue_depth_ceiling: capacity,
            step_limits: None,
            deadline_nanos: None,
            partial_effects_may_remain: false,
        })
    }
    pub const fn step_limits(&self) -> Option<&BridgeManagedExecutionStepLimits> {
        self.step_limits.as_ref()
    }

    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    pub(super) fn identity_proof(&self) -> &BridgeManagedExecutionStepContractIdentity {
        &self.identity
    }

    pub fn safe_point_family(&self) -> &str {
        &self.safe_point_family
    }

    pub const fn queue_depth_ceiling(&self) -> u64 {
        self.queue_depth_ceiling
    }

    pub const fn deadline_nanos(&self) -> Option<u64> {
        self.deadline_nanos
    }

    pub const fn partial_effects_may_remain(&self) -> bool {
        self.partial_effects_may_remain
    }
}

impl BridgeManagedExecutionStepLimits {
    pub const fn max_work_units_per_step(&self) -> u64 {
        self.max_work_units_per_step
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
}
