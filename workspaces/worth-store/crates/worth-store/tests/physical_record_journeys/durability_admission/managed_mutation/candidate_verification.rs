use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalMutationIndeterminateStage, PhysicalMutationOutcome,
    PhysicalRecordId, PhysicalRecordOpen, PhysicalRuntimeAdmission, PhysicalStore, RecordByteLimit,
    RecordReadDenial, RecordReadLimits, ServingPhysicalRuntime,
};
use worth_store_physical_backend::{
    FilesystemAccessPosture, MediaFaultDirective, MediaOperationRole, MediaPauseGate,
};

use super::selected_segment_rewrite::prepare_rewrite;
use super::*;

const PAYLOAD: &[u8] = b"candidate-verified-before-publication";
/// The candidate byte the interposer flips: the middle of its only page, so the
/// damage lands inside the page body rather than on a file boundary.
fn candidate_damage_offset() -> usize {
    configuration().0.declaration().page_size().bytes() as usize / 2
}

#[test]
fn a_candidate_corrupted_before_publication_is_never_published() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let record = published_source(&root);
    let source_segments = segment_files(&root);

    let serving = crate::serving_from_open(&root);
    // The sealed WAL group record is the rewrite's first positioned write;
    // the candidate page is the second.
    let candidate_write = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite)
        + 2;
    serving.close();

    let (serving, gate) = serving_paused_after_write(&root, candidate_write);
    let before = root_generation(&serving);
    let (_, placement, _) = configuration();
    let handle = prepare_rewrite(&serving, placement, [64; 32]).start();
    gate.wait_until_reached();
    let page_bytes = configuration().0.declaration().page_size().bytes();
    assert_eq!(
        gate.reached_context().unwrap().requested_bytes(),
        u64::from(page_bytes),
        "the paused write is the candidate page"
    );
    let candidate = segment_files(&root)
        .difference(&source_segments)
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(candidate.len(), 1, "the paused write created one candidate");
    let mut bytes = std::fs::read(&candidate[0]).unwrap();
    bytes[candidate_damage_offset()] ^= 0xFF;
    std::fs::write(&candidate[0], &bytes).unwrap();
    gate.release();

    match handle.wait() {
        PhysicalMutationOutcome::Indeterminate(fate) => assert_eq!(
            fate.stage(),
            PhysicalMutationIndeterminateStage::DataDispatch
        ),
        PhysicalMutationOutcome::Completed(_) => panic!("an unchecked candidate was published"),
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("the sealed rewrite claimed no effect: {:?}", fate.cause())
        }
    }
    assert_eq!(
        root_generation(&serving),
        before,
        "no root names the damaged candidate"
    );
    assert_eq!(
        read_record(&serving, record),
        PAYLOAD,
        "the unchanged root keeps serving the intact source"
    );
    serving.close();

    // The durable rewrite redo without a verified candidate is recovery-owner
    // work: a fresh serving open fails closed instead of guessing its fate.
    let reopened = crate::serving_from_open(&root);
    assert_eq!(
        reopened
            .records()
            .unwrap()
            .open(
                record,
                RecordReadLimits::new(RecordByteLimit::new(PAYLOAD.len() as u32).unwrap()),
            )
            .err()
            .map(|error| error.denial()),
        Some(RecordReadDenial::ServingRequiresInspection)
    );
    reopened.close();
}

fn published_source(root: &Path) -> PhysicalRecordId {
    let serving = serving_from_initialization(root);
    let (_, placement, _) = configuration();
    let record = completed(prepare(&serving, placement, [63; 32], PAYLOAD).execute())
        .into_acknowledgment()
        .record_ids()
        .next()
        .unwrap();
    serving.close();
    record
}

fn serving_paused_after_write(
    root: &Path,
    ordinal: u64,
) -> (ServingPhysicalRuntime, MediaPauseGate) {
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            MediaOperationRole::PositionedWrite,
            ordinal,
            MediaFaultDirective::PauseAfter(gate.clone()),
        )])
        .unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("paused media should admit"),
    };
    let (format, _, access) = configuration();
    let serving = crate::success(open_record_store!(media, |durability| {
        PhysicalRecordOpen::new(format, access, durability)
    }));
    (serving, gate)
}

fn segment_files(root: &Path) -> BTreeSet<PathBuf> {
    std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect()
}

fn root_generation(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation()
}

fn read_record(serving: &ServingPhysicalRuntime, record: PhysicalRecordId) -> Vec<u8> {
    let reader = serving.records().unwrap();
    let mut session = reader
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(PAYLOAD.len() as u32).unwrap()),
        )
        .unwrap();
    let mut bytes = vec![0_u8; PAYLOAD.len()];
    let mut filled = 0;
    while filled < bytes.len() {
        let count = session.read_next(&mut bytes[filled..]).unwrap();
        assert!(count > 0);
        filled += count;
    }
    bytes
}
