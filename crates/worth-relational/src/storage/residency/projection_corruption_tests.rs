//! Valid member hashes do not authenticate derived kind caches or owned history.
use super::super::region_image::*;
use super::*;

#[test]
fn kind_cache_disagreement_with_authenticated_projection_is_typed() {
    let fixture = region_with_history();
    let frame = RelationalRegionImage::decode(
        &fixture.source.members[&fixture.cold.head],
        fixture.cold.head,
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
    )
    .unwrap();
    let mut head: RegionHeadImage = rmp_serde::from_slice(frame.payload).unwrap();
    head.adjacency_kinds
        .iter_mut()
        .find(|entry| !entry.kinds.is_empty())
        .unwrap()
        .kinds[0]
        .0 = crate::identity::data::KindId(999);
    let member = RelationalRegionImage::encode(
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
        &head,
    )
    .unwrap();
    let cell = replaced_member(&fixture, member, false);
    assert_integrity_refusal(head_demand(&cell, 1 << 24).unwrap_err());
}

#[test]
fn historical_projection_is_reconstructed_from_owned_record_history() {
    let fixture = region_with_history();
    let frame = RelationalRegionImage::decode(
        &fixture.source.members[&fixture.cold.history],
        fixture.cold.history,
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
    )
    .unwrap();
    let mut history: RegionHistoryImage = rmp_serde::from_slice(frame.payload).unwrap();
    history.kind_projection[0].kind = crate::identity::data::KindId(999);
    for entry in history
        .adjacency_kinds
        .iter_mut()
        .chain(&mut history.reverse_adjacency_kinds)
    {
        for (kind, _) in &mut entry.kinds {
            *kind = crate::identity::data::KindId(999);
        }
    }
    let member = RelationalRegionImage::encode(
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
        &history,
    )
    .unwrap();
    let mut cold = fixture.cold.clone();
    cold.kind_authority.history =
        super::super::kind_commitment::digest(&history.kind_projection).unwrap();
    // Even a rehashed projection and matching cache cannot replace owned arena
    // history. The canonical root separately prevents replacing peer authority.
    let mut members = fixture.source.members.clone();
    members.insert(member.0, member.1);
    cold.history = member.0;
    cold.source = Arc::new(CountedSource {
        members,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        history: cold.history,
        corrupt: AtomicBool::new(false),
    });
    assert_integrity_refusal(
        full_demand(&RelationalRegionResidency::cold(cold).unwrap(), 1 << 24).unwrap_err(),
    );
}

fn replaced_member(
    fixture: &RegionFixture,
    member: (RelationalDurableMemberLocator, Vec<u8>),
    history: bool,
) -> Arc<RelationalRegionResidency> {
    let mut cold = fixture.cold.clone();
    if history {
        cold.history = member.0;
    } else {
        cold.head = member.0;
    }
    let mut members = fixture.source.members.clone();
    members.insert(member.0, member.1);
    cold.source = Arc::new(CountedSource {
        members,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        history: cold.history,
        corrupt: AtomicBool::new(false),
    });
    RelationalRegionResidency::cold(cold).unwrap()
}

fn assert_integrity_refusal(failure: crate::storage::data::RelationalStorageUnavailable) {
    assert!(
        failure.cause == Cause::IntegrityMismatch
            && matches!(failure.subject, RelationalStorageSubject::Region(_))
    );
}
