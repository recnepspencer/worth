use super::cursor::{RecoveryPageCursor, RecoveryPageSource};
use super::record::{decode_physical_redo_member, decode_physical_redo_records_with_distinct};
use super::{
    decode_physical_redo_records, PhysicalRedoPlanningDenial, PhysicalRedoRecord,
    PhysicalRedoTarget, PhysicalRedoTargetIdentity, RecoveryPageObservation,
};
use crate::RecoveryOperationFate;
use std::collections::{BTreeMap, BTreeSet};
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PersistedPhysicalDataFrameSubject,
    PersistedPhysicalRecoveryProjection, PhysicalRecordFormatDeclaration,
    PhysicalRecoveryProjectionDecodeLimits, PhysicalRewriteRedo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRedoAdmissionLimits {
    pub recovery_memory_bytes: u64,
    pub targets: u64,
    pub distinct_targets: u64,
    pub projection: PhysicalRecoveryProjectionDecodeLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedPhysicalRedoMembers {
    scratch_bytes: u64,
    members: Box<[AdmittedPhysicalRedoMember]>,
    group_allocations: BTreeMap<[u8; 32], u64>,
    rewrites: Box<[PhysicalRewriteRedo]>,
    rewrite_admissions: Box<[PhysicalRewriteAdmission]>,
    source_copies: Box<[PhysicalExtentCopyAdmission]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdmittedPhysicalRedoMember {
    lsn_range: WalLsnRange,
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    records: Box<[PhysicalRedoRecord]>,
    projection: PersistedPhysicalRecoveryProjection,
    canonical_redo_sha256: [u8; 32],
    inline_frames: Box<[projection_admission::AdmittedInlineFrame]>,
}

/// A borrow of one C.9 semantics-admitted member before page observations
/// exist. Only an admitted member set can mint this view; a caller cannot
/// describe an arbitrary root-step projection by assembling its fields.
#[derive(Debug, Clone, Copy)]
pub struct AdmittedRootStepMemberView<'a> {
    lsn_range: WalLsnRange,
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    canonical_redo_sha256: [u8; 32],
    materialization: &'a PersistedPhysicalRecoveryProjection,
    records: &'a [PhysicalRedoRecord],
}
use worth_store_wal::WalLsnRange;

mod accessors;
mod admission;
mod allocation_truth;
mod group_admission;
mod head_replay;
mod historical_consumed;
mod historical_drop;
mod historical_retired;
mod projection_admission;
mod projection_materialization;
mod projection_validation;
mod retained_storage;
mod source_copy;
mod supersession;
pub use source_copy::{admit_current_source_copy_publication, PhysicalExtentCopyAdmission};

pub use admission::{
    admit_physical_redo_members, physical_redo_observation_target_identities,
    physical_redo_observation_targets, physical_redo_target_identities,
};
pub use head_replay::{
    SelectedReleaseHeadReplayDenial, VerifiedOrderedReleasedHeadReplayV14,
    VerifiedSelectedReleaseHeadReplayV14, VerifiedSelectedTerminalHeadRetirementReplay,
};
pub use historical_consumed::HistoricalConsumedOperationSet;
pub use historical_retired::HistoricalRetirements;

fn checked(value: u64) -> Result<u64, PhysicalRedoPlanningDenial> {
    value
        .checked_add(1)
        .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRedoMemberInput {
    lsn_range: WalLsnRange,
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    canonical_redo: Box<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalRedoGroupBinding {
    group_identity: [u8; 32],
    member_identity: [u8; 32],
    member_ordinal: u32,
    member_count: u32,
    membership_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImmutablePhysicalRedoPlan {
    scratch_bytes: u64,
    records: Box<[PhysicalRedoRecord]>,
    decisions: Box<[PhysicalRedoDecision]>,
    projections: Box<[PhysicalRedoProjection]>,
    recovery_root_allocation_bytes: u64,
    counters: PhysicalRedoPlanCounters,
    rewrites: Box<[PhysicalRewriteRedo]>,
    rewrite_admissions: Box<[PhysicalRewriteAdmission]>,
    source_copies: Box<[PhysicalExtentCopyAdmission]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRewriteAdmission {
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    redo: PhysicalRewriteRedo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRedoProjection {
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    materialization: PersistedPhysicalRecoveryProjection,
    canonical_redo_sha256: Option<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRedoDecision {
    kind: PhysicalRedoDecisionKind,
    prior: PhysicalRedoDecisionPrior,
    operation: [u8; 32],
    record_index: u64,
    target_index: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRedoDecisionPrior {
    OperationFate(RecoveryOperationFate),
    Page(RecoveryPageObservation),
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicalRedoDecisionView<'plan> {
    decision: &'plan PhysicalRedoDecision,
    record: &'plan PhysicalRedoRecord,
    target: &'plan PhysicalRedoTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRedoDecisionKind {
    Apply,
    SkipPageAlreadyAtOrBeyondLsn,
    SkipOperationAlreadyMaterialized,
    SkipHistoricallyReleasedTarget,
    SkipHistoricallyRetiredTarget,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalRedoPlanCounters {
    records: u64,
    targets: u64,
    apply: u64,
    skip_page_lsn: u64,
    skip_historical_drop: u64,
    skip_historical_retired: u64,
    skip_operation: u64,
}

pub fn plan_physical_redo(
    members: Vec<PhysicalRedoMemberInput>,
    observations: Vec<RecoveryPageObservation>,
    maximum_targets: u64,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> Result<ImmutablePhysicalRedoPlan, PhysicalRedoPlanningDenial> {
    admit_physical_redo_members(
        members,
        store,
        format,
        PhysicalRedoAdmissionLimits {
            recovery_memory_bytes: u64::MAX,
            targets: maximum_targets,
            distinct_targets: maximum_targets,
            projection: PhysicalRecoveryProjectionDecodeLimits {
                frames: maximum_targets,
                record_identities: maximum_targets,
                placements: maximum_targets,
                segment_updates: maximum_targets,
                manifests: maximum_targets,
                total_entries: maximum_targets.saturating_mul(3),
                inline_allocations: maximum_targets,
            },
        },
    )?
    .plan(observations)
}

fn decide(
    operation: [u8; 32],
    fate: RecoveryOperationFate,
    record: &PhysicalRedoRecord,
    target: &PhysicalRedoTarget,
    record_index: u64,
    target_index: u64,
    page_cursor: &mut RecoveryPageCursor,
    counters: &mut PhysicalRedoPlanCounters,
) -> Result<PhysicalRedoDecision, PhysicalRedoPlanningDenial> {
    let record_lsn = record.lsn().get();
    match fate {
        RecoveryOperationFate::AcknowledgedDurable
        | RecoveryOperationFate::DurableUnacknowledged => {
            counters.skip_operation = checked(counters.skip_operation)?;
            Ok(PhysicalRedoDecision {
                kind: PhysicalRedoDecisionKind::SkipOperationAlreadyMaterialized,
                prior: PhysicalRedoDecisionPrior::OperationFate(fate),
                operation,
                record_index,
                target_index,
            })
        }
        RecoveryOperationFate::ProvenNoEffect => {
            Err(PhysicalRedoPlanningDenial::ProvenNoEffectHasWalAttempt)
        }
        RecoveryOperationFate::Indeterminate => {
            let observation = page_cursor.observe_record(target.identity(), record_lsn)?;
            let historical = if page_cursor.is_observed_predecessor(target.identity(), record_lsn) {
                retired_predecessor(observation)
            } else {
                historical_skip(observation, operation, target)?
            };
            if let Some(kind) = historical {
                let counter = match kind {
                    PhysicalRedoDecisionKind::SkipHistoricallyReleasedTarget => {
                        &mut counters.skip_historical_drop
                    }
                    _ => &mut counters.skip_historical_retired,
                };
                *counter = checked(*counter)?;
                return Ok(PhysicalRedoDecision {
                    kind,
                    prior: PhysicalRedoDecisionPrior::Page(observation),
                    operation,
                    record_index,
                    target_index,
                });
            }
            let page_lsn = observation.page_lsn();
            if page_lsn == record_lsn && observation.frame_digest() != target.resulting_digest() {
                return Err(PhysicalRedoPlanningDenial::PageDigestMismatch);
            }
            if page_lsn >= record_lsn {
                counters.skip_page_lsn = checked(counters.skip_page_lsn)?;
                Ok(PhysicalRedoDecision {
                    kind: PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn,
                    prior: PhysicalRedoDecisionPrior::Page(observation),
                    operation,
                    record_index,
                    target_index,
                })
            } else {
                counters.apply = checked(counters.apply)?;
                page_cursor.advance(operation, target, record_lsn)?;
                Ok(PhysicalRedoDecision {
                    kind: PhysicalRedoDecisionKind::Apply,
                    prior: PhysicalRedoDecisionPrior::Page(observation),
                    operation,
                    record_index,
                    target_index,
                })
            }
        }
    }
}

/// An older image of a page whose last image is historically retired was
/// superseded by that image and is skipped with it. Any other anchored
/// predecessor falls through to ordinary page-LSN comparison.
fn retired_predecessor(observation: RecoveryPageObservation) -> Option<PhysicalRedoDecisionKind> {
    matches!(
        observation.source(),
        RecoveryPageSource::HistoricalRetiredTarget { .. }
    )
    .then_some(PhysicalRedoDecisionKind::SkipHistoricallyRetiredTarget)
}

/// A historical classification must name this exact older WAL image; any
/// other page source falls through to ordinary page-LSN comparison.
fn historical_skip(
    observation: RecoveryPageObservation,
    operation: [u8; 32],
    target: &PhysicalRedoTarget,
) -> Result<Option<PhysicalRedoDecisionKind>, PhysicalRedoPlanningDenial> {
    let (kind, coordinate, old_operation, wal_target_digest) = match observation.source() {
        RecoveryPageSource::HistoricalReleasedDrop {
            coordinate,
            old_operation,
            wal_target_digest,
            ..
        } => (
            PhysicalRedoDecisionKind::SkipHistoricallyReleasedTarget,
            coordinate,
            old_operation,
            wal_target_digest,
        ),
        RecoveryPageSource::HistoricalRetiredTarget {
            coordinate,
            old_operation,
            wal_target_digest,
            ..
        } => (
            PhysicalRedoDecisionKind::SkipHistoricallyRetiredTarget,
            coordinate,
            old_operation,
            wal_target_digest,
        ),
        _ => return Ok(None),
    };
    if observation.target() != target.identity()
        || old_operation != operation
        || wal_target_digest != target.resulting_digest()
        || Some(coordinate)
            != worth_store_physical_format::RecordFrameCoordinate::new(
                target.artifact(),
                target.artifact_offset(),
                target.artifact_length(),
            )
    {
        return Err(PhysicalRedoPlanningDenial::GenerationMismatch);
    }
    Ok(Some(kind))
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;
