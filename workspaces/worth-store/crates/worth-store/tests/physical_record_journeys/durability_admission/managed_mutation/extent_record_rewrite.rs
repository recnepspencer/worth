use std::path::Path;

use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::{
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordId,
    PhysicalRetirementDenial, RecordByteLimit, RecordReadLimits, ServingPhysicalRuntime,
};
use worth_store_physical_backend::MediaOperationRole;

use super::super::independent_wal_oracle::{
    produced_retirement_payloads, produced_rewrite_payloads, IndependentRetiredKind,
    IndependentRetirementAction,
};
use super::*;

/// Larger than one page, so the record lives in a multi-chunk extent.
const EXTENT_PAYLOAD_BYTES: usize = 40_000;

#[test]
fn extent_rewrite_preserves_identity_and_bytes_then_retires_the_source_generation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = extent_payload();
    let appended = completed(prepare(&serving, placement, [61; 32], &payload).execute());
    let identity = appended.persisted_records()[0];
    let record = appended.into_acknowledgment().record_ids().next().unwrap();
    assert!(extent_file(&root, 1).exists());
    assert!(extent_manifest_file(&root, 1).exists());
    let before = serving.certification_charged_growth_bytes();

    let rewritten =
        completed(prepare_extent_rewrite(&serving, placement, [62; 32], record).execute());
    assert_eq!(rewritten.persisted_records(), &[identity]);
    assert_eq!(read_record(&serving, record), payload);
    assert!(extent_file(&root, 2).exists());
    assert!(extent_manifest_file(&root, 2).exists());
    assert!(
        extent_file(&root, 1).exists(),
        "the displaced generation stays until retirement"
    );
    assert!(serving.certification_charged_growth_bytes() > before);
    let rewrites = produced_rewrite_payloads(&root);
    assert_eq!(rewrites.len(), 1);
    assert_eq!(rewrites[0].source_length as usize, EXTENT_PAYLOAD_BYTES);
    assert!(rewrites[0].resulting_root_generation > rewrites[0].source_root_generation);

    serving.retire_displaced_segment().unwrap();
    assert!(!extent_file(&root, 1).exists());
    assert!(!extent_manifest_file(&root, 1).exists());
    let retirements = produced_retirement_payloads(&root);
    assert_eq!(
        retirements
            .iter()
            .map(|record| (
                record.action,
                record.kind,
                record.artifact_id,
                record.generation
            ))
            .collect::<Vec<_>>(),
        [
            (
                IndependentRetirementAction::Intent,
                IndependentRetiredKind::Extent,
                1,
                1
            ),
            (
                IndependentRetirementAction::Completion,
                IndependentRetiredKind::Extent,
                1,
                1
            ),
        ],
        "the WAL must claim exactly the displaced extent generation"
    );
    serving.close();

    let reopened = crate::serving_from_open(&root);
    assert_eq!(read_record(&reopened, record), payload);
    reopened.close();
}

#[test]
fn reopened_store_keeps_the_displaced_extent_charged_and_retirable() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = extent_payload();
    let appended = completed(prepare(&serving, placement, [63; 32], &payload).execute());
    let record = appended.into_acknowledgment().record_ids().next().unwrap();
    completed(prepare_extent_rewrite(&serving, placement, [64; 32], record).execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();

    let reopened = crate::serving_from_open(&root);
    assert_eq!(
        reopened.certification_charged_growth_bytes(),
        charged,
        "reopen must keep the displaced extent generation charged"
    );
    reopened.retire_displaced_segment().unwrap();
    assert!(!extent_file(&root, 1).exists());
    assert!(!extent_manifest_file(&root, 1).exists());
    assert_eq!(read_record(&reopened, record), payload);
    reopened.close();
}

#[test]
fn a_live_reader_protects_the_displaced_extent_generation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = extent_payload();
    let appended = completed(prepare(&serving, placement, [67; 32], &payload).execute());
    let record = appended.into_acknowledgment().record_ids().next().unwrap();
    let source_reader = serving.records().unwrap();
    completed(prepare_extent_rewrite(&serving, placement, [68; 32], record).execute());
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Protected)
    );
    assert!(extent_file(&root, 1).exists());
    assert!(extent_manifest_file(&root, 1).exists());
    drop(source_reader);
    serving.retire_displaced_segment().unwrap();
    assert!(!extent_file(&root, 1).exists());
    assert_eq!(read_record(&serving, record), payload);
    serving.close();
}

#[test]
fn a_published_predecessor_invalidates_a_prepared_extent_rewrite() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let appended = completed(prepare(&serving, placement, [69; 32], &extent_payload()).execute());
    let record = appended.into_acknowledgment().record_ids().next().unwrap();
    let rewrite = prepare_extent_rewrite(&serving, placement, [70; 32], record);
    completed(prepare(&serving, placement, [71; 32], b"published-predecessor").execute());
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    match rewrite.execute() {
        PhysicalMutationOutcome::ProvenNoEffect(fate) => assert_eq!(
            fate.cause(),
            PhysicalMutationProvenNoEffectCause::SourceChanged
        ),
        _ => panic!("a stale extent rewrite must be refused before any effect"),
    }
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    assert!(!extent_file(&root, 2).exists());
    serving.close();
}

#[test]
fn a_twice_rewritten_extent_reopens_charged_for_both_displaced_generations() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = extent_payload();
    let appended = completed(prepare(&serving, placement, [72; 32], &payload).execute());
    let record = appended.into_acknowledgment().record_ids().next().unwrap();
    completed(prepare_extent_rewrite(&serving, placement, [73; 32], record).execute());
    completed(prepare_extent_rewrite(&serving, placement, [74; 32], record).execute());
    assert!(extent_file(&root, 3).exists());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();

    let reopened = crate::serving_from_open(&root);
    assert_eq!(reopened.certification_charged_growth_bytes(), charged);
    reopened.retire_displaced_segment().unwrap();
    reopened.retire_displaced_segment().unwrap();
    for generation in [1, 2] {
        assert!(!extent_file(&root, generation).exists(), "{generation}");
        assert!(
            !extent_manifest_file(&root, generation).exists(),
            "{generation}"
        );
    }
    assert!(extent_manifest_file(&root, 3).exists());
    assert_eq!(read_record(&reopened, record), payload);
    reopened.close();
}

#[test]
fn rewriting_one_of_several_extents_charges_and_retires_only_its_source() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = extent_payload();
    let mut records = Vec::new();
    for material in [75, 76, 77] {
        let appended = completed(prepare(&serving, placement, [material; 32], &payload).execute());
        records.push(appended.into_acknowledgment().record_ids().next().unwrap());
    }
    completed(prepare_extent_rewrite(&serving, placement, [78; 32], records[1]).execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();

    let reopened = crate::serving_from_open(&root);
    assert_eq!(reopened.certification_charged_growth_bytes(), charged);
    reopened.retire_displaced_segment().unwrap();
    assert!(!extent_file_of(&root, 2, 1).exists());
    assert!(extent_file_of(&root, 2, 2).exists());
    for extent in [1, 3] {
        assert!(extent_file_of(&root, extent, 1).exists(), "{extent}");
    }
    for record in records {
        assert_eq!(read_record(&reopened, record), payload);
    }
    reopened.close();
}

#[test]
fn an_inline_record_is_not_an_extent_rewrite_source() {
    let parent = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let appended = completed(prepare(&serving, placement, [65; 32], b"inline").execute());
    let record = appended.into_acknowledgment().record_ids().next().unwrap();
    let key = serving
        .record_submission()
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([66; 32]))
        .unwrap();
    let outcome = serving
        .record_submission()
        .rewrite_selected_extent_record(placement, request(key), record)
        .into_raw();
    assert!(
        !matches!(
            outcome,
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(_))
        ),
        "an inline record must not prepare an extent rewrite"
    );
    serving.close();
}

fn prepare_extent_rewrite(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: [u8; 32],
    record: PhysicalRecordId,
) -> PreparedPhysicalMutation {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    match submission
        .rewrite_selected_extent_record(placement, request(key), record)
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("an extent record rewrite must prepare"),
    }
}

fn request(
    key: worth_store::physical_runtime::PhysicalMutationIdempotencyKey,
) -> PhysicalMutationRequest {
    PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::at(TemporalDuration::temporal_duration(1_000).unwrap()),
    )
}

fn read_record(serving: &ServingPhysicalRuntime, record: PhysicalRecordId) -> Vec<u8> {
    let reader = serving.records().unwrap();
    let mut session = reader
        .open(
            record,
            RecordReadLimits::new(RecordByteLimit::new(EXTENT_PAYLOAD_BYTES as u32).unwrap()),
        )
        .unwrap();
    let mut bytes = Vec::new();
    let mut buffer = vec![0_u8; 4096];
    loop {
        let count = session.read_next(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    bytes
}

fn extent_payload() -> Vec<u8> {
    (0..EXTENT_PAYLOAD_BYTES)
        .map(|index| (index * 31 % 251) as u8)
        .collect()
}

fn extent_file(root: &Path, generation: u64) -> std::path::PathBuf {
    extent_file_of(root, 1, generation)
}

fn extent_file_of(root: &Path, extent: u64, generation: u64) -> std::path::PathBuf {
    root.join(format!(
        "families/records/extents/extent-{extent:016x}-{generation:016x}.data"
    ))
}

fn extent_manifest_file(root: &Path, generation: u64) -> std::path::PathBuf {
    root.join(format!(
        "families/records/extent-manifests/extent-0000000000000001-{generation:016x}.manifest"
    ))
}
