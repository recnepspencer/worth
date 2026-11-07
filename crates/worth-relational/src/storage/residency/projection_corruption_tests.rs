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
    let mut head = super::head_metadata_tests::decode_head(&fixture);
    let claim = &mut history.kind_projection[0];
    let current = head
        .kind_projection
        .iter()
        .find(|current| current.relation == claim.relation)
        .unwrap();
    let removed = *claim.incoming.difference(&current.incoming).next().unwrap();
    claim.incoming.remove(&removed);
    let cache = history
        .reverse_adjacency_kinds
        .iter_mut()
        .find(|entry| entry.slot == removed as u64)
        .unwrap();
    for (_, relations) in &mut cache.kinds {
        relations.retain(|relation| *relation != claim.relation);
    }
    cache.kinds.retain(|(_, relations)| !relations.is_empty());
    head.history_kind_projection = history.kind_projection.clone();
    let head_member = RelationalRegionImage::encode(
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::Head,
        &head,
    )
    .unwrap();
    let member = RelationalRegionImage::encode(
        fixture.cold.partition_id,
        fixture.cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
        &history,
    )
    .unwrap();
    let mut cold = fixture.cold.clone();
    cold.kind_authority.history = super::super::kind_commitment::digest(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &history.kind_projection,
    )
    .unwrap();
    // Even a rehashed projection and matching cache cannot replace owned arena
    // history. The canonical root separately prevents replacing peer authority.
    let mut members = fixture.source.members.clone();
    members.insert(head_member.0, head_member.1);
    members.insert(member.0, member.1);
    cold.head = head_member.0;
    cold.history = member.0;
    let source = Arc::new(CountedSource {
        members,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        history: cold.history,
        corrupt: AtomicBool::new(false),
    });
    cold.source = source.clone();
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    head_demand(&cell, 1 << 24).unwrap();
    assert_eq!(source.history_reads.load(Ordering::SeqCst), 0);
    assert_integrity_refusal(full_demand(&cell, 1 << 24).unwrap_err());
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

#[test]
fn tampered_carried_kind_commitment_is_refused_during_cold_restore() {
    let fixture = region_with_history();
    let mut cold = fixture.cold.clone();
    cold.kind_authority.history[0] ^= 1;
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    let failure = head_demand(&cell, 1 << 24).unwrap_err();
    assert_eq!(failure.cause, Cause::IntegrityMismatch);
}

#[test]
fn carried_live_counts_are_checked_against_admitted_member_rows() {
    for full in [false, true] {
        let fixture = region_with_history();
        let mut cold = fixture.cold.clone();
        cold.live_counts.entities += 1;
        let cell = RelationalRegionResidency::cold(cold).unwrap();
        let failure = if full {
            full_demand(&cell, 1 << 24).unwrap_err()
        } else {
            head_demand(&cell, 1 << 24).unwrap_err()
        };
        assert_integrity_refusal(failure);
    }
}
