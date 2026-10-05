use std::num::NonZeroU64;

use worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};

use crate::entry::PhysicalRecoverySuccessorCandidateDenial;
use crate::progression::PlanningResidentAllowance;
use crate::progression::RecoveryObservedSuccessorCandidate;

use super::materialization::CandidateMaterialization;
use super::root_manifest::successor_generation;
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

/// The attempt recovery makes with no observation bytes left. No reader
/// opens on nothing, so none read or counted anything: the candidate root
/// it would have read is the artifact recovery could not observe.
fn out_of_observation_bytes(
    media: AdmittedRecoveryFilesystemMedia,
    selected: &DurablePhysicalRootManifest,
) -> (
    AdmittedRecoveryFilesystemMedia,
    SuccessorCandidateObservationAttempt,
) {
    (
        media,
        SuccessorCandidateObservationAttempt {
            result: Err(exhausted(selected)),
            artifact_reads: 0,
            bytes_read: 0,
            peak_materialization_bytes: 0,
            root_protocol_counters: Default::default(),
        },
    )
}

/// The successor root a reader with bytes would have read first, or the
/// denial that reader would have met before reading anything.
fn exhausted(selected: &DurablePhysicalRootManifest) -> PhysicalRecoverySuccessorCandidateDenial {
    match successor_generation(selected) {
        Ok(generation) => PhysicalRecoverySuccessorCandidateDenial::ObservationBytesExhausted {
            artifact: RecordArtifactFile::RootManifest { generation },
            generation,
        },
        Err(denial) => denial,
    }
}

/// The candidate after `selected`, observed with the `remaining_bytes`
/// recovery has left. With none left no reader opens, not even to find the
/// candidate absent.
pub(in crate::orchestration::planning) fn observe(
    media: AdmittedRecoveryFilesystemMedia,
    selected: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    remaining_bytes: u64,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    allowance: &mut PlanningResidentAllowance,
) -> (
    AdmittedRecoveryFilesystemMedia,
    SuccessorCandidateObservationAttempt,
) {
    let Some(maximum_bytes) = NonZeroU64::new(remaining_bytes) else {
        return out_of_observation_bytes(media, selected);
    };
    let mut discovery = media
        .bounded_discovery(
            crate::orchestration::reader_limit::UNCOUNTED_READS,
            maximum_bytes.get(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::{ManifestBlockReference, PersistedRecordIdentity};

    fn root_at(generation: u64) -> DurablePhysicalRootManifest {
        let record = PersistedRecordIdentity::new([1; 16], 1).unwrap();
        let leaf = ManifestBlockReference::new(1, 1, 0, 99, record, record);
        DurablePhysicalRootManifest::builder(generation, 7, 4, 19)
            .record_count(1)
            .next_block(2)
            .routing_root(leaf)
            .admit()
            .unwrap()
    }

    #[test]
    fn an_attempt_with_no_bytes_left_names_the_successor_root_it_could_not_read() {
        assert_eq!(
            exhausted(&root_at(8)),
            PhysicalRecoverySuccessorCandidateDenial::ObservationBytesExhausted {
                artifact: RecordArtifactFile::RootManifest { generation: 9 },
                generation: 9,
            }
        );
        // The last generation has no successor, with bytes left or without.
        assert_eq!(
            exhausted(&root_at(u64::MAX)),
            super::super::denial::invalid(RecordArtifactFile::RootManifest {
                generation: u64::MAX,
            })
        );
    }

    /// A store with no root after `root_at(6)`: an absent candidate.
    fn absent_candidate() -> (tempfile::TempDir, AdmittedRecoveryFilesystemMedia) {
        use worth_proof::TransitionOutcome;
        use worth_store::physical_runtime::{
            FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission,
            PhysicalStore, QualifiedRecoveryFilesystemMedia,
        };
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("store");
        let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
        let TransitionOutcome::Success(media) = runtime
            .try_admit_filesystem_media(FilesystemMediaAdmission::production(
                FilesystemAccessPosture::CoordinatedServiceAccount,
            ))
            .into_raw()
        else {
            panic!("production media admission");
        };
        media.close();
        let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&root)
            .unwrap()
            .admit_persisted_store()
            .unwrap();
        (directory, media)
    }

    /// Finding the candidate absent costs no bytes, yet with none left no
    /// reader opens to find it: the attempt is out of observation bytes.
    #[test]
    fn with_no_bytes_left_no_reader_opens_even_for_an_absent_candidate() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let attempt = |remaining| {
            let (_directory, media) = absent_candidate();
            let (media, attempt) = observe(
                media,
                &root_at(6),
                format,
                &mut ManifestEntryBudget::new(8, 0),
                remaining,
                &mut Default::default(),
                &mut PlanningResidentAllowance::new(0, 4096).unwrap(),
            );
            drop(media);
            (attempt.result, attempt.artifact_reads, attempt.bytes_read)
        };
        assert_eq!(attempt(4096), (Ok(None), 0, 0));
        assert_eq!(attempt(0), (Err(exhausted(&root_at(6))), 0, 0));
    }
}
