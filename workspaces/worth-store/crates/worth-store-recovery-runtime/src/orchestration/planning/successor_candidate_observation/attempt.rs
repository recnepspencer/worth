use worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration};

use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::progression::PlanningResidentAllowance;
use crate::progression::RecoveryObservedSuccessorCandidate;

use super::materialization::CandidateMaterialization;
use super::{observe_bounded, ManifestEntryBudget};

pub(in crate::orchestration::planning) struct SuccessorCandidateObservationAttempt {
    pub(in crate::orchestration::planning) result: Result<
        Option<RecoveryObservedSuccessorCandidate>,
        PhysicalRecoverySuccessorCandidateDenial,
    >,
    pub(in crate::orchestration::planning) artifact_reads: u64,
    pub(in crate::orchestration::planning) bytes_read: u64,
    pub(in crate::orchestration::planning) peak_materialization_bytes: u64,
    pub(in crate::orchestration::planning) root_protocol_counters:
        crate::entry::PhysicalRecoveryRootProtocolCounters,
}

/// What an attempt with no observation bytes left reports. No reader can
/// be opened on nothing, so this says what one would have: its own budget
/// ran out at the first byte of the candidate root. That is a limit, not an
/// oversized root.
pub(in crate::orchestration::planning) fn out_of_observation_bytes(
    selected_generation: u64,
) -> PhysicalRecoverySuccessorCandidateDenial {
    let generation = selected_generation.saturating_add(1);
    PhysicalRecoverySuccessorCandidateDenial::Discovery {
        artifact: worth_store_physical_format::RecordArtifactFile::RootManifest { generation },
        generation,
        failure: worth_store::physical_runtime::RecoveryDiscoveryFailure::ByteLimitExceeded {
            observed: 1,
            admitted: 0,
            scope: worth_store::physical_runtime::RecoveryDiscoveryByteLimitScope::Observation,
        },
    }
}

pub(in crate::orchestration::planning) fn observe(
    media: AdmittedRecoveryFilesystemMedia,
    selected: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    maximum_bytes: u64,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    allowance: &mut PlanningResidentAllowance,
) -> (
    AdmittedRecoveryFilesystemMedia,
    SuccessorCandidateObservationAttempt,
) {
    if maximum_bytes == 0 {
        return failed(media, out_of_observation_bytes(selected.generation()));
    }
    let mut discovery = media
        .bounded_discovery(
            crate::orchestration::reader_limit::UNCOUNTED_READS,
            maximum_bytes,
        )
        .expect("remaining recovery observation limits are nonzero");
    let mut materialization = CandidateMaterialization::default();
    let mut root_protocol_counters = crate::entry::PhysicalRecoveryRootProtocolCounters::default();
    let result = observe_bounded(
        &mut discovery,
        selected,
        format,
        budget,
        &mut materialization,
        &mut root_protocol_counters,
        integrity_trace,
        allowance,
    );
    let counters = discovery.counters();
    let peak_materialization_bytes = materialization.peak_bytes();
    (
        discovery.finish(),
        SuccessorCandidateObservationAttempt {
            result,
            artifact_reads: counters.addressed_artifacts_read,
            bytes_read: counters.bytes_read,
            peak_materialization_bytes,
            root_protocol_counters,
        },
    )
}

fn failed(
    media: AdmittedRecoveryFilesystemMedia,
    denial: PhysicalRecoverySuccessorCandidateDenial,
) -> (
    AdmittedRecoveryFilesystemMedia,
    SuccessorCandidateObservationAttempt,
) {
    (
        media,
        SuccessorCandidateObservationAttempt {
            result: Err(denial),
            artifact_reads: 0,
            bytes_read: 0,
            peak_materialization_bytes: 0,
            root_protocol_counters: Default::default(),
        },
    )
}
