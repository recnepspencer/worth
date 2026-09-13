use std::sync::Arc;

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_backend::PhysicalRecoveryMediaGeneration;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, PhysicalCheckpointIdentity,
    PhysicalRecordFormatDeclaration,
};
use worth_store_physical_integrity::VerifiedCheckpointStream;
use worth_store_wal::{LogSequenceNumber, WalLsnRange, WalSegmentArtifactIdentity};

use crate::physical_runtime::recovery_coordination::PhysicalRecoveryCleanupRemovalCommand;
use crate::physical_runtime::{
    PhysicalRecoveryCleanupFreshnessReadDenial, PhysicalRecoveryCleanupFreshnessReadOutcome,
    PhysicalRecoveryCleanupRemovalOutcome, PhysicalRecoveryCoordination,
};

mod plan;
pub(in crate::physical_runtime) use plan::admit as admit_plan;
pub use plan::{StoreRecoveryCleanupPlan, StoreRecoveryCleanupPlanAdmissionFailure};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRecoveryCleanupFreshnessSample {
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    observed_published_generation: u64,
    cleanup_plan_identity: [u8; 32],
    sealed_publication_basis: [u8; 32],
    policy_identity: [u8; 32],
    selector_read: worth_store_physical_backend::CompletedScheduledRecoveryReopenRead,
    selector_read_operation: worth_store_physical_backend::MediaOperationIdentity,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
}

/// Owner-sampled descriptive evidence plus the only Store-admitted command
/// that may follow that same sampling occurrence.
pub(in crate::physical_runtime) struct StoreRecoveryCleanupFreshnessAdmission {
    sample: StoreRecoveryCleanupFreshnessSample,
    command: Option<PhysicalRecoveryCleanupRemovalCommand>,
}

pub enum StoreRecoveryCleanupAttempt {
    FreshnessDenied(StoreRecoveryCleanupFreshnessFailure),
    PublishedGenerationChanged(StoreRecoveryCleanupFreshnessSample),
    Removal {
        freshness: StoreRecoveryCleanupFreshnessSample,
        outcome: PhysicalRecoveryCleanupRemovalOutcome,
    },
}

pub struct StoreRecoveryCleanupFreshnessFailure {
    denial: StoreRecoveryCleanupFreshnessDenial,
    sample: Option<StoreRecoveryCleanupFreshnessSample>,
    read: Option<PhysicalRecoveryCleanupFreshnessReadDenial>,
    binding: Option<super::StoreRecoveryBindingSampleFailure>,
    terminal_binding_evaluations: u64,
}

/// Store-issued, consuming eligibility for one exact cleanup candidate.
///
/// Callers cannot construct or widen this value from raw coordinates. It is
/// issued only after performed fresh reopen, verified checkpoint coverage,
/// exact verified WAL facts, Store/session bindings, and the cleanup plan are
/// bound together.
pub(super) struct StoreRecoveryCleanupEligibility {
    wal: crate::physical_runtime::IntegrityAdmittedRecoveryWalSegment,
    removal: StoreRecoveryCleanupRemovalBasis,
}

pub(in crate::physical_runtime) struct StoreRecoveryCleanupRemovalBasis {
    store: StableStoreIdentity,
    media_generation: PhysicalRecoveryMediaGeneration,
    session: [u8; 16],
    plan: [u8; 32],
    published_generation: u64,
    format: PhysicalRecordFormatDeclaration,
    sealed_publication_basis: [u8; 32],
    checkpoint: PhysicalCheckpointIdentity,
    compaction_generation: u64,
    compaction_digest: [u8; 32],
    retained_boundary: LogSequenceNumber,
    artifact: WalSegmentArtifactIdentity,
    lsn_range: WalLsnRange,
    byte_count: u64,
    root_read: worth_store_physical_backend::CompletedScheduledRecoveryReopenRead,
}

struct PendingCleanupSample {
    cleanup_plan_identity: [u8; 32],
    policy_identity: [u8; 32],
    checkpoint: Arc<VerifiedCheckpointStream>,
    eligibility: StoreRecoveryCleanupEligibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreRecoveryCleanupFreshnessDenial {
    FreshnessMediaMismatch,
    CurrentSelectorRead,
    InvalidCleanupEligibility,
}

pub(in crate::physical_runtime) fn sample(
    authority: &super::PhysicalRecoveryFreshnessAuthority,
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    plan: &mut StoreRecoveryCleanupPlan,
    artifact: WalSegmentArtifactIdentity,
) -> Result<StoreRecoveryCleanupFreshnessAdmission, StoreRecoveryCleanupFreshnessFailure> {
    validate_sampling_bindings(authority, coordination, media, plan)?;
    let pending = take_pending_sample(plan, artifact)?;
    let sample = read_current_sample(coordination, media, &pending)?;
    #[cfg(feature = "certification-test-authority")]
    if coordination.take_certification_cleanup_eligibility_failure() {
        return Err(StoreRecoveryCleanupFreshnessFailure {
            denial: StoreRecoveryCleanupFreshnessDenial::InvalidCleanupEligibility,
            sample: Some(sample),
            read: None,
            binding: None,
            terminal_binding_evaluations: 0,
        });
    }
    let PendingCleanupSample {
        checkpoint,
        eligibility,
        ..
    } = pending;
    let command =
        if sample.observed_published_generation == eligibility.removal.published_generation {
            Some(PhysicalRecoveryCleanupRemovalCommand::from_freshness(
                eligibility.removal,
                sample.selector_read.clone(),
                checkpoint,
                eligibility.wal,
            ))
        } else {
            None
        };
    Ok(StoreRecoveryCleanupFreshnessAdmission { sample, command })
}

fn validate_sampling_bindings(
    authority: &super::PhysicalRecoveryFreshnessAuthority,
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    plan: &StoreRecoveryCleanupPlan,
) -> Result<(), StoreRecoveryCleanupFreshnessFailure> {
    if authority.matches_media_generation(media.media_generation())
        && plan.bindings_match(coordination, media)
    {
        Ok(())
    } else {
        Err(freshness_failure(
            StoreRecoveryCleanupFreshnessDenial::FreshnessMediaMismatch,
        ))
    }
}

fn take_pending_sample(
    plan: &mut StoreRecoveryCleanupPlan,
    artifact: WalSegmentArtifactIdentity,
) -> Result<PendingCleanupSample, StoreRecoveryCleanupFreshnessFailure> {
    Ok(PendingCleanupSample {
        cleanup_plan_identity: plan.identity(),
        policy_identity: plan.policy_identity(),
        checkpoint: plan.checkpoint(),
        eligibility: plan.take(artifact).ok_or_else(|| {
            freshness_failure(StoreRecoveryCleanupFreshnessDenial::InvalidCleanupEligibility)
        })?,
    })
}

fn read_current_sample(
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    pending: &PendingCleanupSample,
) -> Result<StoreRecoveryCleanupFreshnessSample, StoreRecoveryCleanupFreshnessFailure> {
    let completed = match coordination
        .read_cleanup_current_selector(media, pending.eligibility.removal.format)
    {
        PhysicalRecoveryCleanupFreshnessReadOutcome::Completed(completed) => completed,
        PhysicalRecoveryCleanupFreshnessReadOutcome::Denied(denial) => {
            return Err(StoreRecoveryCleanupFreshnessFailure {
                denial: StoreRecoveryCleanupFreshnessDenial::CurrentSelectorRead,
                sample: None,
                read: Some(denial),
                binding: None,
                terminal_binding_evaluations: 0,
            })
        }
    };
    let observed_published_generation = completed.selector().root_generation();
    #[cfg(feature = "certification-test-authority")]
    let observed_published_generation =
        if coordination.take_certification_cleanup_generation_shift() {
            observed_published_generation
                .checked_add(1)
                .expect("certification generation shift remains representable")
        } else {
            observed_published_generation
        };
    let sample = StoreRecoveryCleanupFreshnessSample {
        store: media.store_identity(),
        observed_published_generation,
        cleanup_plan_identity: pending.cleanup_plan_identity,
        sealed_publication_basis: pending.eligibility.removal.sealed_publication_basis,
        policy_identity: pending.policy_identity,
        selector_read: completed.physical().clone(),
        selector_read_operation: completed.physical().physical().operation(),
        work: completed.work(),
        signal: completed.signal(),
    };
    Ok(sample)
}

fn wal_members_are_terminal(binding: &super::StoreRecoveryBindingFreshnessSample) -> bool {
    !binding.wal_members().is_empty()
        && binding.wal_members().iter().all(|member| {
            binding
                .operations()
                .binary_search_by_key(&member.operation_identity(), |operation| {
                    operation.idempotency_identity()
                })
                .ok()
                .and_then(|index| binding.operations().get(index))
                .is_some_and(|operation| {
                    operation.fate() != super::StoreRecoveryOperationFate::Indeterminate
                })
        })
}

fn freshness_failure(
    denial: StoreRecoveryCleanupFreshnessDenial,
) -> StoreRecoveryCleanupFreshnessFailure {
    StoreRecoveryCleanupFreshnessFailure {
        denial,
        sample: None,
        read: None,
        binding: None,
        terminal_binding_evaluations: 0,
    }
}

impl StoreRecoveryCleanupFreshnessSample {
    pub const fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.store
    }
    pub const fn observed_published_generation(&self) -> u64 {
        self.observed_published_generation
    }
    pub const fn cleanup_plan_identity(&self) -> [u8; 32] {
        self.cleanup_plan_identity
    }
    pub const fn sealed_publication_basis(&self) -> [u8; 32] {
        self.sealed_publication_basis
    }
    pub const fn policy_identity(&self) -> [u8; 32] {
        self.policy_identity
    }
    pub const fn selector_read_operation(
        &self,
    ) -> worth_store_physical_backend::MediaOperationIdentity {
        self.selector_read_operation
    }
    pub const fn selector_read(
        &self,
    ) -> &worth_store_physical_backend::CompletedScheduledRecoveryReopenRead {
        &self.selector_read
    }
    pub const fn work(&self) -> crate::physical_runtime::PhysicalWorkIdentity {
        self.work
    }
    pub const fn signal(&self) -> crate::physical_runtime::PhysicalSignalSettlementOutcome {
        self.signal
    }
}

impl StoreRecoveryCleanupFreshnessAdmission {
    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        StoreRecoveryCleanupFreshnessSample,
        Option<PhysicalRecoveryCleanupRemovalCommand>,
    ) {
        (self.sample, self.command)
    }
}

impl StoreRecoveryCleanupAttempt {
    pub const fn freshness(&self) -> Option<&StoreRecoveryCleanupFreshnessSample> {
        match self {
            Self::FreshnessDenied(failure) => failure.sample(),
            Self::PublishedGenerationChanged(sample)
            | Self::Removal {
                freshness: sample, ..
            } => Some(sample),
        }
    }
}

impl StoreRecoveryCleanupFreshnessFailure {
    pub const fn denial(&self) -> StoreRecoveryCleanupFreshnessDenial {
        self.denial
    }
    pub const fn read(&self) -> Option<&PhysicalRecoveryCleanupFreshnessReadDenial> {
        self.read.as_ref()
    }
    pub const fn sample(&self) -> Option<&StoreRecoveryCleanupFreshnessSample> {
        self.sample.as_ref()
    }
    pub const fn binding(&self) -> Option<&super::StoreRecoveryBindingSampleFailure> {
        self.binding.as_ref()
    }

    pub const fn terminal_binding_evaluations(&self) -> u64 {
        self.terminal_binding_evaluations
    }
}

impl StoreRecoveryCleanupRemovalBasis {
    pub(in crate::physical_runtime) const fn store(&self) -> StableStoreIdentity {
        self.store
    }
    pub(in crate::physical_runtime) const fn media_generation(
        &self,
    ) -> PhysicalRecoveryMediaGeneration {
        self.media_generation
    }
    pub(in crate::physical_runtime) const fn session(&self) -> [u8; 16] {
        self.session
    }
    pub(in crate::physical_runtime) const fn plan(&self) -> [u8; 32] {
        self.plan
    }
    pub(in crate::physical_runtime) const fn published_generation(&self) -> u64 {
        self.published_generation
    }
    pub(in crate::physical_runtime) const fn format(&self) -> PhysicalRecordFormatDeclaration {
        self.format
    }
    pub(in crate::physical_runtime) const fn checkpoint(&self) -> PhysicalCheckpointIdentity {
        self.checkpoint
    }
    pub(in crate::physical_runtime) const fn compaction_generation(&self) -> u64 {
        self.compaction_generation
    }
    pub(in crate::physical_runtime) const fn compaction_digest(&self) -> [u8; 32] {
        self.compaction_digest
    }
    pub(in crate::physical_runtime) const fn retained_boundary(&self) -> LogSequenceNumber {
        self.retained_boundary
    }
    pub(in crate::physical_runtime) const fn artifact(&self) -> WalSegmentArtifactIdentity {
        self.artifact
    }
    pub(in crate::physical_runtime) const fn lsn_range(&self) -> WalLsnRange {
        self.lsn_range
    }
    pub(in crate::physical_runtime) const fn byte_count(&self) -> u64 {
        self.byte_count
    }
    pub(in crate::physical_runtime) fn root_read(
        &self,
    ) -> worth_store_physical_backend::CompletedScheduledRecoveryReopenRead {
        self.root_read.clone()
    }
}
