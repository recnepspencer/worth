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
use crate::manifest_fixture::{current_extent_route, IndependentExtentRoute};
/// Larger than one page, so the record lives in a multi-chunk extent.
const EXTENT_PAYLOAD_BYTES: usize = 40_000;
#[path = "arena_copy_crash.rs"]
mod arena_copy_crash;
#[path = "arena_file_scale.rs"]
mod arena_file_scale;
#[path = "arena_history_post_prune.rs"]
mod arena_history_post_prune;
#[path = "arena_retirement.rs"]
mod arena_retirement;
#[path = "arena_retirement_crash.rs"]
mod arena_retirement_crash;
#[path = "extent_copy_failure.rs"]
mod copy_failure;
#[path = "extent_release_crash.rs"]
mod crash;
#[path = "arena_evacuation.rs"]
mod evacuation;
#[path = "extent_frame_fixture.rs"]
mod extent_frame_fixture;
#[path = "independent_arena_observer.rs"]
mod independent_arena_observer;
#[path = "reused_extent_frame.rs"]
mod reused_extent_frame;
use extent_frame_fixture::{arena_file, arena_range_bytes, extent_payload};
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
    let source = current_extent_route(&root, identity);
    let before = serving.certification_charged_growth_bytes();

    let rewritten =
        completed(prepare_extent_rewrite(&serving, placement, [62; 32], record).execute());
    assert_eq!(rewritten.persisted_records(), &[identity]);
    assert_eq!(read_record(&serving, record), payload);
    let destination = current_extent_route(&root, identity);
    assert_eq!(
        source.arena, destination.arena,
        "successive extents pack one arena"
    );
    assert_eq!(source.extent, destination.extent);
    assert_eq!(destination.generation, source.generation + 1);
    assert!(source.offset + source.length <= destination.offset);
    let retained_source = arena_range_bytes(&root, source);
    assert!(serving.certification_charged_growth_bytes() > before);
    let rewrites = produced_rewrite_payloads(&root);
    assert_eq!(rewrites.len(), 1);
    assert_eq!(rewrites[0].source_length as usize, EXTENT_PAYLOAD_BYTES);
    assert!(rewrites[0].resulting_root_generation > rewrites[0].source_root_generation);
    let (redo_source, redo_destination, alignment) = rewrites[0].arena_routes.unwrap();
    assert_eq!(redo_source, [source.arena, source.offset, source.length]);
    assert_eq!(
        redo_destination,
        [destination.arena, destination.offset, destination.length]
    );
    assert_eq!(source.offset % alignment, 0);
    assert_eq!(destination.offset % alignment, 0);

    serving.retire_displaced_segment().unwrap();
    assert_eq!(
        arena_range_bytes(&root, source),
        retained_source,
        "range retirement must not delete or overwrite the shared arena"
    );
    let replacement_payload = vec![137; EXTENT_PAYLOAD_BYTES];
    let replacement =
        completed(prepare(&serving, placement, [79; 32], &replacement_payload).execute());
    let replacement_identity = replacement.persisted_records()[0];
    let replacement_record = replacement
        .into_acknowledgment()
        .record_ids()
        .next()
        .unwrap();
    let reused = current_extent_route(&root, replacement_identity);
    assert_eq!(
        (reused.arena, reused.offset, reused.length),
        (source.arena, source.offset, source.length)
    );
    assert_ne!(reused.extent, source.extent);
    assert_eq!(
        read_record(&serving, replacement_record),
        replacement_payload
    );
    assert_eq!(read_record(&serving, record), payload);
    let retirements = produced_retirement_payloads(&root);
    for retirement in &retirements {
        assert_eq!(
            retirement.arena_range,
            Some([source.arena, source.offset, source.length])
        );
        let roots = retirement.release_roots.unwrap();
        let bytes = std::fs::read(root.join(format!(
            "families/records/roots/root-{:016x}.manifest",
            roots[1]
        )))
        .unwrap();
        use sha2::Digest;
        let digest: [u8; 32] = sha2::Sha256::digest(bytes).into();
        assert_eq!(retirement.release_digest, Some(digest));
        assert!(retirement.metadata_bytes > 0);
    }
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
    let charged = serving.certification_charged_growth_bytes();
    serving.close();

    let reopened = crate::serving_from_open(&root);
    assert_eq!(
        reopened.certification_charged_growth_bytes(),
        charged,
        "ordinary and exact release-root charges remain identical after reopen"
    );
    assert_eq!(read_record(&reopened, record), payload);
    assert_eq!(
        read_record(&reopened, replacement_record),
        replacement_payload
    );
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
    let identity = appended.persisted_records()[0];
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
    assert!(arena_file(&root, current_extent_route(&root, identity).arena).exists());
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
    let identity = appended.persisted_records()[0];
    let source = current_extent_route(&root, identity);
    let record = appended.into_acknowledgment().record_ids().next().unwrap();
    let source_reader = serving.records().unwrap();
    completed(prepare_extent_rewrite(&serving, placement, [68; 32], record).execute());
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Protected)
    );
    assert!(produced_retirement_payloads(&root).is_empty());
    assert_eq!(
        read_from_reader(&source_reader, record),
        payload,
        "the pinned predecessor remains readable after the route moves"
    );
    let retained_source = arena_range_bytes(&root, source);
    drop(source_reader);
    serving.retire_displaced_segment().unwrap();
    assert_eq!(arena_range_bytes(&root, source), retained_source);
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
    let identity = appended.persisted_records()[0];
    let source = current_extent_route(&root, identity);
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
    assert_eq!(current_extent_route(&root, identity), source);
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
    let identity = appended.persisted_records()[0];
    let record = appended.into_acknowledgment().record_ids().next().unwrap();
    completed(prepare_extent_rewrite(&serving, placement, [73; 32], record).execute());
    completed(prepare_extent_rewrite(&serving, placement, [74; 32], record).execute());
    let destination = current_extent_route(&root, identity);
    assert_eq!(destination.generation, 3);
    let charged = serving.certification_charged_growth_bytes();
    serving.close();

    let reopened = crate::serving_from_open(&root);
    assert_eq!(reopened.certification_charged_growth_bytes(), charged);
    reopened.retire_displaced_segment().unwrap();
    reopened.retire_displaced_segment().unwrap();
    assert_eq!(current_extent_route(&root, identity), destination);
    assert!(arena_file(&root, destination.arena).exists());
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
    let mut identities = Vec::new();
    let mut routes = Vec::new();
    for material in [75, 76, 77] {
        let appended = completed(prepare(&serving, placement, [material; 32], &payload).execute());
        identities.push(appended.persisted_records()[0]);
        routes.push(current_extent_route(&root, *identities.last().unwrap()));
        records.push(appended.into_acknowledgment().record_ids().next().unwrap());
    }
    completed(prepare_extent_rewrite(&serving, placement, [78; 32], records[1]).execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();

    let reopened = crate::serving_from_open(&root);
    assert_eq!(reopened.certification_charged_growth_bytes(), charged);
    reopened.retire_displaced_segment().unwrap();
    assert_eq!(current_extent_route(&root, identities[1]).generation, 2);
    for index in [0, 2] {
        assert_eq!(
            current_extent_route(&root, identities[index]),
            routes[index]
        );
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
    read_from_reader(&reader, record)
}

fn read_from_reader(
    reader: &worth_store::physical_runtime::PhysicalRecordReader,
    record: PhysicalRecordId,
) -> Vec<u8> {
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
