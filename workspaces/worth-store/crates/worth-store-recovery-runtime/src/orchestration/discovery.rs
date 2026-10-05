use worth_store::physical_runtime::RecoveryDiscoveryFailure;
use worth_store_recovery_physics::PhysicalBootstrapFallbackAnchor;
use worth_store_recovery_physics::{PhysicalRecoveryResidue, PhysicalRootSlotObservation};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind as PhysicalRecoveryBlock, PhysicalRecoveryLimitDimension,
    PhysicalRecoveryLimitFailure, PhysicalRecoveryMediaObservationFailure,
    PhysicalRecoverySourceDenial,
};

use super::reader_limit::{
    OversizedArtifact, PastCeiling, ReadCeiling, ReaderLimit, UNCOUNTED_READS,
};
use super::{ManifestFactsDiscovery, RecoveryCoordination};

mod observation;
mod wal;

use observation::observe_all;
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
    Rejected(crate::entry::PhysicalRecoveryCheckpointIntegrityDenial),
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
    kind: PhysicalRecoveryBlock,
    limit: Option<PhysicalRecoveryLimitFailure>,
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
        Self {
            kind,
            limit: None,
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
        PhysicalRecoveryBlock,
        PhysicalRecoveryBlockEvidence,
    ),
> {
    let limits = authority.limits;
    let declaration = limits.declaration();
    if declaration.selector_candidates < 2 {
        return Err((
            authority,
            coordination,
            PhysicalRecoveryBlock::DiscoveryLimit,
            limit_evidence(
                PhysicalRecoveryLimitDimension::SelectorCandidates,
                2,
                declaration.selector_candidates,
            ),
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
                kind,
                limit,
                source_denials,
                mut integrity_trace,
                integrity_observations,
            } = failure;
            integrity_trace.append(ingress_trace);
            Err((
                authority,
                coordination,
                kind,
                PhysicalRecoveryBlockEvidence {
                    counters,
                    limit,
                    artifact: Some(discovery_artifact_context(kind).to_owned()),
                    source_denials,
                    integrity_trace,
                    integrity_observations,
                    ..PhysicalRecoveryBlockEvidence::default()
                },
            ))
        }
    }
}

/// A read the reader refused. `Ok` is an artifact larger than its own
/// ceiling: damage, which the caller words for the artifact it read. `Err`
/// blocks discovery. `reader_limit` decides what is a limit, and
/// `budget_dimension` names the caller's budget that narrowed `ceiling`.
pub(super) fn refused_read(
    failure: RecoveryDiscoveryFailure,
    ceiling: ReadCeiling,
    budget_dimension: PhysicalRecoveryLimitDimension,
) -> Result<OversizedArtifact, DiscoveryFailure> {
    let (dimension, observed, admitted) =
        match (ReaderLimit::of(&failure), ceiling.passed(&failure)) {
            // Discovery's reader counts no reads: each count limit counts its
            // own before the read, so the reader's count names none of them.
            (Some(ReaderLimit::Reads { .. }), _) => {
                return Err(DiscoveryFailure::from(
                    PhysicalRecoveryBlock::DiscoveryLimit,
                ))
            }
            (Some(ReaderLimit::ObservationBytes { observed, admitted }), _) => (
                PhysicalRecoveryLimitDimension::ObservationBytes,
                observed,
                admitted,
            ),
            (None, Some(PastCeiling::Budget { observed, admitted })) => {
                (budget_dimension, observed, admitted)
            }
            (None, Some(PastCeiling::Artifact)) => return Ok(OversizedArtifact),
            (None, None) => return Err(media_observation(failure)),
        };
    Err(discovery_limit(dimension, observed, admitted))
}

fn media_observation(failure: RecoveryDiscoveryFailure) -> DiscoveryFailure {
    let mut blocked = DiscoveryFailure::from(PhysicalRecoveryBlock::MediaObservation);
    let (artifact, failure) = match failure {
        RecoveryDiscoveryFailure::Media { artifact, failure } => (
            artifact,
            PhysicalRecoveryMediaObservationFailure::Backend {
                kind: failure.kind(),
                io_kind: failure.io_kind(),
            },
        ),
        RecoveryDiscoveryFailure::InvalidAddress { artifact } => (
            artifact,
            PhysicalRecoveryMediaObservationFailure::InvalidAddress,
        ),
        // `refused_read` decided every count and byte refusal before this.
        RecoveryDiscoveryFailure::EntryLimitExceeded { .. }
        | RecoveryDiscoveryFailure::ByteLimitExceeded { .. } => return blocked,
    };
    blocked
        .source_denials
        .push(PhysicalRecoverySourceDenial::MediaObservation { artifact, failure });
    blocked
}

fn limit_evidence(
    dimension: PhysicalRecoveryLimitDimension,
    observed: u64,
    admitted: u64,
) -> PhysicalRecoveryBlockEvidence {
    PhysicalRecoveryBlockEvidence {
        limit: Some(PhysicalRecoveryLimitFailure {
            dimension,
            observed,
            admitted,
        }),
        ..PhysicalRecoveryBlockEvidence::default()
    }
}

pub(super) fn discovery_limit(
    dimension: PhysicalRecoveryLimitDimension,
    observed: u64,
    admitted: u64,
) -> DiscoveryFailure {
    DiscoveryFailure {
        kind: PhysicalRecoveryBlock::DiscoveryLimit,
        limit: Some(PhysicalRecoveryLimitFailure {
            dimension,
            observed,
            admitted,
        }),
        source_denials: Vec::new(),
        integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace::new(),
        integrity_observations: crate::entry::PhysicalRecoveryIntegrityObservations::default(),
    }
}

fn discovery_artifact_context(kind: PhysicalRecoveryBlock) -> &'static str {
    match kind {
        PhysicalRecoveryBlock::DiscoveryLimit => "bounded recovery-media observation",
        PhysicalRecoveryBlock::MediaObservation => "recovery-media artifact",
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
