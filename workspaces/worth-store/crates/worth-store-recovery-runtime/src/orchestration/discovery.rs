use worth_store::physical_runtime::{FilesystemObservationBound, RecoveryDiscoveryFailure};
use worth_store_recovery_physics::PhysicalBootstrapFallbackAnchor;
use worth_store_recovery_physics::{PhysicalRecoveryResidue, PhysicalRootSlotObservation};

use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlockCause, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind as PhysicalRecoveryBlock, PhysicalRecoveryLimitDeclaration,
    PhysicalRecoveryLimitDimension, PhysicalRecoveryMediaObservationFailure,
    PhysicalRecoverySourceDenial,
};

use super::reader_limit::{OversizedArtifact, PastCeiling, ReadCeiling, UNCOUNTED_READS};
use super::recovery_budget::{ExceededRecoveryLimit, RecoveryAllowance};
use super::{ManifestFactsDiscovery, RecoveryCoordination};

mod observation;
pub(super) mod source_memory;
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

/// A read the reader refused. `Ok` is an artifact larger than its own
/// ceiling: damage, which the caller words for the artifact it read. `Err`
/// blocks discovery. `budget_dimension` names the caller's budget that
/// narrowed `ceiling`. Discovery's reader was handed all of recovery's
/// observation bytes, so its counts are recovery's.
pub(super) fn refused_read(
    failure: RecoveryDiscoveryFailure,
    ceiling: ReadCeiling,
    limits: &PhysicalRecoveryLimitDeclaration,
    budget_dimension: PhysicalRecoveryLimitDimension,
) -> Result<OversizedArtifact, DiscoveryFailure> {
    let past = match failure {
        RecoveryDiscoveryFailure::Limit(past) => past,
        RecoveryDiscoveryFailure::Media { artifact, failure } => {
            return Err(media_observation(
                artifact,
                PhysicalRecoveryMediaObservationFailure::Backend {
                    kind: failure.kind(),
                    io_kind: failure.io_kind(),
                },
            ))
        }
        RecoveryDiscoveryFailure::InvalidAddress { artifact } => {
            return Err(media_observation(
                artifact,
                PhysicalRecoveryMediaObservationFailure::InvalidAddress,
            ))
        }
        // A count past every count is no limit and names no artifact.
        RecoveryDiscoveryFailure::CountOverflow(_) => {
            return Err(DiscoveryFailure::from(
                PhysicalRecoveryBlock::MediaObservation,
            ))
        }
    };
    let (dimension, observed, admitted) = match past.dimension() {
        FilesystemObservationBound::ObservationBytes => (
            PhysicalRecoveryLimitDimension::ObservationBytes,
            past.observed(),
            past.admitted(),
        ),
        FilesystemObservationBound::RequestedBytes => {
            match ceiling.passed(&RecoveryDiscoveryFailure::Limit(past)) {
                Some(PastCeiling::Budget { observed, admitted }) => {
                    (budget_dimension, observed, admitted)
                }
                Some(PastCeiling::Artifact) | None => return Ok(OversizedArtifact),
            }
        }
        // Discovery's main reader counts no reads or entries: each count
        // limit counts its own before the read, so a refused count is past
        // every count, and no limit can state it. A WAL listing counts its
        // entries, and its caller reads them as WAL segments first.
        FilesystemObservationBound::Reads | FilesystemObservationBound::Entries => {
            return Err(DiscoveryFailure::from(
                PhysicalRecoveryBlock::MediaObservation,
            ));
        }
    };
    Err(refused_beside(
        RecoveryAllowance::declared(limits, dimension),
        observed,
        admitted,
        PhysicalRecoveryBlock::MediaObservation,
    ))
}

/// `observed` refused by the `remaining` a caller was handed of the `whole`
/// allowance: the rest was spent before it. `phase` ran out of the whole;
/// counts that do not cross it name no limit, and `phase` failed.
pub(super) fn refused_beside(
    whole: RecoveryAllowance,
    observed: u64,
    remaining: u64,
    phase: PhysicalRecoveryBlock,
) -> DiscoveryFailure {
    whole.beside(observed, remaining).map_or_else(
        || DiscoveryFailure::from(phase),
        |limit| DiscoveryFailure::limit(phase, limit),
    )
}

/// A read that failed for the media, not for any count.
fn media_observation(
    artifact: worth_store::physical_runtime::RecoveryDiscoveryArtifact,
    failure: PhysicalRecoveryMediaObservationFailure,
) -> DiscoveryFailure {
    let mut blocked = DiscoveryFailure::from(PhysicalRecoveryBlock::MediaObservation);
    blocked
        .source_denials
        .push(PhysicalRecoverySourceDenial::MediaObservation { artifact, failure });
    blocked
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
