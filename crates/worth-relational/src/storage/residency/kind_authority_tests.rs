use super::*;
use crate::storage::residency::region_image::{
    RegionHeadImage, RegionHistoryImage, RelationalRegionImage, RelationalRegionImageKind,
};

#[test]
fn intact_member_with_forged_current_kind_cannot_serve_false_absence() {
    let fixture = region_with_history();
    let mut cold = fixture.cold;
    let bytes = fixture.source.members.get(&cold.head).unwrap();
    let frame = RelationalRegionImage::decode(
        bytes,
        cold.head,
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
    )
    .unwrap();
    let mut head: RegionHeadImage = rmp_serde::from_slice(frame.payload).unwrap();
    let bucket = head
        .adjacency_kinds
        .iter_mut()
        .find(|entry| !entry.kinds.is_empty())
        .unwrap();
    assert!(bucket
        .kinds
        .iter()
        .any(|(_, ids)| ids.contains(&fixture.relation)));
    bucket.kinds[0].0 = crate::identity::data::KindId(u32::MAX);
    let member = RelationalRegionImage::encode(
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
        &head,
    )
    .unwrap();
    cold.head = member.0;
    let source = source_with_member(&fixture.source, member);
    cold.source = source.clone();
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    let failure = head_demand(&cell, 1 << 24).unwrap_err();
    assert_eq!(failure.cause, Cause::IntegrityMismatch);
    let reads = source.reads.load(Ordering::SeqCst);
    assert_eq!(full_demand(&cell, 1 << 24).unwrap_err(), failure);
    assert_eq!(source.reads.load(Ordering::SeqCst), reads);
}

#[test]
fn intact_history_cannot_omit_authoritative_kind_membership() {
    let fixture = region_with_history();
    let mut cold = fixture.cold;
    let bytes = fixture.source.members.get(&cold.history).unwrap();
    let frame = RelationalRegionImage::decode(
        bytes,
        cold.history,
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
    )
    .unwrap();
    let mut history: RegionHistoryImage = rmp_serde::from_slice(frame.payload).unwrap();
    assert!(!history.relations[0].is_empty());
    history.relations[0].clear();
    let member = RelationalRegionImage::encode(
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
        &history,
    )
    .unwrap();
    cold.history = member.0;
    cold.source = source_with_member(&fixture.source, member);
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    assert!(head_demand(&cell, 1 << 24)
        .unwrap()
        .head_view()
        .relation(fixture.relation)
        .is_some());
    assert_eq!(
        full_demand(&cell, 1 << 24).unwrap_err().cause,
        Cause::IntegrityMismatch
    );
}

fn source_with_member(
    source: &CountedSource,
    member: (RelationalDurableMemberLocator, Vec<u8>),
) -> Arc<CountedSource> {
    let mut members = source.members.clone();
    members.insert(member.0, member.1);
    Arc::new(CountedSource {
        members,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        history: source.history,
        corrupt: AtomicBool::new(false),
    })
}
