use std::marker::PhantomData;

use worth_proof::{ActionMarker, Performed};
use worth_store_physical_backend::{
    CompletedArtifactTreePublicationEffect, CompletedRecoveryStagingWrite,
};

mod cleanup;
mod publication_candidate;
mod reopen;
mod resident_storage;

pub use cleanup::{RecoveryCleanupRemovalAction, RecoveryCleanupRemovalOccurrence};
pub(in crate::physical_runtime::recovery_coordination) use cleanup::{
    RecoveryCleanupRemovalBinding, RecoveryCleanupRemovalSettlement, RecoveryCleanupRemovalTarget,
};

worth_proof::authority_marker!(RecoveryPhysicalEffectAuthority);

pub struct RecoveryStagingWriteAction;
impl ActionMarker for RecoveryStagingWriteAction {}

pub struct RecoveryStagingSynchronizationAction;
impl ActionMarker for RecoveryStagingSynchronizationAction {}

pub struct RecoveryRootProtocolReplacementAction;
impl ActionMarker for RecoveryRootProtocolReplacementAction {}

pub struct RecoveryRecordNamespaceSynchronizationAction;
impl ActionMarker for RecoveryRecordNamespaceSynchronizationAction {}

pub struct RecoveryPublicationCandidateMaterializationAction;
impl ActionMarker for RecoveryPublicationCandidateMaterializationAction {}

pub struct RecoveryPublicationCandidateSynchronizationAction;
impl ActionMarker for RecoveryPublicationCandidateSynchronizationAction {}

pub struct RecoveryFreshReopenAction;
impl ActionMarker for RecoveryFreshReopenAction {}

pub struct PerformedRecoveryPhysicalEffect<Action>
where
    Action: ActionMarker,
{
    evidence: Performed<Action, RecoveryPhysicalEffectAuthority, RecoveryPhysicalEffectOccurrence>,
    _action: PhantomData<Action>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecoveryPhysicalEffectOccurrence {
    StagingWrite(RecoveryStagingWriteOccurrence),
    StagingSynchronization(RecoveryStagingSynchronizationOccurrence),
    PublicationCandidateMaterialization(RecoveryPublicationCandidateMaterializationOccurrence),
    PublicationCandidateSynchronization(RecoveryPublicationCandidateSynchronizationOccurrence),
    FreshReopen(RecoveryFreshReopenOccurrence),
    CleanupRemoval(RecoveryCleanupRemovalOccurrence),
    RootProtocolReplacement(RecoveryPublicationOccurrence),
    RecordNamespaceSynchronization(RecoveryPublicationOccurrence),
}

#[derive(Debug, PartialEq, Eq)]
pub struct RecoveryPublicationOccurrence {
    session: [u8; 16],
    plan: [u8; 32],
    staging_generation: u64,
    publication: u64,
    physical: CompletedArtifactTreePublicationEffect,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    scheduler: crate::physical_runtime::PhysicalWorkSchedulerPosture,
    signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RecoveryPublicationCandidateMaterializationOccurrence {
    publication: RecoveryPublicationCandidateOccurrence,
    physical: CompletedRecoveryStagingWrite,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RecoveryPublicationCandidateSynchronizationOccurrence {
    publication: RecoveryPublicationCandidateOccurrence,
    physical: CompletedArtifactTreePublicationEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryPublicationCandidateOccurrence {
    session: [u8; 16],
    plan: [u8; 32],
    staging_generation: u64,
    publication: u64,
    artifact: worth_store_physical_format::RecordArtifactFile,
    ordinal: u64,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    scheduler: crate::physical_runtime::PhysicalWorkSchedulerPosture,
    signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RecoveryFreshReopenOccurrence {
    session: [u8; 16],
    plan: [u8; 32],
    generation: u64,
    selector: worth_store_physical_backend::CompletedScheduledRecoveryReopenRead,
    root: worth_store_physical_backend::CompletedScheduledRecoveryReopenRead,
    selector_work: crate::physical_runtime::PhysicalWorkIdentity,
    root_work: crate::physical_runtime::PhysicalWorkIdentity,
    selector_signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    root_signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RecoveryStagingWriteOccurrence {
    session: [u8; 16],
    plan: [u8; 32],
    staging_generation: u64,
    command_ordinal: u64,
    physical: CompletedRecoveryStagingWrite,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    scheduler: crate::physical_runtime::PhysicalWorkSchedulerPosture,
    signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RecoveryStagingSynchronizationOccurrence {
    session: [u8; 16],
    plan: [u8; 32],
    staging_generation: u64,
    command_ordinal: u64,
    physical: CompletedArtifactTreePublicationEffect,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    scheduler: crate::physical_runtime::PhysicalWorkSchedulerPosture,
    signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
}

impl PerformedRecoveryPhysicalEffect<RecoveryStagingWriteAction> {
    pub(super) fn record_write(outcome: RecoveryStagingWriteOccurrence) -> Self {
        Self {
            evidence: Performed::record(
                &RecoveryPhysicalEffectAuthority::witness(),
                RecoveryPhysicalEffectOccurrence::StagingWrite(outcome),
            ),
            _action: PhantomData,
        }
    }
}

impl PerformedRecoveryPhysicalEffect<RecoveryStagingSynchronizationAction> {
    pub(super) fn record_synchronization(
        outcome: RecoveryStagingSynchronizationOccurrence,
    ) -> Self {
        Self {
            evidence: Performed::record(
                &RecoveryPhysicalEffectAuthority::witness(),
                RecoveryPhysicalEffectOccurrence::StagingSynchronization(outcome),
            ),
            _action: PhantomData,
        }
    }
}

impl PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction> {
    pub(super) fn record_root_protocol(outcome: RecoveryPublicationOccurrence) -> Self {
        Self {
            evidence: Performed::record(
                &RecoveryPhysicalEffectAuthority::witness(),
                RecoveryPhysicalEffectOccurrence::RootProtocolReplacement(outcome),
            ),
            _action: PhantomData,
        }
    }
}

impl PerformedRecoveryPhysicalEffect<RecoveryRecordNamespaceSynchronizationAction> {
    pub(super) fn record_record_namespace(outcome: RecoveryPublicationOccurrence) -> Self {
        Self {
            evidence: Performed::record(
                &RecoveryPhysicalEffectAuthority::witness(),
                RecoveryPhysicalEffectOccurrence::RecordNamespaceSynchronization(outcome),
            ),
            _action: PhantomData,
        }
    }
}

impl PerformedRecoveryPhysicalEffect<RecoveryPublicationCandidateMaterializationAction> {
    pub(super) fn record_candidate_materialization(
        outcome: RecoveryPublicationCandidateMaterializationOccurrence,
    ) -> Self {
        Self {
            evidence: Performed::record(
                &RecoveryPhysicalEffectAuthority::witness(),
                RecoveryPhysicalEffectOccurrence::PublicationCandidateMaterialization(outcome),
            ),
            _action: PhantomData,
        }
    }
}

impl PerformedRecoveryPhysicalEffect<RecoveryPublicationCandidateSynchronizationAction> {
    pub(super) fn record_candidate_synchronization(
        outcome: RecoveryPublicationCandidateSynchronizationOccurrence,
    ) -> Self {
        Self {
            evidence: Performed::record(
                &RecoveryPhysicalEffectAuthority::witness(),
                RecoveryPhysicalEffectOccurrence::PublicationCandidateSynchronization(outcome),
            ),
            _action: PhantomData,
        }
    }
}

impl PerformedRecoveryPhysicalEffect<RecoveryFreshReopenAction> {
    pub(super) fn record_fresh_reopen(outcome: RecoveryFreshReopenOccurrence) -> Self {
        Self {
            evidence: Performed::record(
                &RecoveryPhysicalEffectAuthority::witness(),
                RecoveryPhysicalEffectOccurrence::FreshReopen(outcome),
            ),
            _action: PhantomData,
        }
    }
}

impl<Action> PerformedRecoveryPhysicalEffect<Action>
where
    Action: ActionMarker,
{
    pub fn occurrence(&self) -> &RecoveryPhysicalEffectOccurrence {
        self.evidence.outcome()
    }

    pub fn into_occurrence(self) -> RecoveryPhysicalEffectOccurrence {
        self.evidence.into_outcome()
    }
}

impl RecoveryStagingWriteOccurrence {
    #[allow(clippy::too_many_arguments)]
    pub(super) const fn new(
        session: [u8; 16],
        plan: [u8; 32],
        staging_generation: u64,
        command_ordinal: u64,
        physical: CompletedRecoveryStagingWrite,
        work: crate::physical_runtime::PhysicalWorkIdentity,
        scheduler: crate::physical_runtime::PhysicalWorkSchedulerPosture,
        signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    ) -> Self {
        Self {
            session,
            plan,
            staging_generation,
            command_ordinal,
            physical,
            work,
            scheduler,
            signal,
        }
    }

    pub const fn physical(&self) -> &CompletedRecoveryStagingWrite {
        &self.physical
    }
    pub const fn session(&self) -> [u8; 16] {
        self.session
    }
    pub const fn plan(&self) -> [u8; 32] {
        self.plan
    }
    pub const fn staging_generation(&self) -> u64 {
        self.staging_generation
    }
    pub const fn command_ordinal(&self) -> u64 {
        self.command_ordinal
    }
    pub const fn work(&self) -> crate::physical_runtime::PhysicalWorkIdentity {
        self.work
    }
    pub const fn scheduler(&self) -> crate::physical_runtime::PhysicalWorkSchedulerPosture {
        self.scheduler
    }
    pub const fn signal(&self) -> crate::physical_runtime::PhysicalSignalSettlementOutcome {
        self.signal
    }
}

impl RecoveryStagingSynchronizationOccurrence {
    #[allow(clippy::too_many_arguments)]
    pub(super) const fn new(
        session: [u8; 16],
        plan: [u8; 32],
        staging_generation: u64,
        command_ordinal: u64,
        physical: CompletedArtifactTreePublicationEffect,
        work: crate::physical_runtime::PhysicalWorkIdentity,
        scheduler: crate::physical_runtime::PhysicalWorkSchedulerPosture,
        signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    ) -> Self {
        Self {
            session,
            plan,
            staging_generation,
            command_ordinal,
            physical,
            work,
            scheduler,
            signal,
        }
    }

    pub const fn physical(&self) -> &CompletedArtifactTreePublicationEffect {
        &self.physical
    }
    pub const fn session(&self) -> [u8; 16] {
        self.session
    }
    pub const fn plan(&self) -> [u8; 32] {
        self.plan
    }
    pub const fn staging_generation(&self) -> u64 {
        self.staging_generation
    }
    pub const fn command_ordinal(&self) -> u64 {
        self.command_ordinal
    }
    pub const fn work(&self) -> crate::physical_runtime::PhysicalWorkIdentity {
        self.work
    }
    pub const fn scheduler(&self) -> crate::physical_runtime::PhysicalWorkSchedulerPosture {
        self.scheduler
    }
    pub const fn signal(&self) -> crate::physical_runtime::PhysicalSignalSettlementOutcome {
        self.signal
    }
}

impl RecoveryPublicationOccurrence {
    #[allow(clippy::too_many_arguments)]
    pub(super) const fn new(
        session: [u8; 16],
        plan: [u8; 32],
        staging_generation: u64,
        publication: u64,
        physical: CompletedArtifactTreePublicationEffect,
        work: crate::physical_runtime::PhysicalWorkIdentity,
        scheduler: crate::physical_runtime::PhysicalWorkSchedulerPosture,
        signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    ) -> Self {
        Self {
            session,
            plan,
            staging_generation,
            publication,
            physical,
            work,
            scheduler,
            signal,
        }
    }

    pub const fn physical(&self) -> &CompletedArtifactTreePublicationEffect {
        &self.physical
    }
    pub const fn session(&self) -> [u8; 16] {
        self.session
    }
    pub const fn plan(&self) -> [u8; 32] {
        self.plan
    }
    pub const fn staging_generation(&self) -> u64 {
        self.staging_generation
    }
    pub const fn publication(&self) -> u64 {
        self.publication
    }
    pub const fn work(&self) -> crate::physical_runtime::PhysicalWorkIdentity {
        self.work
    }
    pub const fn scheduler(&self) -> crate::physical_runtime::PhysicalWorkSchedulerPosture {
        self.scheduler
    }
    pub const fn signal(&self) -> crate::physical_runtime::PhysicalSignalSettlementOutcome {
        self.signal
    }
}
