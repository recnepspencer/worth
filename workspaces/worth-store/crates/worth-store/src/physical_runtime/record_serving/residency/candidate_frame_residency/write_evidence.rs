use sha2::{Digest, Sha256};
use worth_store_physical_backend::{CompletedArtifactNewWrite, CompletedArtifactRangeWrite};
use worth_store_physical_format::{store_namespace::StableStoreIdentity, RecordFrameCoordinate};

use crate::physical_runtime::record_serving::RecordAppendDenial;
use crate::physical_runtime::{
    PhysicalEffectIdentity, PhysicalWorkEffectFate, PhysicalWorkIdentity,
    PhysicalWorkRecoveryDisposition,
};

pub(in crate::physical_runtime::record_serving) struct CandidateFramePhysicalWrite {
    receipt: CandidateWriteReceipt,
    coordinate: RecordFrameCoordinate,
    settlement: crate::physical_runtime::record_serving::CanonicalRecordMutationSettlement,
}

enum CandidateWriteReceipt {
    New(CompletedArtifactNewWrite),
    ArenaRange(CompletedArtifactRangeWrite),
    RetirementRetry(
        CompletedArtifactRangeWrite,
        crate::physical_runtime::record_serving::RetirementCandidateRetryScope,
    ),
}

pub(in crate::physical_runtime::record_serving) struct CandidateFrameResidencySettlement {
    settlement: crate::physical_runtime::record_serving::CanonicalRecordMutationSettlement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum CandidateFrameEffectSource {
    NewArtifact,
    ExistingArtifactWriteback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct CandidateFrameEffectSettlement {
    source: CandidateFrameEffectSource,
    coordinate: RecordFrameCoordinate,
    payload_digest: [u8; 32],
    work: PhysicalWorkIdentity,
    effect: Option<PhysicalEffectIdentity>,
    fate: PhysicalWorkEffectFate,
    recovery: PhysicalWorkRecoveryDisposition,
}

impl CandidateFramePhysicalWrite {
    pub(in crate::physical_runtime::record_serving) fn completed_retirement_retry(
        receipt: CompletedArtifactRangeWrite,
        settlement: crate::physical_runtime::record_serving::CanonicalRecordMutationSettlement,
        scope: crate::physical_runtime::record_serving::RetirementCandidateRetryScope,
    ) -> Self {
        Self {
            coordinate: receipt.coordinate(),
            receipt: CandidateWriteReceipt::RetirementRetry(receipt, scope),
            settlement,
        }
    }
    pub(in crate::physical_runtime::record_serving) fn completed(
        receipt: CompletedArtifactNewWrite,
        coordinate: RecordFrameCoordinate,
        settlement: crate::physical_runtime::record_serving::CanonicalRecordMutationSettlement,
    ) -> Self {
        Self {
            receipt: CandidateWriteReceipt::New(receipt),
            coordinate,
            settlement,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn completed_arena_range(
        receipt: CompletedArtifactRangeWrite,
        settlement: crate::physical_runtime::record_serving::CanonicalRecordMutationSettlement,
    ) -> Self {
        Self {
            coordinate: receipt.coordinate(),
            receipt: CandidateWriteReceipt::ArenaRange(receipt),
            settlement,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn settle_residency(
        self,
        store: StableStoreIdentity,
        coordinate: super::CandidateFrameCoordinate,
        bytes: &[u8],
    ) -> Result<CandidateFrameResidencySettlement, CandidateFrameContractViolation> {
        let length = u32::try_from(bytes.len())
            .map_err(|_| CandidateFrameContractViolation::PhysicalWriteMismatch)?;
        let coordinate =
            RecordFrameCoordinate::new(coordinate.artifact(), coordinate.offset(), length)
                .ok_or(CandidateFrameContractViolation::PhysicalWriteMismatch)?;
        let exact = match &self.receipt {
            CandidateWriteReceipt::RetirementRetry(receipt, scope) => {
                scope.admits(coordinate, bytes)
                    && receipt.store() == store
                    && receipt.coordinate() == coordinate
                    && receipt.completed_bytes() == bytes.len() as u64
                    && receipt.payload_digest() == <[u8; 32]>::from(Sha256::digest(bytes))
            }
            CandidateWriteReceipt::New(receipt) => {
                completed_new_artifact_matches(receipt, self.coordinate, store, coordinate, bytes)
            }
            CandidateWriteReceipt::ArenaRange(receipt) => {
                let digest: [u8; 32] = Sha256::digest(bytes).into();
                matches!(
                    coordinate.artifact(),
                    worth_store_physical_format::RecordArtifactFile::ExtentArena { .. }
                ) && receipt.store() == store
                    && receipt.coordinate() == coordinate
                    && receipt.completed_bytes() == bytes.len() as u64
                    && receipt.payload_digest() == digest
            }
        };
        if !exact {
            return Err(CandidateFrameContractViolation::PhysicalWriteMismatch);
        }
        Ok(CandidateFrameResidencySettlement {
            settlement: self.settlement,
        })
    }
}

pub(super) fn completed_new_artifact_matches(
    receipt: &CompletedArtifactNewWrite,
    receipt_coordinate: RecordFrameCoordinate,
    store: StableStoreIdentity,
    coordinate: RecordFrameCoordinate,
    bytes: &[u8],
) -> bool {
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    receipt.store() == store
        && receipt_coordinate == coordinate
        && receipt.range().offset() == coordinate.offset()
        && receipt.range().byte_count() == bytes.len() as u64
        && receipt.completed_bytes() == bytes.len() as u64
        && receipt.payload_digest() == digest
}

impl CandidateFrameResidencySettlement {
    pub(in crate::physical_runtime::record_serving) const fn settlement(
        self,
    ) -> crate::physical_runtime::record_serving::CanonicalRecordMutationSettlement {
        self.settlement
    }
}

impl CandidateFrameEffectSettlement {
    fn canonical(
        coordinate: RecordFrameCoordinate,
        payload_digest: [u8; 32],
        settlement: crate::physical_runtime::record_serving::CanonicalRecordMutationSettlement,
    ) -> Self {
        Self {
            source: CandidateFrameEffectSource::NewArtifact,
            coordinate,
            payload_digest,
            work: settlement.identity(),
            effect: settlement.effect(),
            fate: settlement.effect_fate(),
            recovery: settlement.recovery(),
        }
    }

    fn writeback(
        coordinate: RecordFrameCoordinate,
        payload_digest: [u8; 32],
        settlement: crate::physical_runtime::record_serving::residency::dirty::
            PhysicalWritebackSettlement,
    ) -> Self {
        Self {
            source: CandidateFrameEffectSource::ExistingArtifactWriteback,
            coordinate,
            payload_digest,
            work: settlement.identity(),
            effect: settlement.effect(),
            fate: settlement.effect_fate(),
            recovery: settlement.recovery(),
        }
    }

    pub(in crate::physical_runtime) const fn source(self) -> CandidateFrameEffectSource {
        self.source
    }

    pub(in crate::physical_runtime) const fn coordinate(self) -> RecordFrameCoordinate {
        self.coordinate
    }

    pub(in crate::physical_runtime) const fn payload_digest(self) -> [u8; 32] {
        self.payload_digest
    }

    pub(in crate::physical_runtime) const fn work(self) -> PhysicalWorkIdentity {
        self.work
    }

    pub(in crate::physical_runtime) const fn effect(self) -> Option<PhysicalEffectIdentity> {
        self.effect
    }

    pub(in crate::physical_runtime) const fn fate(self) -> PhysicalWorkEffectFate {
        self.fate
    }

    pub(in crate::physical_runtime) const fn recovery(self) -> PhysicalWorkRecoveryDisposition {
        self.recovery
    }
}

#[derive(Debug)]
pub(in crate::physical_runtime::record_serving) struct CandidateFrameWriteCompletion {
    frame_bytes: u64,
    effect: Option<CandidateFrameEffectSettlement>,
}

impl CandidateFrameWriteCompletion {
    pub(in crate::physical_runtime::record_serving) fn canonical(
        frame_bytes: u64,
        coordinate: RecordFrameCoordinate,
        payload_digest: [u8; 32],
        settlement: crate::physical_runtime::record_serving::CanonicalRecordMutationSettlement,
    ) -> Self {
        Self {
            frame_bytes,
            effect: Some(CandidateFrameEffectSettlement::canonical(
                coordinate,
                payload_digest,
                settlement,
            )),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn writeback(
        frame_bytes: u64,
        coordinate: RecordFrameCoordinate,
        payload_digest: [u8; 32],
        settlement: crate::physical_runtime::record_serving::residency::dirty::
            PhysicalWritebackSettlement,
    ) -> Self {
        Self {
            frame_bytes,
            effect: Some(CandidateFrameEffectSettlement::writeback(
                coordinate,
                payload_digest,
                settlement,
            )),
        }
    }

    #[cfg(test)]
    pub(in crate::physical_runtime::record_serving) fn retained(frame_bytes: u64) -> Self {
        Self {
            frame_bytes,
            effect: None,
        }
    }

    pub(in crate::physical_runtime::record_serving) const fn frame_bytes(&self) -> u64 {
        self.frame_bytes
    }

    pub(in crate::physical_runtime) const fn effect(
        &self,
    ) -> Option<CandidateFrameEffectSettlement> {
        self.effect
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateFrameContractViolation {
    FrameCountExceedsDeclaration,
    FrameBytesExceedDeclaration,
    RetainedFrameMismatch,
    FrameCompletionMismatch,
    CoordinateRoleMismatch,
    UnexpectedFrame,
    RetainedFrameBytesChanged,
    PhysicalWriteMismatch,
    IncompleteFrameSet,
}

#[derive(Debug)]
pub(in crate::physical_runtime::record_serving) enum CandidateFrameWriteFailure<EffectFailure> {
    Contract {
        violation: CandidateFrameContractViolation,
        posture: CandidateFrameFailurePosture,
    },
    Effect(EffectFailure),
    Residency {
        denial: RecordAppendDenial,
        posture: CandidateFrameFailurePosture,
    },
}

pub(in crate::physical_runtime::record_serving) struct RecoverableCandidateFrameWriteFailure<
    EffectFailure,
> {
    cause: CandidateFrameWriteFailure<EffectFailure>,
    frame: super::CandidateFrame,
}

impl<EffectFailure> RecoverableCandidateFrameWriteFailure<EffectFailure> {
    pub(super) fn new(
        cause: CandidateFrameWriteFailure<EffectFailure>,
        frame: super::CandidateFrame,
    ) -> Self {
        Self { cause, frame }
    }

    pub(super) fn into_cause(self) -> CandidateFrameWriteFailure<EffectFailure> {
        self.cause
    }

    pub(in crate::physical_runtime::record_serving) fn into_parts(
        self,
    ) -> (CandidateFrameWriteFailure<EffectFailure>, Vec<u8>) {
        (self.cause, self.frame.into_bytes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) enum CandidateFrameFailurePosture {
    ProvenNoEffect,
    UnsettledBeforeEffect,
    EffectPossible,
}
