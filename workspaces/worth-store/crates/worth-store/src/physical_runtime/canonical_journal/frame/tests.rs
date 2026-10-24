use super::*;

fn journal() -> RecordAnchorName {
    RecordAnchorName::new("relational.canonical-commits").unwrap()
}

fn record(position: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([3; 16], position).unwrap()
}

/// Builds headers for positions 1..=count, each from its predecessor alone.
fn chain(count: u64) -> Vec<JournalRecordHeader> {
    let mut headers: Vec<JournalRecordHeader> = Vec::new();
    for position in 1..=count {
        let head = headers.last().map(|head| (head, record(head.position)));
        let next =
            JournalRecordHeader::successor(journal(), head, &position.to_le_bytes()).unwrap();
        assert_eq!(next.position, position);
        headers.push(next);
    }
    headers
}

/// Finds `target` from the record at `from`, counting visited headers.
fn lookup(headers: &[JournalRecordHeader], from: u64, target: u64) -> (u64, usize) {
    let mut current = &headers[from as usize - 1];
    let mut hops = 0;
    while current.position != target {
        let next = current
            .descend_toward(target)
            .expect("an earlier target is reachable");
        assert_eq!(next.record, record(next.position));
        current = &headers[next.position as usize - 1];
        hops += 1;
    }
    (current.position, hops)
}

#[test]
fn spines_name_the_largest_aligned_predecessors() {
    let headers = chain(12);
    let positions = |p: usize| {
        headers[p - 1]
            .spine
            .iter()
            .map(|e| e.position)
            .collect::<Vec<_>>()
    };
    assert!(headers[0].spine.is_empty());
    assert_eq!(positions(2), [1]);
    assert_eq!(positions(9), [8]);
    assert_eq!(positions(12), [11, 10, 8]);
}

#[test]
fn every_target_is_reachable_in_logarithmic_hops() {
    let headers = chain(300);
    for target in 1..=300 {
        let (found, hops) = lookup(&headers, 300, target);
        assert_eq!(found, target);
        assert!(hops <= 2 * 9, "target {target} took {hops} hops");
    }
}

#[test]
fn records_round_trip_and_bind_their_payload() {
    let headers = chain(6);
    let header = &headers[5];
    let payload = 6_u64.to_le_bytes();
    let bytes = header.encode_record(&payload);
    assert_eq!(bytes.len(), header.encoded_bytes() + payload.len());
    let decoded = JournalRecordHeader::decode(&bytes, journal(), Some(bytes.len() as u64)).unwrap();
    assert_eq!(&decoded, header);
    decoded
        .verify_payload(&bytes[header.encoded_bytes()..])
        .unwrap();
    assert_eq!(
        decoded.verify_payload(b"other!!!"),
        Err(JournalFrameDenial::PayloadDigestMismatch)
    );
}

#[test]
fn decoding_refuses_foreign_or_damaged_headers() {
    let header = &chain(6)[5];
    let bytes = header.encode_record(&6_u64.to_le_bytes());
    let length = Some(bytes.len() as u64);
    let other = RecordAnchorName::new("world.occurrences").unwrap();
    assert_eq!(
        JournalRecordHeader::decode(&bytes, other, length),
        Err(JournalFrameDenial::WrongJournal)
    );
    let mut magic = bytes.clone();
    magic[0] = b'X';
    assert_eq!(
        JournalRecordHeader::decode(&magic, journal(), length),
        Err(JournalFrameDenial::WrongMagic)
    );
    assert_eq!(
        JournalRecordHeader::decode(&bytes, journal(), Some(bytes.len() as u64 + 1)),
        Err(JournalFrameDenial::Malformed)
    );
    let mut wrong_spine = bytes.clone();
    wrong_spine[96] ^= 1;
    assert_eq!(
        JournalRecordHeader::decode(&wrong_spine, journal(), length),
        Err(JournalFrameDenial::Malformed)
    );
    assert_eq!(
        JournalRecordHeader::decode(&bytes[..40], journal(), length),
        Err(JournalFrameDenial::Truncated)
    );
}
