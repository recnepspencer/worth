//! Pins the canonical row-set and grouped-projection digests to fixed bytes, so
//! a move or refactor of grouped truth cannot change what a digest certifies.

use crate::facade::{
    RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
    SnapshotReadContract, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
    SnapshotReadRequest, SnapshotReadValue, TruthSnapshotIdentity,
};
use worth_foundational::facade::{
    AspectKey, AspectValue, FieldKey, ScalarAspectType, StructAspectValue,
};

use super::{
    materialize_relational_authoritative_row_set, project_relational_grouped_truth,
    RelationalGroupedProjectionContract,
};

const ROW_SET_DIGEST: &str =
    "relational-row-set:sha256:5be580af22545ab7ab045f462e8e93199c3dde7e35fd13fe474eacecdee0fc6e";
const GROUPED_PROJECTION_DIGEST: &str = "relational-grouped-projection:sha256:b1e8e971d2b36f31eb486f8b4c91e8c73edfa4418a19ea6d8e79384990486287";

#[test]
fn canonical_digests_match_the_pinned_bytes() {
    let task = RelationalBridgeRecordIdentityParts::entity(3, 17, 2);
    let link = RelationalBridgeRecordIdentityParts::relation(5, 41, 9);
    let packet = SnapshotReadPacket::new(vec![
        read(task, "identity.id"),
        read(task, "status.lane"),
        read(link, "identity.id"),
        read(link, "status.lane"),
        read(link, "detail"),
    ]);
    let values: [SnapshotReadValue; 5] = [
        AspectValue::String("task-1".into()).into(),
        AspectValue::String("todo".into()).into(),
        AspectValue::String("link-9".into()).into(),
        AspectValue::String("doing".into()).into(),
        StructAspectValue::new([(
            FieldKey::new("weight").expect("valid field key"),
            AspectValue::String("heavy".into()),
        )])
        .expect("valid struct aspect value")
        .into(),
    ];
    let records = values
        .into_iter()
        .enumerate()
        .map(|(index, value)| SnapshotReadRecord::for_request(&packet.reads()[index], value))
        .collect();
    let result = SnapshotReadPacketResult::new(
        TruthSnapshotIdentity::from_relational_snapshot(
            RelationalBridgeSnapshotIdentityParts::new(29, 31),
        ),
        records,
    );

    let row_set = materialize_relational_authoritative_row_set(&packet, &result).unwrap();
    let grouped = project_relational_grouped_truth(
        &row_set,
        RelationalGroupedProjectionContract::new(
            aspect_key("status"),
            aspect_key("identity.id"),
            aspect_key("status.lane"),
        ),
    )
    .unwrap();

    assert_eq!(row_set.digest().as_str(), ROW_SET_DIGEST);
    assert_eq!(grouped.digest().as_str(), GROUPED_PROJECTION_DIGEST);
}

fn read(record: RelationalBridgeRecordIdentityParts, aspect: &str) -> SnapshotReadRequest {
    SnapshotReadRequest::for_relational_record(
        record,
        SnapshotReadContract::scalar(aspect_key(aspect), ScalarAspectType::String),
    )
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid test aspect key")
}
