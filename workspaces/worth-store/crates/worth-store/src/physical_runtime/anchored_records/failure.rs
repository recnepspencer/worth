//! Typed refusals and fates of the anchored-record door.

use crate::physical_runtime::{
    IndeterminatePhysicalMutation, PhysicalMutationIdempotencyIssuanceDenial,
    PhysicalMutationPreparationOutcome, PhysicalReadProtectionDenial,
    ProvenNoEffectPhysicalMutation, RecordAppendDenial, RecordReadError, RecordStreamFailure,
};

/// A publication that did not make its value durable. Only `Indeterminate`,
/// `CompletedWithoutRecord` and `MemberWrite` may have had an effect, and
/// only `Indeterminate` and `CompletedWithoutRecord` may have moved the anchor.
#[non_exhaustive]
pub enum AnchoredRecordPublicationFailure {
    EmptyPayload,
    /// More members than `MAXIMUM_ANCHORED_RECORD_MEMBERS`. No effect.
    TooManyMembers {
        members: usize,
    },
    /// The new member at `index` does not hash to its key. No effect.
    MemberKeyMismatch {
        index: usize,
    },
    /// The new member at `index` is larger than one batch may carry. No
    /// effect.
    MemberTooLarge {
        index: usize,
    },
    /// The carried member at `index` is not a member of the current value.
    /// No effect.
    CarriedMemberNotCurrent {
        index: usize,
    },
    /// The anchor is not at the expected generation. No effect.
    GenerationConflict {
        current: u64,
    },
    /// A new name would exceed the root's anchor table. No effect.
    AnchorTableFull,
    /// An earlier publication to this name has an indeterminate fate; recovery
    /// must establish it before the name admits another. No effect.
    Unresolved,
    /// The anchored record could not form an admissible batch. No effect.
    Batch(RecordAppendDenial),
    Idempotency(PhysicalMutationIdempotencyIssuanceDenial),
    /// Physical preparation refused before any WAL effect.
    Preparation(PhysicalMutationPreparationOutcome),
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    /// The publication may or may not be durable.
    Indeterminate(IndeterminatePhysicalMutation),
    /// The mutation completed but did not report exactly one record. It may
    /// be durable; reopening establishes the current value.
    CompletedWithoutRecord,
    /// Writing the new members stopped after `written_batches` durable member
    /// batches. The anchor did not move, so the value was not published; any
    /// member records written are unreferenced.
    MemberWrite {
        written_batches: usize,
        failure: AnchoredRecordMemberWriteFailure,
    },
    /// Reading the current anchor or value failed before any effect.
    Read(AnchoredRecordReadFailure),
}

/// Why one batch of new members was not reported durable. Only
/// `Indeterminate` and `CompletedWithoutRecords` may have written records.
#[non_exhaustive]
pub enum AnchoredRecordMemberWriteFailure {
    Batch(RecordAppendDenial),
    Idempotency(PhysicalMutationIdempotencyIssuanceDenial),
    Preparation(PhysicalMutationPreparationOutcome),
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    Indeterminate(IndeterminatePhysicalMutation),
    CompletedWithoutRecords,
}

impl std::fmt::Debug for AnchoredRecordPublicationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPayload => f.write_str("EmptyPayload"),
            Self::TooManyMembers { members } => f
                .debug_struct("TooManyMembers")
                .field("members", members)
                .finish(),
            Self::MemberKeyMismatch { index } => f
                .debug_struct("MemberKeyMismatch")
                .field("index", index)
                .finish(),
            Self::MemberTooLarge { index } => f
                .debug_struct("MemberTooLarge")
                .field("index", index)
                .finish(),
            Self::CarriedMemberNotCurrent { index } => f
                .debug_struct("CarriedMemberNotCurrent")
                .field("index", index)
                .finish(),
            Self::GenerationConflict { current } => f
                .debug_struct("GenerationConflict")
                .field("current", current)
                .finish(),
            Self::AnchorTableFull => f.write_str("AnchorTableFull"),
            Self::Unresolved => f.write_str("Unresolved"),
            Self::Batch(denial) => f.debug_tuple("Batch").field(denial).finish(),
            Self::Idempotency(_) => f.write_str("Idempotency"),
            Self::Preparation(_) => f.write_str("Preparation"),
            Self::ProvenNoEffect(fate) => f
                .debug_struct("ProvenNoEffect")
                .field("cause", &fate.cause())
                .field("detail", &fate.admission_detail())
                .finish(),
            Self::Indeterminate(fate) => f
                .debug_struct("Indeterminate")
                .field("stage", &fate.stage())
                .finish(),
            Self::CompletedWithoutRecord => f.write_str("CompletedWithoutRecord"),
            Self::MemberWrite {
                written_batches,
                failure,
            } => f
                .debug_struct("MemberWrite")
                .field("written_batches", written_batches)
                .field("failure", failure)
                .finish(),
            Self::Read(failure) => f.debug_tuple("Read").field(failure).finish(),
        }
    }
}

impl std::fmt::Debug for AnchoredRecordMemberWriteFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Batch(denial) => f.debug_tuple("Batch").field(denial).finish(),
            Self::Idempotency(_) => f.write_str("Idempotency"),
            Self::Preparation(_) => f.write_str("Preparation"),
            Self::ProvenNoEffect(fate) => f
                .debug_struct("ProvenNoEffect")
                .field("cause", &fate.cause())
                .field("detail", &fate.admission_detail())
                .finish(),
            Self::Indeterminate(fate) => f
                .debug_struct("Indeterminate")
                .field("stage", &fate.stage())
                .finish(),
            Self::CompletedWithoutRecords => f.write_str("CompletedWithoutRecords"),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum AnchoredRecordReadFailure {
    Protection(PhysicalReadProtectionDenial),
    Open(RecordReadError),
    Stream(RecordStreamFailure),
    /// The anchored record failed frame or digest validation.
    Damaged(AnchoredRecordDamage),
    /// The payload exceeds the caller's limit; nothing was allocated for it.
    LimitExceeded {
        payload_bytes: u64,
    },
}

/// Why an anchored record is not the value its anchor selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchoredRecordDamage {
    Truncated,
    WrongMagic,
    /// The frame names a different anchor.
    WrongAnchor,
    /// The frame's generation is not the anchor's.
    WrongGeneration,
    /// The member table is longer than any value may be, or names no record.
    MalformedMemberTable,
    /// The record is longer than its frame declares.
    TrailingBytes,
    /// The whole record does not hash to the anchor's record digest.
    RecordDigestMismatch,
    /// The payload does not hash to the frame's payload digest.
    PayloadDigestMismatch,
}

/// Why one member could not be read. Never absence: a locator the current
/// value does not list refuses as `NotCurrentMember`.
#[derive(Debug)]
#[non_exhaustive]
pub enum AnchoredRecordMemberReadFailure {
    /// The current value, whose table decides membership, could not be read
    /// and verified.
    Value(AnchoredRecordReadFailure),
    /// The anchor's current value does not list this locator.
    NotCurrentMember,
    /// The member is longer than the caller's limit; nothing was allocated.
    LimitExceeded {
        length: u64,
    },
    Open(RecordReadError),
    Stream(RecordStreamFailure),
    /// The member's bytes are not the content its locator names.
    Damaged(AnchoredRecordMemberDamage),
}

/// Why a member record is not the content its locator names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchoredRecordMemberDamage {
    /// The record ends before the locator's length.
    Truncated,
    /// The record is longer than the locator's length.
    TrailingBytes,
    /// The bytes do not hash to the locator's SHA-256.
    DigestMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchoredRecordsDenial {
    ServingRequiresInspection,
}
