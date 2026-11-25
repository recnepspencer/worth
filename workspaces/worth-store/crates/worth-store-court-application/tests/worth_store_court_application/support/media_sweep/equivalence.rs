//! Sound, content-verified post-recovery equivalence classes for deduplicating
//! the reopen-based `assert_continues_world` tail. If Store requires inspection
//! or any checkpoint manifest/member read fails, continuation runs unconditionally.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Condvar, Mutex};

use sha2::{Digest, Sha256};
use worth_execution::ExecutionAllocationPolicy;
use worth_relational::facade::schema::SchemaVersionId;
use worth_store::physical_runtime::{
    AnchoredRecordName, PhysicalDurabilityReopenObservation, RecordPublicationResidueObservation,
    ServingPhysicalRuntime,
};

use crate::branch_lifecycle::State;
use crate::crash_oracle::InFlight;
use crate::media_case::Ended;
use crate::media_sweep::manifest_digest::relational_manifest_digest;
use crate::media_sweep::signal_digest::verify_application_journals;
use crate::media_sweep::targets::{hash_byte_slices, hash_disk_layout, hash_logical_state};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum EndedKey {
    Skipped,
    Acknowledged,
    Refused,
    Unresolved { recovered: bool },
}

impl From<Ended> for EndedKey {
    fn from(value: Ended) -> Self {
        match value {
            Ended::Skipped => Self::Skipped,
            Ended::Acknowledged => Self::Acknowledged,
            Ended::Refused => Self::Refused,
            Ended::Unresolved { recovered } => Self::Unresolved { recovered },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum InFlightClass {
    Nothing,
    Refused,
    NotDurable,
    Durable,
    Unresolved,
}

impl From<&InFlight> for InFlightClass {
    fn from(value: &InFlight) -> Self {
        match value {
            InFlight::Nothing => Self::Nothing,
            InFlight::Refused => Self::Refused,
            InFlight::NotDurable => Self::NotDurable,
            InFlight::Durable => Self::Durable,
            InFlight::Unresolved(_) => Self::Unresolved,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PublicationResidueKey {
    pub(crate) staging_catalog_candidate: bool,
    pub(crate) successor_root: bool,
    pub(crate) successor_routing_block: bool,
    pub(crate) successor_free_space: bool,
    pub(crate) successor_membership_blocks: bool,
    pub(crate) next_segment_artifacts: bool,
    pub(crate) reusable_segment_artifacts: bool,
    pub(crate) next_arena_artifact: bool,
}

impl From<RecordPublicationResidueObservation> for PublicationResidueKey {
    fn from(obs: RecordPublicationResidueObservation) -> Self {
        Self {
            staging_catalog_candidate: obs.staging_catalog_candidate(),
            successor_root: obs.successor_root(),
            successor_routing_block: obs.successor_routing_block(),
            successor_free_space: obs.successor_free_space(),
            successor_membership_blocks: obs.successor_membership_blocks(),
            next_segment_artifacts: obs.next_segment_artifacts(),
            reusable_segment_artifacts: obs.reusable_segment_artifacts(),
            next_arena_artifact: obs.next_arena_artifact(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DurabilityReopenKey {
    pub(crate) checkpoint_artifact_bytes: u64,
    pub(crate) checkpoint_bytes_read: u64,
    pub(crate) dirty_body_bytes_skipped: u64,
    pub(crate) binding_records_read: u64,
    pub(crate) checkpoint_integrity_admissions: u64,
    pub(crate) wal_members_read: u64,
}

impl From<PhysicalDurabilityReopenObservation> for DurabilityReopenKey {
    fn from(obs: PhysicalDurabilityReopenObservation) -> Self {
        Self {
            checkpoint_artifact_bytes: obs.checkpoint_artifact_bytes(),
            checkpoint_bytes_read: obs.checkpoint_bytes_read(),
            dirty_body_bytes_skipped: obs.dirty_body_bytes_skipped(),
            binding_records_read: obs.binding_records_read(),
            checkpoint_integrity_admissions: obs.checkpoint_integrity_admissions(),
            wal_members_read: obs.wal_members_read(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct VerifiedPhysicalState {
    pub(crate) canonical_journal_head: u64,
    pub(crate) transition_journal_head: u64,
    pub(crate) checkpoint_generation: u64,
    pub(crate) checkpoint_relational_manifest_sha256: [u8; 32],
    pub(crate) checkpoint_member_count: usize,
    pub(crate) checkpoint_members_content_sha256: [u8; 32],
    pub(crate) application_journals_sha256: [u8; 32],
    pub(crate) publication_residue: PublicationResidueKey,
    pub(crate) durability_reopen: Option<DurabilityReopenKey>,
    pub(crate) recovery_journal_counters: (u64, u64),
}

/// Verify the full `home.checkpoint` payload SHA-256, all nested checkpoint
/// sections (Relational, World, Signal), every checkpoint member's actual bytes
/// from disk, and every entry across all non-Relational Application journals
/// (`signal.*`, `world.*`, `home.transitions`, `query.*`), plus post-readmission
/// residue and reopen counters. Returns `None` on any verification failure so
/// continuation runs unconditionally.
pub(crate) fn verify_serving_physical(
    serving: &ServingPhysicalRuntime,
) -> Option<VerifiedPhysicalState> {
    let canonical_journal_head = serving
        .canonical_commits()
        .ok()?
        .head()
        .ok()?
        .map_or(0, |entry| entry.position());
    let transition_journal_head = serving
        .branch_transitions()
        .ok()?
        .head()
        .ok()?
        .map_or(0, |entry| entry.position());
    let records = serving.anchored_records().ok()?;
    let anchor = AnchoredRecordName::new(crate::selected_checkpoint::ANCHOR).unwrap();
    let (
        checkpoint_generation,
        checkpoint_summary,
        checkpoint_member_count,
        checkpoint_members_content_sha256,
    ) = match records.read(anchor, 1 << 30).ok()? {
        None => (0, None, 0, [0u8; 32]),
        Some(record) => {
            let full_payload_sha256: [u8; 32] = Sha256::digest(record.bytes()).into();
            if full_payload_sha256 != record.version().payload_sha256() {
                return None;
            }
            let summary = relational_manifest_digest(record.bytes())?;
            let mut hasher = Sha256::new();
            hasher.update(record.version().generation().to_le_bytes());
            hasher.update((record.members().len() as u64).to_le_bytes());
            for member in record.members() {
                hasher.update(member.sha256());
                hasher.update(member.length().to_le_bytes());
                let bytes = records.read_member(anchor, *member, 1 << 30).ok()?;
                hasher.update((bytes.len() as u64).to_le_bytes());
                hasher.update(&bytes);
            }
            (
                record.version().generation(),
                Some(summary),
                record.members().len(),
                hasher.finalize().into(),
            )
        }
    };
    let application_journals_sha256 =
        verify_application_journals(serving, checkpoint_summary.as_ref())?;
    let checkpoint_relational_manifest_sha256 =
        checkpoint_summary.map_or([0u8; 32], |summary| summary.digest);
    let journal_counters = serving.physical_recovery_journal_counters();
    Some(VerifiedPhysicalState {
        canonical_journal_head,
        transition_journal_head,
        checkpoint_generation,
        checkpoint_relational_manifest_sha256,
        checkpoint_member_count,
        checkpoint_members_content_sha256,
        application_journals_sha256,
        publication_residue: serving.publication_residue().into(),
        durability_reopen: serving.durability_observation().reopen().map(Into::into),
        recovery_journal_counters: (
            journal_counters.records_written(),
            journal_counters.directory_barriers(),
        ),
    })
}

/// Complete post-recovery equivalence key for reopen-based continuation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PostRecoveryEquivalenceKey {
    pub(crate) ended: EndedKey,
    pub(crate) in_flight_class: InFlightClass,
    pub(crate) recovered_schema: u64,
    pub(crate) commits: usize,
    pub(crate) transitions: usize,
    pub(crate) commit_artifacts_sha256: [u8; 32],
    pub(crate) transition_artifacts_sha256: [u8; 32],
    pub(crate) logical_state_sha256: [u8; 32],
    pub(crate) transition_covered_sequence: u64,
    pub(crate) selected_checkpoint_generation: u64,
    pub(crate) physical: VerifiedPhysicalState,
    pub(crate) disk_layout_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EquivalenceStats {
    pub(crate) total_cases: usize,
    pub(crate) distinct_classes: usize,
    pub(crate) continuations_deduped: usize,
}

#[derive(Default)]
pub(crate) struct EquivalenceLedger {
    inner: Mutex<EquivalenceLedgerInner>,
    ready: Condvar,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ClassState {
    InProgress,
    Succeeded,
}

#[derive(Default)]
struct EquivalenceLedgerInner {
    states: BTreeMap<PostRecoveryEquivalenceKey, ClassState>,
    #[cfg(test)]
    waiters: usize,
    direct_continuations: usize,
    total_cases: usize,
    continuations_deduped: usize,
}

/// Guard held while a representative runs `assert_continues_world`. Calling
/// `mark_succeeded` records the class as proven and wakes waiting duplicates to
/// skip; dropping without calling `mark_succeeded` (e.g. on panic) removes the
/// entry and wakes waiters so duplicates execute continuation and fail loudly.
pub(crate) struct ContinuationGuard<'a> {
    ledger: &'a EquivalenceLedger,
    key: Option<PostRecoveryEquivalenceKey>,
}

impl ContinuationGuard<'_> {
    pub(crate) fn mark_succeeded(mut self) {
        if let Some(key) = self.key.take() {
            let mut guard = self.ledger.lock_inner();
            guard.states.insert(key, ClassState::Succeeded);
            self.ledger.ready.notify_all();
        }
    }
}

impl Drop for ContinuationGuard<'_> {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            let mut guard = self.ledger.lock_inner();
            guard.states.remove(&key);
            self.ledger.ready.notify_all();
        }
    }
}

impl EquivalenceLedger {
    fn lock_inner(&self) -> std::sync::MutexGuard<'_, EquivalenceLedgerInner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn claim_continuation(
        &self,
        key: PostRecoveryEquivalenceKey,
    ) -> Option<ContinuationGuard<'_>> {
        let mut guard = self.lock_inner();
        loop {
            match guard.states.get(&key).copied() {
                Some(ClassState::Succeeded) => {
                    guard.total_cases += 1;
                    guard.continuations_deduped += 1;
                    return None;
                }
                Some(ClassState::InProgress) => {
                    #[cfg(test)]
                    {
                        guard.waiters += 1;
                    }
                    guard = self
                        .ready
                        .wait(guard)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    #[cfg(test)]
                    {
                        guard.waiters -= 1;
                    }
                }
                None => {
                    guard.total_cases += 1;
                    guard.states.insert(key.clone(), ClassState::InProgress);
                    return Some(ContinuationGuard {
                        ledger: self,
                        key: Some(key),
                    });
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn waiter_count(&self) -> usize {
        self.lock_inner().waiters
    }

    pub(crate) fn record_direct_continuation(&self) {
        let mut guard = self.lock_inner();
        guard.total_cases += 1;
        guard.direct_continuations += 1;
    }

    pub(crate) fn stats(&self) -> EquivalenceStats {
        let guard = self.lock_inner();
        let succeeded = guard
            .states
            .values()
            .filter(|state| matches!(state, ClassState::Succeeded))
            .count();
        EquivalenceStats {
            total_cases: guard.total_cases,
            distinct_classes: succeeded + guard.direct_continuations,
            continuations_deduped: guard.continuations_deduped,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_equivalence_key(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    physical: VerifiedPhysicalState,
    ended: Ended,
    in_flight: &InFlight,
    recovered_schema: SchemaVersionId,
    commit_artifacts: &[Vec<u8>],
    transition_artifacts: &[Vec<u8>],
    logical_state: &State,
    selected_checkpoint_generation: u64,
) -> PostRecoveryEquivalenceKey {
    let transition_covered_sequence = runtime
        .durability()
        .branch_transition_artifacts(ExecutionAllocationPolicy::SystemAllocation)
        .covered_sequence()
        .0;
    PostRecoveryEquivalenceKey {
        ended: ended.into(),
        in_flight_class: in_flight.into(),
        recovered_schema: u64::from(recovered_schema.0),
        commits: commit_artifacts.len(),
        transitions: transition_artifacts.len(),
        commit_artifacts_sha256: hash_byte_slices(commit_artifacts),
        transition_artifacts_sha256: hash_byte_slices(transition_artifacts),
        logical_state_sha256: hash_logical_state(logical_state),
        transition_covered_sequence,
        selected_checkpoint_generation,
        physical,
        disk_layout_sha256: [0u8; 32],
    }
}

impl PostRecoveryEquivalenceKey {
    pub(crate) fn with_quiescent_disk_layout(mut self, location: &Path) -> Self {
        self.disk_layout_sha256 = hash_disk_layout(location);
        self
    }
}
