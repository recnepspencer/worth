//! Real encoded members cross Head capture, placement, reopen and Full demand.
use super::*;
use crate::durability::data::{RelationalCheckpointMember, RelationalHistoryMemberDirectory};
use crate::identity::data::VersionId;
use crate::storage::overlay::PartitionState;
use std::collections::BTreeSet;

#[test]
fn head_capture_acknowledges_and_carries_the_entire_layer_closure_without_loading() {
    let fixture = region_with_history();
    let mut cold = fixture.cold.clone();
    let mut source = fixture.source.clone();
    let mut cell = fixture.cell.clone();
    let expected = fixture.expected_digest;
    for layer in 1..=4 {
        head_demand(&cell, 1 << 24).unwrap();
        let destination = source
            .members
            .keys()
            .copied()
            .filter(|member| *member != cold.head)
            .collect::<BTreeSet<_>>();
        let before = source.reads.load(Ordering::SeqCst);
        let captured = cell
            .capture_checkpoint(
                &cold.schema,
                cold.schema_carrier_digest,
                &cold.symbols.interner_snapshot(),
                Some(&destination),
            )
            .unwrap();
        assert_eq!(source.reads.load(Ordering::SeqCst), before);
        assert_eq!(captured.leaf.content_digest, expected);
        assert!(captured.members.iter().any(|member| matches!(member, RelationalCheckpointMember::Carried { locator } if *locator == cold.history)));
        let members = place(&captured.members, &source.members);
        captured.publication.acknowledge();
        let current = members.keys().copied().collect::<BTreeSet<_>>();
        let repeated = cell
            .capture_checkpoint(
                &cold.schema,
                cold.schema_carrier_digest,
                &cold.symbols.interner_snapshot(),
                Some(&current),
            )
            .unwrap();
        assert!(repeated
            .members
            .iter()
            .all(|member| matches!(member, RelationalCheckpointMember::Carried { .. })));
        assert_eq!(repeated.history_dependencies, captured.history_dependencies);
        assert_eq!(source.reads.load(Ordering::SeqCst), before);
        let directory = RelationalHistoryMemberDirectory::readmit(
            &captured.history_dependencies,
            [captured.leaf.history],
        )
        .unwrap();
        let next_source = counted(members, captured.leaf.history);
        cold = RelationalColdRegion {
            head: captured.leaf.head,
            history: captured.leaf.history,
            history_members: Arc::new(directory),
            source: next_source.clone(),
            kind_authority: captured.leaf.kind_authority,
            ..cold
        };
        cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
        head_demand(&cell, 1 << 24).unwrap();
        assert_eq!(next_source.history_reads.load(Ordering::SeqCst), 0);
        let full = full_demand(&cell, 1 << 24).unwrap();
        assert_eq!(
            full.full_view()
                .partition()
                .authoritative_content_digest(&cold.symbols.interner_snapshot())
                .unwrap(),
            expected
        );
        assert!(
            next_source.reads.load(Ordering::SeqCst) <= layer + 4,
            "shared bases are read within layer headroom"
        );
        source = next_source;
        // Start the next capture from a fresh Head cell, without the Full oracle.
        cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
    }
}

#[test]
fn appending_at_head_across_reopened_layers_matches_complete_history() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let original = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let mut oracle = region_load::load_full(&fixture.cold, &original, execution, 1 << 24)
        .unwrap()
        .partition
        .clone();
    let mut cold = fixture.cold.clone();
    let mut source = fixture.source.clone();
    for version in 1_000..1_004 {
        let head = region_load::load_head(&cold, execution, 1 << 24).unwrap();
        let mut partial = head.partition.clone();
        let extra = partial
            .entity_arena
            .get_slot(fixture.entity.slot_index())
            .unwrap()
            .extra()
            .clone();
        partial.entity_arena.apply_extra_update(
            fixture.entity.slot_index(),
            extra.clone(),
            VersionId(version),
        );
        oracle.entity_arena.apply_extra_update(
            fixture.entity.slot_index(),
            extra,
            VersionId(version),
        );
        let before = source.reads.load(Ordering::SeqCst);
        let (next, next_source) = encode_layer(&cold, &source, &partial);
        assert_eq!(source.reads.load(Ordering::SeqCst), before);
        let reopened = RelationalRegionResidency::cold(next.clone()).unwrap();
        let full = full_demand(&reopened, 1 << 24).unwrap();
        let symbols = cold.symbols.interner_snapshot();
        assert_eq!(
            full.full_view()
                .partition()
                .authoritative_content_digest(&symbols)
                .unwrap(),
            oracle.authoritative_content_digest(&symbols).unwrap()
        );
        cold = next;
        source = next_source;
    }
}

#[test]
fn ancestor_damage_is_sticky_and_aggregate_input_limits_are_retryable() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let (cold, source) = encode_layer(&fixture.cold, &fixture.source, &head.partition);
    let cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
    head_demand(&cell, 1 << 24).unwrap();
    let limit = cold.history.length.max(fixture.cold.history.length);
    let failure = full_demand(&cell, limit).unwrap_err();
    assert!(matches!(failure.cause, Cause::LimitExceeded { .. }));
    head_demand(&cell, 1 << 24).unwrap();
    full_demand(&cell, 1 << 24).unwrap();

    let mut members = source.members.clone();
    members.get_mut(&fixture.cold.history).unwrap()[0] ^= 1;
    let damaged = counted(members, cold.history);
    let cell = RelationalRegionResidency::cold(RelationalColdRegion {
        source: damaged.clone(),
        ..cold
    })
    .unwrap();
    head_demand(&cell, 1 << 24).unwrap();
    let failure = full_demand(&cell, 1 << 24).unwrap_err();
    assert_eq!(failure.cause, Cause::IntegrityMismatch);
    assert_eq!(
        failure.subject,
        RelationalStorageSubject::Region(fixture.entity.partition_id)
    );
    let before = damaged.reads.load(Ordering::SeqCst);
    assert_eq!(head_demand(&cell, 1 << 24).unwrap_err(), failure);
    assert_eq!(full_demand(&cell, 1 << 24).unwrap_err(), failure);
    assert_eq!(damaged.reads.load(Ordering::SeqCst), before);
}

pub(super) fn encode_layer(
    cold: &RelationalColdRegion,
    source: &Arc<CountedSource>,
    partition: &PartitionState,
) -> (RelationalColdRegion, Arc<CountedSource>) {
    let symbols = cold.symbols.interner_snapshot();
    let (commitment, _) = PartitionContentCommitment::capture_layered(partition, &symbols).unwrap();
    let captured = capture_region_members(
        partition,
        &commitment,
        &cold.schema,
        cold.schema_carrier_digest,
    )
    .unwrap();
    let mut members = BTreeMap::new();
    for member in captured.history_members.members(captured.history.0) {
        if member != captured.history.0 {
            members.insert(member, source.members[&member].clone());
        }
    }
    members.insert(captured.head.0, captured.head.1);
    members.insert(captured.history.0, captured.history.1);
    let source = counted(members, captured.history.0);
    (
        RelationalColdRegion {
            head: captured.head.0,
            history: captured.history.0,
            history_members: captured.history_members,
            content_digest: commitment.digest(partition.partition_id),
            kind_authority: captured.kind_authority,
            source: source.clone(),
            ..cold.clone()
        },
        source,
    )
}

fn place(
    members: &[RelationalCheckpointMember],
    prior: &BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
) -> BTreeMap<RelationalDurableMemberLocator, Vec<u8>> {
    members
        .iter()
        .map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.to_vec()),
            RelationalCheckpointMember::Carried { locator } => (*locator, prior[locator].clone()),
        })
        .collect()
}

pub(super) fn counted(
    members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    history: RelationalDurableMemberLocator,
) -> Arc<CountedSource> {
    Arc::new(CountedSource {
        members,
        history,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        corrupt: AtomicBool::new(false),
    })
}
