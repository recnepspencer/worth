//! Current metadata is authenticated and sufficient for an append, not history.
use super::*;
use crate::storage::residency::region_image::{
    RegionHeadImage, RegionHistoryImage, RelationalRegionImage, RelationalRegionImageKind,
};

#[test]
fn head_contains_current_metadata_and_history_contains_only_earlier_versions() {
    let fixture = region_with_history();
    let head = decode_head(&fixture);
    let head_rows = &head.partition.entity_arena.metadata_history[0];
    assert_eq!(
        head_rows.len(),
        1,
        "one current metadata version belongs in Head"
    );
    let bytes = fixture.source.members.get(&fixture.cold.history).unwrap();
    let frame = RelationalRegionImage::decode(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &mut |_, _| Ok(()),
        bytes,
        fixture.cold.history,
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
    )
    .unwrap();
    let history: RegionHistoryImage = rmp_serde::from_slice(frame.payload).unwrap();
    assert_eq!(
        history.entities[0].earlier.len() + head_rows.len(),
        fixture.expected_history
    );
    assert!(history.entities[0].earlier.last().unwrap().effective_at < head_rows[0].effective_at);
    assert!(
        !history.relations[0].earlier.is_empty(),
        "fixture has real earlier endpoint versions"
    );
    assert_eq!(
        fixture.source.reads.load(Ordering::SeqCst),
        0,
        "inspect producer bytes without warming the receiver"
    );
}

#[test]
fn a_head_append_matches_a_separately_recomputed_full_history_root() {
    let fixture = region_with_history();
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let head = region_load::load_head(
        &fixture.cold,
        fixture.cold.source.as_ref().unwrap(),
        execution,
        1 << 24,
    )
    .unwrap();
    let symbols = fixture.cold.symbols.interner_snapshot();
    let slot = fixture.entity.slot_index();
    let mut written_head = head.partition.clone();
    let extra = written_head
        .entity_arena
        .get_slot(slot)
        .unwrap()
        .extra()
        .clone();
    let version = crate::identity::data::VersionId(1_000);
    written_head
        .entity_arena
        .apply_extra_update(slot, extra.clone(), version)
        .expect("fixture slot is materialized");
    let physical = written_head.entity_arena.physical_index(slot).unwrap();
    let baseline = &head.partition.entity_arena.history_prefixes[physical]
        .as_ref()
        .unwrap()
        .history_opening;
    let carried = &written_head.entity_arena.history_prefixes[physical]
        .as_ref()
        .unwrap()
        .history_opening;
    assert_eq!(
        baseline.id(),
        carried.id(),
        "appending keeps the prefix's original opening"
    );
    let mut journal = crate::storage::overlay::PartitionMutationJournal::default();
    journal.entity_slots.insert(slot);
    let (appended, _) = PartitionContentCommitment::maintain(
        &written_head,
        &symbols,
        (head.commitment.as_ref().unwrap(), &head.partition, &journal),
    )
    .unwrap();
    let (advanced, _) = appended
        .history_frontiers(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            execution,
            &written_head,
        )
        .unwrap()
        .into_arenas();
    let (_, advanced) = advanced
        .into_iter()
        .find(|(logical, _)| *logical == slot as u64)
        .unwrap();
    assert_ne!(
        &carried.frontier, &advanced,
        "the current commitment advances independently of its cold baseline"
    );
    assert_eq!(
        written_head
            .entity_arena
            .metadata_history_at(slot)
            .unwrap()
            .len(),
        2,
        "only the old current row and the appended row are resident"
    );
    assert_eq!(
        fixture.source.history_reads.load(Ordering::SeqCst),
        0,
        "retiring and appending under the carried frontier loads no history"
    );

    let full = region_load::load_full(
        &fixture.cold,
        fixture.cold.source.as_ref().unwrap(),
        &head,
        execution,
        1 << 24,
    )
    .unwrap();
    let mut written_full = full.partition.clone();
    written_full
        .entity_arena
        .apply_extra_update(slot, extra, version)
        .expect("fixture slot is materialized");
    let (oracle, _) = PartitionContentCommitment::capture(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &worth_execution::ExecutionRequest::serial(&crate::test_execution_policy::serial_request()),
        &written_full,
        &symbols,
    )
    .unwrap();
    assert_eq!(
        appended.digest(fixture.entity.partition_id),
        oracle.digest(fixture.entity.partition_id)
    );
    assert_eq!(
        written_full
            .entity_arena
            .metadata_history_at(slot)
            .unwrap()
            .len(),
        fixture.expected_history + 1
    );
}

#[derive(Clone, Copy, Debug)]
enum Tamper {
    MissingCurrent,
    ChangedCurrent,
    ExtraFrontier,
    ExtraMetadataRow,
}

#[test]
fn malformed_current_metadata_and_frontier_coverage_are_sticky_typed_refusals() {
    for tamper in [
        Tamper::MissingCurrent,
        Tamper::ChangedCurrent,
        Tamper::ExtraFrontier,
        Tamper::ExtraMetadataRow,
    ] {
        let fixture = region_with_history();
        let mut head = decode_head(&fixture);
        match tamper {
            Tamper::MissingCurrent => head.partition.entity_arena.metadata_history[0].clear(),
            Tamper::ChangedCurrent => {
                head.partition.entity_arena.metadata_history[0][0].generation ^= 1
            }
            Tamper::ExtraFrontier => {
                let mut wire = serde_json::to_value(&head.history_frontiers).unwrap();
                let entries = wire["entities"].as_array_mut().unwrap();
                let mut extra = entries[0].clone();
                extra[0] = serde_json::Value::from(u64::MAX - 1);
                entries.push(extra);
                head.history_frontiers = serde_json::from_value(wire).unwrap();
            }
            Tamper::ExtraMetadataRow => {
                head.partition
                    .entity_arena
                    .metadata_history
                    .push(head.partition.entity_arena.metadata_history[0].clone());
            }
        }
        let (cell, source) = cell_with_head(fixture, head);
        let first = head_demand(&cell, 1 << 24).unwrap_err();
        assert_eq!(first.cause, Cause::Damaged, "{tamper:?}");
        assert!(matches!(first.subject, RelationalStorageSubject::Region(_)));
        let reads = source.reads.load(Ordering::SeqCst);
        assert_eq!(
            full_demand(&cell, 1 << 24).unwrap_err(),
            first,
            "{tamper:?}"
        );
        assert_eq!(source.reads.load(Ordering::SeqCst), reads);
        assert_eq!(cell.leases.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn the_head_historical_projection_is_authenticated_without_a_history_read() {
    let fixture = region_with_history();
    let mut head = decode_head(&fixture);
    head.history_kind_projection[0].kind = KindId(999);
    let (cell, source) = cell_with_head(fixture, head);
    let failure = head_demand(&cell, 1 << 24).unwrap_err();
    assert_eq!(failure.cause, Cause::IntegrityMismatch);
    assert!(matches!(
        failure.subject,
        RelationalStorageSubject::Region(_)
    ));
    let reads = source.reads.load(Ordering::SeqCst);
    assert_eq!(full_demand(&cell, 1 << 24).unwrap_err(), failure);
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
    assert_eq!(source.history_reads.load(Ordering::SeqCst), 0);
    assert_eq!(cell.leases.load(Ordering::SeqCst), 0);
}

pub(super) fn decode_head(fixture: &RegionFixture) -> RegionHeadImage {
    let bytes = fixture.source.members.get(&fixture.cold.head).unwrap();
    let frame = RelationalRegionImage::decode(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &mut |_, _| Ok(()),
        bytes,
        fixture.cold.head,
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
    )
    .unwrap();
    rmp_serde::from_slice(frame.payload).unwrap()
}

fn cell_with_head(
    fixture: RegionFixture,
    head: RegionHeadImage,
) -> (Arc<RelationalRegionResidency>, Arc<CountedSource>) {
    let mut cold = fixture.cold;
    let (locator, bytes) = RelationalRegionImage::encode(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &mut |_, _| Ok(()),
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
        &head,
    )
    .unwrap();
    let mut members = fixture.source.members.clone();
    members.insert(locator, bytes);
    let source = Arc::new(CountedSource {
        members,
        history: cold.history,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        corrupt: AtomicBool::new(false),
    });
    cold.head = locator;
    cold.source = Some(source.clone());
    (RelationalRegionResidency::cold(cold).unwrap(), source)
}

// Fixture adaptation (C2.4): restores 427700edd0:cold_metadata_pins.rs's
// private cached-Head / absent-per-runtime-dictionary `SymbolsNotResident`
// window over ExternalMembers recovery into a Memory runtime and its fork.
#[test]
fn metadata_pin_refuses_shared_head_until_its_runtime_has_the_aspect_meanings() {
    use crate::durability::data::{
        RecoveryVerificationMode, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    };
    use crate::durability::medium::{
        RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
    };
    use crate::tests::support::{
        create_entity, runtime_with_test_schema, snapshot_for_owner_branch,
        update_entity_and_release_snapshot,
    };
    use crate::visibility::materialization::read_records::RelationalEntityMetadataPinDenial as Denial;
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let name_aspect = worth_foundational::facade::AspectKey::new("name").unwrap();
    let producer = runtime_with_test_schema();
    let entity = create_entity(execution, &producer, "shared-head-meta");
    update_entity_and_release_snapshot(execution, &producer, entity, "shared-head-meta-updated");
    let native = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let members = native
        .members()
        .iter()
        .map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.to_vec()),
            RelationalCheckpointMember::Carried { .. } => unreachable!(),
        })
        .collect();
    let source = Arc::new(CountedSource {
        members,
        history: crate::durability::medium::RelationalDurableMemberLocator {
            sha256: [0; 32],
            length: 0,
        },
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        corrupt: AtomicBool::new(false),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            native,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut runtime = runtime_with_test_schema();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime.durability_recovery().recover(execution, plan).unwrap();
    let fork = runtime.fork().unwrap();
    let orig_snap =
        snapshot_for_owner_branch(&runtime, &crate::history::data::BranchId("main".into()));
    let _ = runtime
        .read_truth()
        .read_exact_snapshot_entity_aspect_version(execution, &orig_snap, entity, &name_aspect)
        .unwrap();
    runtime.snapshots().release_snapshot(&orig_snap).unwrap();
    let snapshot = snapshot_for_owner_branch(&fork, &crate::history::data::BranchId("main".into()));
    let (_, basis) = fork.observe_branch(&fork.main_branch_identity()).unwrap();
    let reads = source.reads.load(Ordering::Relaxed);
    basis
        .observation()
        .selected_root()
        .demand_region_head_cached(execution, entity.partition_id)
        .unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), reads);
    assert!(matches!(
        fork.read_truth().pin_resident_entity_metadata(&snapshot, entity),
        Err(Denial::SymbolsNotResident(partition)) if partition == entity.partition_id
    ));
    let original = snapshot_for_owner_branch(&runtime, &crate::history::data::BranchId("main".into()));
    let expected = runtime
        .read_truth()
        .read_exact_snapshot_entity_aspect_version(execution, &original, entity, &name_aspect)
        .unwrap();
    runtime.snapshots().release_snapshot(&original).unwrap();
    assert_eq!(
        fork.read_truth()
            .read_exact_snapshot_entity_aspect_version(execution, &snapshot, entity, &name_aspect)
            .unwrap(),
        expected
    );
    let metadata = fork
        .read_truth()
        .pin_resident_entity_metadata(&snapshot, entity)
        .unwrap()
        .unwrap();
    assert_eq!(metadata.live_kind(), Some(KindId(1)));
    assert_eq!(metadata.aspect_version(&name_aspect), expected);
    fork.snapshots().release_snapshot(&snapshot).unwrap();
}
