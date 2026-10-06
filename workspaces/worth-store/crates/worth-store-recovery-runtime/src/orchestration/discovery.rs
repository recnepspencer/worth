use worth_store_recovery_physics::PhysicalBootstrapFallbackAnchor;
use worth_store_recovery_physics::{PhysicalRecoveryResidue, PhysicalRootSlotObservation};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlockCause, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind as PhysicalRecoveryBlock, PhysicalRecoveryLimitDimension,
    PhysicalRecoverySourceDenial,
};

use super::reader_limit::UNCOUNTED_READS;
use super::recovery_budget::{ExceededRecoveryLimit, RecoveryAllowance};
use super::{ManifestFactsDiscovery, RecoveryCoordination};

mod observation;
mod read_refusal;
pub(super) mod source_memory;
mod wal;

use observation::observe_all;
pub(super) use read_refusal::{
    observation_refused, past_grant, refused_beside, unread, OversizedArtifact,
};
pub(crate) use wal::AdmittedWalInventory;

pub(crate) struct DiscoveryMaterial {
    pub(crate) authority: AdmittedPlatformAuthority,
    pub(crate) coordination: RecoveryCoordination,
    pub(crate) current: PhysicalRootSlotObservation,
    pub(crate) previous: PhysicalRootSlotObservation,
    pub(crate) bootstrap: BootstrapDiscovery,
    pub(crate) current_manifest_facts: ManifestFactsDiscovery,
    pub(crate) previous_manifest_facts: ManifestFactsDiscovery,
    pub(crate) checkpoint: CheckpointDiscovery,
    pub(crate) wal: WalDiscovery,
    pub(crate) residue: Vec<PhysicalRecoveryResidue>,
    pub(crate) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(crate) counters: crate::progression::PhysicalRecoveryDiscoveryCounters,
    pub(crate) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

pub(crate) enum CheckpointDiscovery {
    Absent(worth_store::physical_runtime::ObservedRecoveryArtifact),
    /// `limit` is the one the checkpoint's records ran out of, if any.
    Rejected {
        denial: crate::entry::PhysicalRecoveryCheckpointIntegrityDenial,
        limit: Option<crate::entry::PhysicalRecoveryLimitFailure>,
    },
    Admitted {
        projection: crate::integrity_ingress::OwnerCheckpointProjection,
        source_root: worth_store::physical_runtime::FundedRecoveryObservation,
    },
}

pub(crate) enum BootstrapDiscovery {
    NotRequired,
    Absent,
    Rejected(crate::integrity_ingress::RecoveryIntegrityIngressRejection),
    Admitted(PhysicalBootstrapFallbackAnchor),
}

pub(crate) struct WalDiscovery {
    pub(crate) candidates: crate::orchestration::wal_selection::ResidentWalCandidates,
    pub(crate) rejected: bool,
    pub(super) admitted: AdmittedWalInventory,
    pub(super) integrity_observations: crate::entry::PhysicalRecoveryIntegrityObservations,
    pub(super) integrity_ingress: crate::integrity_ingress::RecoveryIntegrityIngressCounters,
    scanned_frames: u64,
    valid_frames: u64,
    valid_bytes: u64,
    observed_bytes: u64,
    torn_suffix_frames: u64,
    torn_suffix_bytes: u64,
    corruption_denials: u64,
    scanned_segments: u64,
    valid_segments: u64,
    pub(crate) corruptions: Vec<crate::entry::PhysicalRecoveryWalIntegrityDenial>,
}

impl WalDiscovery {
    pub(crate) fn integrity_observations(
        &self,
    ) -> crate::entry::PhysicalRecoveryIntegrityObservations {
        self.integrity_observations.clone()
    }

    pub(crate) fn into_selection_parts(
        self,
    ) -> (
        crate::orchestration::wal_selection::ResidentWalCandidates,
        bool,
        Vec<crate::entry::PhysicalRecoveryWalIntegrityDenial>,
        AdmittedWalInventory,
        crate::entry::PhysicalRecoveryIntegrityObservations,
    ) {
        (
            self.candidates,
            self.rejected,
            self.corruptions,
            self.admitted,
            self.integrity_observations,
        )
    }
}

pub(super) struct DiscoveryFailure {
    cause: PhysicalRecoveryBlockCause,
    source_denials: Vec<PhysicalRecoverySourceDenial>,
    integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    integrity_observations: crate::entry::PhysicalRecoveryIntegrityObservations,
}

impl DiscoveryFailure {
    pub(super) fn with_root_protocol_denials(
        mut self,
        denials: &[PhysicalRecoverySourceDenial],
    ) -> Self {
        let mut combined = denials.to_vec();
        combined.append(&mut self.source_denials);
        self.source_denials = combined;
        self
    }

    pub(super) fn with_integrity_trace(
        mut self,
        trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Self {
        self.integrity_trace.append(trace);
        self
    }

    pub(super) fn with_integrity_observations(
        mut self,
        observations: crate::entry::PhysicalRecoveryIntegrityObservations,
    ) -> Self {
        self.integrity_observations = observations;
        self
    }
}

impl From<PhysicalRecoveryBlock> for DiscoveryFailure {
    fn from(kind: PhysicalRecoveryBlock) -> Self {
        Self::of(PhysicalRecoveryBlockCause::Damage(kind))
    }
}

impl DiscoveryFailure {
    /// `phase` ran out of `limit`.
    pub(super) fn limit(phase: PhysicalRecoveryBlock, limit: ExceededRecoveryLimit) -> Self {
        Self::of(PhysicalRecoveryBlockCause::Limit {
            phase,
            limit: limit.into(),
        })
    }

    /// Why discovery stopped, as a test reads it.
    #[cfg(test)]
    pub(super) const fn cause(&self) -> PhysicalRecoveryBlockCause {
        self.cause
    }

    fn of(cause: PhysicalRecoveryBlockCause) -> Self {
        Self {
            cause,
            source_denials: Vec::new(),
            integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace::new(),
            integrity_observations: crate::entry::PhysicalRecoveryIntegrityObservations::default(),
        }
    }
}

pub(crate) fn discover_sources(
    authority: AdmittedPlatformAuthority,
    mut coordination: RecoveryCoordination,
) -> Result<
    DiscoveryMaterial,
    (
        AdmittedPlatformAuthority,
        RecoveryCoordination,
        PhysicalRecoveryBlockCause,
        PhysicalRecoveryBlockEvidence,
    ),
> {
    let limits = authority.limits;
    let declaration = limits.declaration();
    // Recovery reads both root selectors.
    if let Some(limit) = RecoveryAllowance::declared(
        &declaration,
        PhysicalRecoveryLimitDimension::SelectorCandidates,
    )
    .past(2)
    {
        return Err((
            authority,
            coordination,
            PhysicalRecoveryBlockCause::Limit {
                phase: PhysicalRecoveryBlock::RootProtocol,
                limit,
            },
            PhysicalRecoveryBlockEvidence::default(),
        ));
    }
    let AdmittedPlatformAuthority {
        media,
        session,
        _world_binding,
        limits,
        record_format,
    } = authority;
    let mut discovery = media
        .bounded_discovery(UNCOUNTED_READS, declaration.observation_bytes)
        .expect("a nonzero admitted discovery limit constructs a bounded observer");
    let mut counters = crate::progression::PhysicalRecoveryDiscoveryCounters::default();
    let mut ingress_trace = crate::integrity_ingress::RecoveryIntegrityIngressTrace::new();
    let result = observe_all(
        &mut discovery,
        &mut coordination,
        limits,
        record_format,
        &mut counters,
        &mut ingress_trace,
    );
    counters.bytes_observed = discovery.counters().bytes_read;
    counters.wal_entries = discovery.counters().directory_entries_observed;
    counters.wal_bytes = discovery.counters().wal_bytes_read;
    let media = discovery.finish();
    let authority = AdmittedPlatformAuthority {
        media,
        session,
        _world_binding,
        limits,
        record_format,
    };
    match result {
        Ok(observed) => Ok(DiscoveryMaterial {
            authority,
            coordination,
            current: observed.current,
            previous: observed.previous,
            bootstrap: observed.bootstrap,
            current_manifest_facts: observed.current_manifest_facts,
            previous_manifest_facts: observed.previous_manifest_facts,
            checkpoint: observed.checkpoint,
            wal: observed.wal,
            residue: observed.residue,
            root_protocol_denials: observed.root_protocol_denials,
            counters,
            integrity_trace: ingress_trace,
        }),
        Err(failure) => {
            let DiscoveryFailure {
                cause,
                source_denials,
                mut integrity_trace,
                integrity_observations,
            } = failure;
            integrity_trace.append(ingress_trace);
            Err((
                authority,
                coordination,
                cause,
                PhysicalRecoveryBlockEvidence {
                    counters,
                    artifact: Some(discovery_artifact_context(cause).to_owned()),
                    source_denials,
                    integrity_trace,
                    integrity_observations,
                    ..PhysicalRecoveryBlockEvidence::default()
                },
            ))
        }
    }
}

fn discovery_artifact_context(cause: PhysicalRecoveryBlockCause) -> &'static str {
    let kind = match cause {
        PhysicalRecoveryBlockCause::Limit { .. } => return "bounded recovery-media observation",
        PhysicalRecoveryBlockCause::Damage(kind) => kind,
    };
    match kind {
        PhysicalRecoveryBlock::MediaObservation => "recovery-media artifact",
        PhysicalRecoveryBlock::SourceAllocation => "recovery-media source read allocation",
        PhysicalRecoveryBlock::RootProtocol => "records/root selectors",
        PhysicalRecoveryBlock::Checkpoint => "families/checkpoint.current",
        PhysicalRecoveryBlock::WalInventory => "families/wal",
        PhysicalRecoveryBlock::SourceSelection => "persisted-source cut",
        PhysicalRecoveryBlock::BindingFreshness => "selected checkpoint binding freshness",
        PhysicalRecoveryBlock::PageAdmission => "manifest-addressed page or extent",
        PhysicalRecoveryBlock::OperationReconciliation => "operation-fate evidence",
        PhysicalRecoveryBlock::RedoPlanning => "canonical redo plan",
        PhysicalRecoveryBlock::SelectedCustody => "checkpoint-source-release-head-v2",
        PhysicalRecoveryBlock::Staging => "closed recovery staging generation",
        PhysicalRecoveryBlock::Publication => "recovered-root publication",
    }
}
