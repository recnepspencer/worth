use std::marker::PhantomData;

use worth_store_physical_backend::{
    BackendDurabilityProfile, BackendDurabilityProfileId, BackendDurabilitySupport,
    WalDurabilityBarrier, WalDurabilityBarrierSet,
};

use worth_store_wal::{WalLsnRange, WalSegmentGeneration, WalSegmentId};

#[cfg(feature = "certification-test-authority")]
use super::WalAppendFailureObservation;
use super::{WalAppendReceipt, WalFrameDigest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalDurabilityObservationDenialKind {
    EmptyFrameDigest,
    EmptyFrameWrite,
    AppendNotCompleted,
    ShortWrite,
    RequiredBarrierMissing,
    BarrierFailed,
    DelayedFlush,
    LostFlush,
    UnsupportedDurabilityCapability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalDurabilityObservationDenial {
    kind: WalDurabilityObservationDenialKind,
    profile_id: Option<BackendDurabilityProfileId>,
    barrier: Option<WalDurabilityBarrier>,
}

impl WalDurabilityObservationDenial {
    pub(super) const fn new(kind: WalDurabilityObservationDenialKind) -> Self {
        Self {
            kind,
            profile_id: None,
            barrier: None,
        }
    }

    const fn for_profile(
        kind: WalDurabilityObservationDenialKind,
        profile_id: BackendDurabilityProfileId,
    ) -> Self {
        Self {
            kind,
            profile_id: Some(profile_id),
            barrier: None,
        }
    }

    const fn for_barrier(
        kind: WalDurabilityObservationDenialKind,
        profile_id: BackendDurabilityProfileId,
        barrier: WalDurabilityBarrier,
    ) -> Self {
        Self {
            kind,
            profile_id: Some(profile_id),
            barrier: Some(barrier),
        }
    }

    pub const fn kind(&self) -> WalDurabilityObservationDenialKind {
        self.kind
    }

    pub const fn profile_id(&self) -> Option<BackendDurabilityProfileId> {
        self.profile_id
    }

    pub const fn barrier(&self) -> Option<WalDurabilityBarrier> {
        self.barrier
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalDurabilityObservationBasis {
    profile_id: BackendDurabilityProfileId,
    segment_id: WalSegmentId,
    generation: WalSegmentGeneration,
    lsn_range: WalLsnRange,
    frame_digest: WalFrameDigest,
    required_barriers: WalDurabilityBarrierSet,
    completed_barriers: WalDurabilityBarrierSet,
}

impl WalDurabilityObservationBasis {
    pub const fn profile_id(&self) -> BackendDurabilityProfileId {
        self.profile_id
    }

    pub const fn segment_id(&self) -> WalSegmentId {
        self.segment_id
    }

    pub const fn generation(&self) -> WalSegmentGeneration {
        self.generation
    }

    pub const fn lsn_range(&self) -> WalLsnRange {
        self.lsn_range
    }

    pub fn frame_digest(&self) -> &WalFrameDigest {
        &self.frame_digest
    }

    pub const fn required_barriers(&self) -> WalDurabilityBarrierSet {
        self.required_barriers
    }

    pub const fn completed_barriers(&self) -> WalDurabilityBarrierSet {
        self.completed_barriers
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalDurabilityObservation<P: BackendDurabilityProfile> {
    profile: PhantomData<P>,
    basis: WalDurabilityObservationBasis,
}

impl<P: BackendDurabilityProfile> WalDurabilityObservation<P> {
    pub fn from_append_receipt(
        receipt: WalAppendReceipt<P>,
    ) -> Result<Self, WalDurabilityObservationDenial> {
        validate_profile::<P>()?;
        validate_append(&receipt)?;
        let completed = receipt.completed_barriers();
        let required = receipt.required_barriers();
        if !completed.satisfies(required) {
            let missing = completed
                .first_missing_from(required)
                .expect("an incomplete barrier set names its missing barrier");
            return Err(WalDurabilityObservationDenial::for_barrier(
                WalDurabilityObservationDenialKind::RequiredBarrierMissing,
                P::ID,
                missing,
            ));
        }
        Ok(Self {
            profile: PhantomData,
            basis: WalDurabilityObservationBasis {
                profile_id: P::ID,
                segment_id: receipt.segment_id(),
                generation: receipt.generation(),
                lsn_range: receipt.lsn_range(),
                frame_digest: receipt.frame_digest().clone(),
                required_barriers: required,
                completed_barriers: completed,
            },
        })
    }

    pub const fn profile_id(&self) -> BackendDurabilityProfileId {
        self.basis.profile_id
    }

    pub const fn basis(&self) -> &WalDurabilityObservationBasis {
        &self.basis
    }
}

fn validate_profile<P: BackendDurabilityProfile>() -> Result<(), WalDurabilityObservationDenial> {
    match P::SUPPORT {
        BackendDurabilitySupport::Certified => Ok(()),
        BackendDurabilitySupport::UnsupportedDurabilityCapability => {
            Err(WalDurabilityObservationDenial::for_profile(
                WalDurabilityObservationDenialKind::UnsupportedDurabilityCapability,
                P::ID,
            ))
        }
        BackendDurabilitySupport::AdversarialLostFlush => {
            Err(WalDurabilityObservationDenial::for_profile(
                WalDurabilityObservationDenialKind::LostFlush,
                P::ID,
            ))
        }
    }
}

fn validate_append<P: BackendDurabilityProfile>(
    receipt: &WalAppendReceipt<P>,
) -> Result<(), WalDurabilityObservationDenial> {
    if receipt.observed_bytes() == 0 {
        return Err(WalDurabilityObservationDenial::for_profile(
            WalDurabilityObservationDenialKind::AppendNotCompleted,
            P::ID,
        ));
    }
    if receipt.observed_bytes() != receipt.expected_bytes() {
        return Err(WalDurabilityObservationDenial::for_profile(
            WalDurabilityObservationDenialKind::ShortWrite,
            P::ID,
        ));
    }
    #[cfg(feature = "certification-test-authority")]
    if let Some(failure) = receipt.failure() {
        let (kind, barrier) = match failure {
            WalAppendFailureObservation::BarrierFailed(barrier) => (
                WalDurabilityObservationDenialKind::BarrierFailed,
                Some(barrier),
            ),
            WalAppendFailureObservation::DelayedFlush(barrier) => (
                WalDurabilityObservationDenialKind::DelayedFlush,
                Some(barrier),
            ),
            WalAppendFailureObservation::LostFlush => {
                (WalDurabilityObservationDenialKind::LostFlush, None)
            }
        };
        return Err(match barrier {
            Some(barrier) => WalDurabilityObservationDenial::for_barrier(kind, P::ID, barrier),
            None => WalDurabilityObservationDenial::for_profile(kind, P::ID),
        });
    }
    Ok(())
}
