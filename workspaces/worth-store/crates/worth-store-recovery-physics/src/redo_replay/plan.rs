use super::cursor::RecoveryPageCursor;
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
    PhysicalRecoveryProjectionDecodeLimits,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AdmittedPhysicalRedoMember {
    lsn_range: WalLsnRange,
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    records: Box<[PhysicalRedoRecord]>,
    projection: PersistedPhysicalRecoveryProjection,
    inline_frames: Box<[projection_admission::AdmittedInlineFrame]>,
}
use worth_store_wal::WalLsnRange;

mod accessors;
mod admission;
mod allocation_truth;
mod group_admission;
mod projection_admission;
mod projection_materialization;
mod projection_validation;
mod supersession;

pub use admission::{
    admit_physical_redo_members, physical_redo_observation_target_identities,
    physical_redo_observation_targets, physical_redo_target_identities,
};

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRedoProjection {
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    materialization: PersistedPhysicalRecoveryProjection,
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
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalRedoPlanCounters {
    records: u64,
    targets: u64,
    apply: u64,
    skip_page_lsn: u64,
    skip_operation: u64,
}

pub fn plan_physical_redo(
    members: Vec<PhysicalRedoMemberInput>,
    observations: Vec<RecoveryPageObservation>,
    maximum_targets: u64,
    store: StableStoreIdentity,
) -> Result<ImmutablePhysicalRedoPlan, PhysicalRedoPlanningDenial> {
    let format = PhysicalRecordFormatDeclaration::builder()
        .admit()
        .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
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

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;
