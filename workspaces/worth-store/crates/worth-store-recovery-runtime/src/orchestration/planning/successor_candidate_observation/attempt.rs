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

/// The candidate after `selected`, observed with the `remaining_bytes`
/// recovery has left. None left refuses nothing yet: an absent candidate
/// costs no bytes, and a present one is refused with its real length.
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
    let mut discovery = media
        .bounded_discovery(
            crate::orchestration::reader_limit::UNCOUNTED_READS,
            remaining_bytes,
        )
        .expect("a reader that counts no reads opens on any byte bound");
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

    /// Finding the candidate absent costs no bytes, so it is found absent
    /// with none left as with many.
    #[test]
    fn an_absent_candidate_is_found_absent_with_no_bytes_left() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let attempt = |remaining| {
            let (_directory, media) = absent_candidate();
            let (media, attempt) = observe(
                media,
                &root_at(6),
                format,
                &mut ManifestEntryBudget::for_test(8, 0),
                remaining,
                &mut Default::default(),
                &mut PlanningResidentAllowance::new(0, 4096).unwrap(),
            );
            drop(media);
            (attempt.result, attempt.artifact_reads, attempt.bytes_read)
        };
        assert_eq!(attempt(4096), (Ok(None), 0, 0));
        assert_eq!(attempt(0), (Ok(None), 0, 0));
    }
}
