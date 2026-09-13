use worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration};

use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
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

pub(in crate::orchestration::planning) fn observe(
    media: AdmittedRecoveryFilesystemMedia,
    selected: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    maximum_manifest_entries: u64,
    maximum_bytes: u64,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> (
    AdmittedRecoveryFilesystemMedia,
    SuccessorCandidateObservationAttempt,
) {
    let Some(maximum_entries) = budget.remaining().checked_add(2) else {
        return failed(
            media,
            PhysicalRecoverySuccessorCandidateDenial::ManifestEntryLimit {
                artifact: worth_store_physical_format::RecordArtifactFile::RootManifest {
                    generation: selected.generation().saturating_add(1),
                },
                generation: selected.generation().saturating_add(1),
                observed: maximum_manifest_entries.saturating_add(1),
                admitted: maximum_manifest_entries,
            },
        );
    };
    if maximum_bytes == 0 {
        let artifact = worth_store_physical_format::RecordArtifactFile::RootManifest {
            generation: selected.generation().saturating_add(1),
        };
        return failed(media, PhysicalRecoverySuccessorCandidateDenial::Discovery {
            artifact,
            generation: selected.generation().saturating_add(1),
            failure: worth_store::physical_runtime::RecoveryDiscoveryFailure::ByteLimitExceeded {
                observed: 1,
                admitted: 0,
                scope: worth_store::physical_runtime::RecoveryDiscoveryByteLimitScope::Requested,
            },
        });
    }
    let mut discovery = media
        .bounded_discovery(maximum_entries, maximum_bytes)
        .expect("remaining recovery observation limits are nonzero");
    let mut materialization = CandidateMaterialization::default();
    let mut root_protocol_counters = crate::entry::PhysicalRecoveryRootProtocolCounters::default();
    let result = observe_bounded(
        &mut discovery,
        selected,
        format,
        budget,
        maximum_bytes,
        &mut materialization,
        &mut root_protocol_counters,
        integrity_trace,
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
