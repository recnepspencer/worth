//! Directory disagreement and row amplification refuse bounded reconstruction.
use super::super::region_image::*;
use super::layered_history_tests::{counted, encode_layer};
use super::*;
use crate::durability::data::{
    RelationalCheckpointHistoryDependency, RelationalHistoryMemberDirectory,
};
use crate::storage::substrate::EntityRecordKind;

#[test]
fn an_undeclared_base_is_refused_before_its_payload_is_read() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let (mut cold, source) = encode_layer(&fixture.cold, &fixture.source, &head.partition);
    cold.history_members = Arc::default();
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    head_demand(&cell, 1 << 24).unwrap();
    let before = source.reads.load(Ordering::SeqCst);
    let failure = full_demand(&cell, 1 << 24).unwrap_err();
    assert_eq!(failure.cause, Cause::Damaged);
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        before + 1,
        "only the declared root is read"
    );
    assert_eq!(head_demand(&cell, 1 << 24).unwrap_err(), failure);
}

#[test]
fn shared_row_bases_cannot_expand_beyond_the_original_read_limit() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let mut complete = region_load::load_full(&fixture.cold, &head, execution, 1 << 24)
        .unwrap()
        .partition
        .clone();
    let slot = fixture.entity.slot_index();
    let extra = complete
        .entity_arena
        .get_slot(slot)
        .unwrap()
        .extra()
        .clone();
    for version in 1_000..1_064 {
        complete.entity_arena.apply_extra_update(
            slot,
            extra.clone(),
            crate::identity::data::VersionId(version),
        );
    }
    let (cold, produced) = super::mixed_history_tests::recapture(&fixture.cold, &complete);
    let bytes = &produced.members[&cold.history];
    let frame = RelationalRegionImage::decode(
        bytes,
        cold.history,
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
    )
    .unwrap();
    let mut image: RegionHistoryImage = rmp_serde::from_slice(frame.payload).unwrap();
    let base = RegionHistoryBase {
        member: cold.history,
        partition_id: cold.partition_id,
        schema_carrier_digest: cold.schema_carrier_digest,
        row_index: 0,
        kind_commitment: cold.kind_authority.history,
    };
    let per_row = image.entities[0]
        .earlier
        .iter()
        .map(|metadata| rmp_serde::to_vec(metadata).unwrap().len() as u64)
        .sum::<u64>();
    image.entities = (0..128)
        .map(|_| RegionHistoryRow {
            prefix: Some(base.clone()),
            earlier: Vec::new(),
        })
        .collect();
    // This is an untrusted amplification input to the folder, not a claimed
    // branch root or a fabricated Full residency witness.
    let encoded = RelationalRegionImage::encode(
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
        &image,
    )
    .unwrap();
    let directory = Arc::new(
        RelationalHistoryMemberDirectory::readmit(
            &[RelationalCheckpointHistoryDependency {
                member: encoded.0,
                bases: vec![cold.history],
            }],
            [encoded.0],
        )
        .unwrap(),
    );
    let limit = encoded.0.length + cold.history.length + 16;
    assert!(
        per_row * image.entities.len() as u64 > limit * 2,
        "the fixture must actually amplify beyond the input budget"
    );
    let source = counted(
        BTreeMap::from([encoded.clone(), (cold.history, bytes.clone())]),
        encoded.0,
    );
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let mut folder =
        super::super::history_fold::HistoryFolder::new(execution, fixture.cold.partition_id, limit);
    let root = RegionHistoryBase {
        member: encoded.0,
        ..base
    };
    let failure = match folder.fold_rows::<EntityRecordKind>(
        source.as_ref(),
        &directory,
        &root,
        image.entities.len(),
    ) {
        Err(failure) => failure,
        Ok(rows) => panic!(
            "accepted {} amplified rows beyond limit {limit}",
            rows.len()
        ),
    };
    assert!(
        matches!(failure, super::super::RelationalResidencyDemandFailure::StorageUnavailable(failure)
        if matches!(failure.cause, Cause::LimitExceeded { length, limit: actual } if actual == limit && length > limit))
    );
    assert!(
        source.reads.load(Ordering::SeqCst) <= 4,
        "member reuse stays within input-read headroom"
    );
}
