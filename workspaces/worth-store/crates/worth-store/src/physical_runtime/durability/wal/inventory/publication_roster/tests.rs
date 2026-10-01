use super::*;
use worth_store_wal::LogSequenceNumber;

#[test]
fn complete_group_carries_each_member_price_fact_once() {
    let roster = roster(vec![member(2, 2, 1, 2, 3, 5), member(1, 2, 1, 1, 2, 2)]);
    let groups = roster.finish().unwrap();
    assert_eq!(groups.len(), 1);
    let group = &groups[0];
    assert_eq!(group.segment(), (1, 1));
    assert_eq!(group.lsn_range(), range(1, 3));
    assert_eq!(group.encoded_wal_bytes(), 200);
    assert_eq!(group.members().len(), 2);
    assert_eq!(group.members()[0].inserted_records(), 2);
    assert_eq!(group.members()[1].inserted_records(), 5);
}

#[test]
fn missing_ordinal_is_an_explicit_incomplete_group() {
    let roster = roster(vec![member(1, 2, 1, 1, 2, 1)]);
    assert_eq!(
        roster.finish(),
        Err(PhysicalWalOpenFailure::IncompletePublicationGroup)
    );
}

#[test]
fn split_segment_duplicate_ordinal_and_lsn_gap_are_rejected() {
    for second in [
        member(2, 2, 2, 2, 3, 1),
        member(1, 2, 1, 2, 3, 1),
        member(2, 2, 1, 3, 4, 1),
    ] {
        let roster = roster(vec![member(1, 2, 1, 1, 2, 1), second]);
        assert_eq!(
            roster.finish(),
            Err(PhysicalWalOpenFailure::MemberPayloadRejected)
        );
    }
}

#[test]
fn consistent_but_forged_membership_digest_is_rejected() {
    let mut roster = roster(vec![member(1, 2, 1, 1, 2, 1), member(2, 2, 1, 2, 3, 1)]);
    for member in &mut roster.members {
        member.membership = [0x55; 32];
    }
    assert_eq!(
        roster.finish(),
        Err(PhysicalWalOpenFailure::MemberPayloadRejected)
    );
}

#[test]
fn correctly_digested_duplicate_member_or_idempotency_identity_is_rejected() {
    for duplicate_member in [true, false] {
        let first = member(1, 2, 1, 1, 2, 1);
        let mut second = member(2, 2, 1, 2, 3, 1);
        if duplicate_member {
            second.member_identity = first.member_identity;
        } else {
            second.idempotency_identity = first.idempotency_identity;
        }
        let roster = roster(vec![first, second]);
        assert_eq!(
            roster.finish(),
            Err(PhysicalWalOpenFailure::MemberPayloadRejected)
        );
    }
}

#[test]
fn one_verified_frame_requires_its_four_slot_capacity_before_inspection() {
    let frame = std::mem::size_of::<worth_store_wal::VerifiedWalFramePayload<'_>>() as u64;
    let segment = 1_024;
    let too_small = PublicationRoster::new(segment + frame * 3);
    assert_eq!(too_small.admitted_frame_view_count(segment).unwrap(), 0);
    let admitted = PublicationRoster::new(segment + frame * 4);
    assert_eq!(admitted.admitted_frame_view_count(segment).unwrap(), 1);
    assert_eq!(
        PublicationRoster::frame_view_capacity_ceiling(1).unwrap(),
        frame * 4
    );
}

#[test]
fn fifth_frame_requires_old_and_new_doubling_buffers_at_peak() {
    let frame = std::mem::size_of::<worth_store_wal::VerifiedWalFramePayload<'_>>() as u64;
    let segment = 1_024;
    assert_eq!(
        PublicationRoster::frame_view_capacity_ceiling(5).unwrap(),
        frame * 15
    );
    let short = PublicationRoster::new(segment + frame * 14);
    assert_eq!(short.admitted_frame_view_count(segment).unwrap(), 4);
    let enough = PublicationRoster::new(segment + frame * 15);
    assert_eq!(enough.admitted_frame_view_count(segment).unwrap(), 5);
    assert_eq!(
        PublicationRoster::frame_view_capacity_ceiling(9).unwrap(),
        frame * 27
    );
    let short = PublicationRoster::new(segment + frame * 26);
    assert_eq!(short.admitted_frame_view_count(segment).unwrap(), 8);
    let enough = PublicationRoster::new(segment + frame * 27);
    assert_eq!(enough.admitted_frame_view_count(segment).unwrap(), 9);
}

#[test]
fn the_same_valid_member_admits_with_sufficient_grant_and_denies_with_one_entry_slot() {
    use worth_store_physical_format::{
        CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, ExtentArenaId,
        ExtentArenaRange, ExtentChunkCoordinate, PersistedPhysicalDataFrameSubject,
        PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryManifest,
        PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState,
        PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
        RecordArtifactFile, RecordFrameCoordinate,
    };

    let record = PersistedRecordIdentity::new([7; 16], 9).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 53_248).unwrap();
    let placement = DurableExtentRecordPlacement::legacy_unknown(record, extent, 1, range).unwrap();
    let artifact = RecordArtifactFile::ExtentArena { arena: 2 };
    let coordinate = RecordFrameCoordinate::new(artifact, 0, 1).unwrap();
    let subject = PersistedPhysicalDataFrameSubject::ExtentChunk(
        ExtentChunkCoordinate::new(record, extent, 1, 0, 1).unwrap(),
    );
    let manifests = [1_u64, 2].map(|offset| {
        PersistedPhysicalRecoveryManifest::new(
            RecordFrameCoordinate::new(artifact, offset, 1).unwrap(),
            &[offset as u8],
        )
        .unwrap()
    });
    let projection = PersistedPhysicalRecoveryProjection::new(
        11,
        PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap(),
        vec![record],
        vec![PersistedPhysicalRecoveryFrame::new(subject, coordinate, &[3]).unwrap()],
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        vec![],
        manifests.into(),
    )
    .unwrap();
    let mut redo = Vec::new();
    write_field(&mut redo, CANONICAL_REDO_V3_DOMAIN);
    redo.extend_from_slice(&1_u64.to_le_bytes());
    redo.extend_from_slice(&0_u32.to_le_bytes());
    redo.extend_from_slice(&1_u64.to_le_bytes());
    redo.extend_from_slice(&1_u64.to_le_bytes());
    let mut target = vec![2];
    target.extend_from_slice(&[7; 16]);
    for number in [9_u64, 3, 4, 1, 0] {
        target.extend_from_slice(&number.to_le_bytes());
    }
    target.extend_from_slice(&1_u32.to_le_bytes());
    target.push(16);
    target.extend_from_slice(&2_u64.to_le_bytes());
    target.extend_from_slice(&0_u64.to_le_bytes());
    target.extend_from_slice(&1_u32.to_le_bytes());
    write_field(&mut redo, &target);
    redo.extend_from_slice(&Sha256::digest([3]));
    write_field(&mut redo, b"redo");
    write_field(&mut redo, &projection.encode());
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let entry_size = [
        std::mem::size_of::<worth_store_physical_format::CanonicalRedoWireRecord>(),
        std::mem::size_of::<worth_store_physical_format::CanonicalRedoTarget>(),
        std::mem::size_of::<PersistedPhysicalRecoveryFrame>(),
        std::mem::size_of::<PersistedPhysicalRecoveryManifest>(),
        std::mem::size_of::<PersistedRecordIdentity>(),
        std::mem::size_of::<CurrentPhysicalRecordPlacement>(),
        std::mem::size_of::<worth_store_physical_format::RecordSegmentPageManifestEntry>(),
        std::mem::size_of::<worth_store_physical_format::PersistedInlineSegmentAllocation>(),
        std::mem::size_of::<usize>(),
    ]
    .into_iter()
    .max()
    .unwrap() as u64;
    let per_entry = entry_size * 18;
    let payload = redo.len() as u64 * 4;
    assert!(decode_metadata(&redo, range_lsn(), format, payload + per_entry * 4).is_ok());
    let mut unsupported = redo.clone();
    let domain = b"store.physical.recovery-projection.v15";
    let domain_offset = unsupported
        .windows(domain.len())
        .position(|window| window == domain)
        .unwrap();
    unsupported[domain_offset..domain_offset + domain.len()]
        .copy_from_slice(b"store.physical.recovery-projection.v14");
    assert_eq!(
        decode_metadata(&unsupported, range_lsn(), format, payload + per_entry * 4),
        Err(PhysicalWalOpenFailure::UnsupportedRecoveryProjectionVersion(14))
    );
    assert!(matches!(
        decode_metadata(&redo, range_lsn(), format, payload + per_entry),
        Err(PhysicalWalOpenFailure::ReopenAllocationLimitExceeded { .. })
    ));
}

fn range_lsn() -> WalLsnRange {
    range(1, 2)
}

fn write_field(target: &mut Vec<u8>, field: &[u8]) {
    target.extend_from_slice(&(field.len() as u64).to_le_bytes());
    target.extend_from_slice(field);
}

fn roster(mut members: Vec<ObservedPublicationMember>) -> PublicationRoster {
    members.sort_unstable_by_key(|member| member.ordinal);
    let digest = reopened_membership_digest_fields(
        members.len(),
        members.iter().map(|member| {
            (
                member.mutation_store,
                member.mutation_runtime,
                member.mutation_operation,
                member.member_identity,
                member.idempotency_identity,
            )
        }),
    )
    .unwrap();
    for member in &mut members {
        member.membership = digest;
    }
    PublicationRoster {
        members,
        admitted_bytes: u64::MAX,
    }
}

fn member(
    ordinal: u32,
    count: u32,
    segment: u64,
    start: u64,
    end: u64,
    inserted_records: u64,
) -> ObservedPublicationMember {
    ObservedPublicationMember {
        group: [7; 32],
        membership: [0; 32],
        ordinal,
        count,
        segment,
        generation: 1,
        range: range(start, end),
        encoded_bytes: 100,
        metadata: ReopenedPublicationMemberMetadata {
            source_root_generation: 3,
            successor_manifest_capacity: Some(32),
            inserted_records,
        },
        mutation_store: [1; 16],
        mutation_runtime: 2,
        mutation_operation: ordinal as u64,
        member_identity: [ordinal as u8; 32],
        idempotency_identity: [ordinal as u8 + 10; 32],
    }
}

fn range(start: u64, end: u64) -> WalLsnRange {
    WalLsnRange::new(LogSequenceNumber::new(start), LogSequenceNumber::new(end)).unwrap()
}
