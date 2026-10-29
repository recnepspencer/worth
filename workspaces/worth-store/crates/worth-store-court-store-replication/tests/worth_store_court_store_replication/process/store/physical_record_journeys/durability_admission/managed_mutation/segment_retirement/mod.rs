use {worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::SegmentPageCount};
use {worth_store_physical_backend::MediaOperationRole};

use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction};
use {worth_store_testkit_store::durability_admission::managed_mutation::published_segments::segment_identity, worth_store_testkit_store::durability_admission::managed_mutation::published_segments::segment_names};
use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite};
use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendBatch, worth_signal::facade::TemporalDuration, worth_proof::TransitionOutcome, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare_records, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
use {worth_store_testkit_store::physical_record_journeys::manifest_fixture::current_extent_route};

#[test]
fn retirement_waits_for_the_source_reader_then_releases_charge() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (format, placement, _) = configuration();
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    completed(prepare(&serving, placement, [51; 32], b"retirement-source").execute());
    let first_reader = serving.records().unwrap();
    let second_reader = serving.records().unwrap();
    let appended = serving.certification_charged_growth_bytes();
    completed(prepare_rewrite(&serving, placement, [52; 32]).execute());
    let charged = serving.certification_charged_growth_bytes();
    // The settled prior rewrite is a lower bound on a fresh admission; a
    // half-page deficit proves pressure while the displaced source is held.
    // The later rewrite is tested under a fresh finite cap because its WAL
    // publication reserves a conservative routing ceiling before settlement.
    let rewrite_cost = charged - appended;
    serving.certification_limit_candidate_growth_bytes(charged + rewrite_cost - page_bytes / 2);
    let successor_reader = serving.records().unwrap();
    let before = segment_names(&root);
    assert!(before.len() >= 2);
    match prepare_rewrite(&serving, placement, [53; 32]).execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            assert_eq!(
                fate.cause(),
                worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::RetentionPressure
            );
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) | worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("retained garbage must deny the next rewrite at the growth limit")
        }
    }
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected)
    );
    assert_eq!(segment_names(&root), before);
    drop(first_reader);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected)
    );
    assert_eq!(segment_names(&root), before);
    drop(second_reader);
    let before_retirement_charge = serving.certification_charged_growth_bytes();
    serving.retire_displaced_segment().unwrap();
    drop(successor_reader);
    let after_retirement_charge = serving.certification_charged_growth_bytes();
    assert!(
        after_retirement_charge < before_retirement_charge,
        "the released page must exceed the newly retained maintenance WAL charge"
    );
    let after = segment_names(&root);
    assert_eq!(after.len(), before.len() - 1);
    let retired = before
        .iter()
        .find(|name| !after.contains(name))
        .map(|name| segment_identity(name))
        .unwrap();
    let records = produced_retirement_payloads(&root);
    assert_eq!(
        records
            .iter()
            .map(|record| (record.action, record.artifact_id, record.generation))
            .collect::<Vec<_>>(),
        [
            (worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent, retired.0, retired.1),
            (
                worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Completion,
                retired.0,
                retired.1
            ),
        ],
        "the WAL must carry exactly one intent and one completion for the retired generation"
    );
    assert!(records.iter().all(|record| record.bytes == page_bytes));
    // This is a new bounded allowance, not a claim that a fresh publication
    // has the same peak reservation as the already-settled prior rewrite.
    serving.certification_limit_candidate_growth_bytes(after_retirement_charge + 1024 * 1024);
    completed(prepare_rewrite(&serving, placement, [54; 32]).execute());
    assert!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
            > 0
    );
    serving.close();
    let reopened = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert!(reopened.records().is_ok());
    reopened.close();
}

#[test]
fn reopened_store_keeps_displaced_rewrite_garbage() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [45; 32], b"displaced-source").execute());
    completed(prepare_rewrite(&serving, placement, [46; 32]).execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must keep the displaced source page charged"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_displaced_garbage_after_a_later_append() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    completed(prepare(&serving, placement, [47; 32], b"displaced-then-append").execute());
    completed(prepare_rewrite(&serving, placement, [48; 32]).execute());
    completed(prepare(&serving, placement, [49; 32], b"after-rewrite").execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "a later append must not drop the displaced source charge"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_each_publication_routing_bound() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    for ordinal in 0_u8..=64 {
        let mut material = [76_u8; 32];
        material[0] = ordinal;
        completed(prepare(&serving, placement, material, b"height-step").execute());
    }
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must price each publication at the routing bound it sealed"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_a_rewrite_routing_bound_at_the_height_boundary() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = configuration();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(&root);
    let durability = worth_store_testkit_store::physical_record_journeys::durability(&media);
    let serving = worth_store_testkit_store::physical_record_journeys::success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        ),
    ));
    for ordinal in 0_u8..2 {
        let mut material = [77_u8; 32];
        material[0] = ordinal;
        completed(prepare(&serving, placement, material, b"rewrite-bound").execute());
    }
    completed(prepare_rewrite(&serving, placement, [78; 32]).execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "a rewrite must not raise routing height when it inserts no records"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_a_capacity_transition_routing_bound() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = configuration();
    let wide = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(4).unwrap())
        .admit(format)
        .unwrap();
    let narrow = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(&root);
    let durability = worth_store_testkit_store::physical_record_journeys::durability(&media);
    let serving = worth_store_testkit_store::physical_record_journeys::success(media.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, wide, access, durability,
        ),
    ));
    let mut records = Vec::new();
    for ordinal in 0_u8..8 {
        records.push(vec![ordinal, 9, 9, 9]);
    }
    let batch: Vec<&[u8]> = records.iter().map(Vec::as_slice).collect();
    completed(prepare_records(&serving, wide, [79; 32], &batch).execute());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([80; 32]),
        )
        .unwrap();
    let prepared = match submission
        .prepare_durable_append_with_manifest_capacity_transition(
            worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"narrow" as &[u8]])
                .unwrap(),
            narrow,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::ReconstructToRequested,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::at(
                    worth_signal::facade::TemporalDuration::temporal_duration(1_000).unwrap(),
                ),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        _ => panic!("capacity reconstruction must prepare"),
    };
    completed(prepared.execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must price the successor manifest capacity"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_every_segment_from_one_batch() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (format, _, _) = configuration();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(1).unwrap())
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let payload = vec![9_u8; 7_500];
    completed(
        prepare_records(
            &serving,
            placement,
            [75; 32],
            &[&payload, &payload, &payload],
        )
        .execute(),
    );
    let segments = std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .flatten()
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("pages"))
        .count();
    assert_eq!(
        segments, 3,
        "one page per segment must publish three segments"
    );
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must charge every segment the batch published"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_a_multi_page_rewrite_charge() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = vec![9_u8; 7_500];
    completed(prepare_records(&serving, placement, [71; 32], &[&payload, &payload]).execute());
    completed(prepare_rewrite(&serving, placement, [72; 32]).execute());
    let charged = serving.certification_charged_growth_bytes();
    serving.close();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must keep every page the multi-page append and rewrite sealed"
    );
    serving.close();
}

#[test]
fn reopened_store_keeps_an_extent_published_after_a_denied_reservation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    serving.certification_limit_candidate_growth_bytes(80_000);
    let oversized = vec![3_u8; 100_000];
    match prepare(&serving, placement, [73; 32], &oversized).execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(_) => {}
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {
            panic!("the oversized extent must be denied before it publishes")
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(_) => {
            panic!("growth denial must prove no effect")
        }
    }
    let arena_dir = root.join("families/records/arenas");
    assert!(
        !arena_dir.exists() || std::fs::read_dir(&arena_dir).unwrap().next().is_none(),
        "the denied reservation must not publish an arena"
    );
    let published = vec![7_u8; 8_192];
    let publication = completed(prepare(&serving, placement, [74; 32], &published).execute());
    let charged = serving.certification_charged_growth_bytes();
    let route = current_extent_route(&root, publication.persisted_records()[0]);
    let published_arena = arena_dir.join(format!("arena-{:016x}.data", route.arena));
    assert!(
        published_arena.is_file(),
        "the later append must publish a routed arena range"
    );
    serving.close();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert_eq!(serving.certification_charged_growth_bytes(), charged);
    serving.close();
}

#[test]
fn reopened_store_keeps_the_published_extent_charge() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (format, placement, _) = configuration();
    let payload =
        vec![7_u8; usize::try_from(format.declaration().page_size().bytes() / 2).unwrap()];
    let publication = completed(prepare(&serving, placement, [70; 32], &payload).execute());
    let charged = serving.certification_charged_growth_bytes();
    let route = current_extent_route(&root, publication.persisted_records()[0]);
    let arena = root.join(format!(
        "families/records/arenas/arena-{:016x}.data",
        route.arena
    ));
    assert!(arena.is_file(), "the payload must publish an arena range");
    serving.close();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(&root);
    assert_eq!(
        serving.certification_charged_growth_bytes(),
        charged,
        "reopen must keep the published extent bytes charged"
    );
    serving.close();
}
