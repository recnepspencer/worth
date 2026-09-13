use std::marker::PhantomData;

#[cfg(feature = "certification-test-authority")]
use worth_store_physical_backend::WalDurabilityBarrier;
use worth_store_physical_backend::{
    BackendDurabilityProfile, BackendDurabilityProfileId, WalDurabilityBarrierSet,
};

use worth_store_wal::{WalLsnRange, WalSegmentGeneration, WalSegmentId};

use super::{WalDurabilityObservationDenial, WalDurabilityObservationDenialKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalFrameDigest {
    value: String,
}

impl WalFrameDigest {
    pub fn new(value: impl Into<String>) -> Result<Self, WalDurabilityObservationDenial> {
        let value = value.into();
        if value.is_empty() {
            return Err(WalDurabilityObservationDenial::new(
                WalDurabilityObservationDenialKind::EmptyFrameDigest,
            ));
        }
        Ok(Self { value })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalAppendObservationScope {
    segment_id: WalSegmentId,
    generation: WalSegmentGeneration,
    lsn_range: WalLsnRange,
    frame_digest: WalFrameDigest,
    expected_bytes: u64,
}

impl WalAppendObservationScope {
    pub fn new(
        segment_id: WalSegmentId,
        generation: WalSegmentGeneration,
        lsn_range: WalLsnRange,
        frame_digest: impl Into<String>,
        expected_bytes: u64,
    ) -> Result<Self, WalDurabilityObservationDenial> {
        if expected_bytes == 0 {
            return Err(WalDurabilityObservationDenial::new(
                WalDurabilityObservationDenialKind::EmptyFrameWrite,
            ));
        }
        Ok(Self {
            segment_id,
            generation,
            lsn_range,
            frame_digest: WalFrameDigest::new(frame_digest)?,
            expected_bytes,
        })
    }
}

#[cfg(feature = "certification-test-authority")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalAppendFailureObservation {
    BarrierFailed(WalDurabilityBarrier),
    DelayedFlush(WalDurabilityBarrier),
    LostFlush,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalAppendReceipt<P: BackendDurabilityProfile> {
    profile: PhantomData<P>,
    scope: WalAppendObservationScope,
    observed_bytes: u64,
    completed_barriers: WalDurabilityBarrierSet,
    #[cfg(feature = "certification-test-authority")]
    failure: Option<WalAppendFailureObservation>,
}

impl<P: BackendDurabilityProfile> WalAppendReceipt<P> {
    #[cfg(feature = "certification-test-authority")]
    pub fn from_certification_observation(
        scope: WalAppendObservationScope,
        observed_bytes: u64,
        completed_barriers: WalDurabilityBarrierSet,
        failure: Option<WalAppendFailureObservation>,
    ) -> Self {
        Self {
            profile: PhantomData,
            scope,
            observed_bytes,
            completed_barriers,
            failure,
        }
    }

    pub const fn profile_id(&self) -> BackendDurabilityProfileId {
        P::ID
    }

    pub const fn segment_id(&self) -> WalSegmentId {
        self.scope.segment_id
    }

    pub const fn generation(&self) -> WalSegmentGeneration {
        self.scope.generation
    }

    pub const fn lsn_range(&self) -> WalLsnRange {
        self.scope.lsn_range
    }

    pub const fn expected_bytes(&self) -> u64 {
        self.scope.expected_bytes
    }

    pub const fn observed_bytes(&self) -> u64 {
        self.observed_bytes
    }

    pub fn frame_digest(&self) -> &WalFrameDigest {
        &self.scope.frame_digest
    }

    pub const fn required_barriers(&self) -> WalDurabilityBarrierSet {
        P::REQUIRED_BARRIERS
    }

    pub const fn completed_barriers(&self) -> WalDurabilityBarrierSet {
        self.completed_barriers
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) const fn failure(&self) -> Option<WalAppendFailureObservation> {
        self.failure
    }
}
