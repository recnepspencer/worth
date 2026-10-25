use super::*;
use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordPublication, OrderedJournalPublication,
    PhysicalMutationDeadline,
};

#[test]
fn cold_port_reads_current_members_carried_members_and_bounded_journal_payloads() {
    let directory = tempfile::tempdir().unwrap();
    let serving = Arc::new(
        crate::physical_store::initialize(
            directory.path(),
            crate::physical_store::PhysicalOpening::production(),
        )
        .unwrap(),
    );
    let anchor = AnchoredRecordName::new("relational.checkpoint").unwrap();
    let payload = b"one region's framed bytes".to_vec();
    let key = Sha256::digest(&payload).into();
    let locator = RelationalDurableMemberLocator {
        sha256: key,
        length: payload.len() as u64,
    };
    let publish = |generation, members| {
        serving
            .anchored_records()
            .unwrap()
            .publish(
                AnchoredRecordPublication::new(
                    anchor,
                    generation,
                    vec![generation as u8 + 1],
                    crate::physical_store::placement(),
                    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                )
                .with_members(members),
            )
            .unwrap()
    };
    let first = publish(
        0,
        vec![AnchoredRecordMember::New {
            key,
            bytes: payload.clone(),
        }],
    );
    let source = StoreColdSource::new(serving.clone());
    assert_eq!(source.read_member(locator, 1 << 20).unwrap(), payload);
    assert!(matches!(
        source.read_member(locator, 1),
        Err(Cause::LimitExceeded { limit: 1, .. })
    ));
    let second = publish(
        1,
        vec![AnchoredRecordMember::Carried {
            locator: first.members()[0],
        }],
    );
    assert_eq!(second.members(), first.members());
    assert_eq!(source.read_member(locator, 1 << 20).unwrap(), payload);

    let commit = b"canonical artifact bytes, opaque to the placement adapter".to_vec();
    serving
        .canonical_commits()
        .unwrap()
        .publish(OrderedJournalPublication::new(
            1,
            commit.clone(),
            crate::physical_store::placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
        .unwrap();
    assert_eq!(
        source.read_commit(PatchStreamPosition(1), 1 << 20).unwrap(),
        commit
    );
    assert!(matches!(
        source.read_commit(PatchStreamPosition(1), 1),
        Err(Cause::LimitExceeded { limit: 1, .. })
    ));
    assert_eq!(
        source.read_commit(PatchStreamPosition(2), 1 << 20),
        Err(Cause::SourceUnavailable)
    );
    publish(2, vec![]);
    assert_eq!(
        source.read_member(locator, 1 << 20),
        Err(Cause::SourceUnavailable)
    );
    drop(source);
    let serving = Arc::try_unwrap(serving)
        .ok()
        .expect("all read ports released");
    let _closed = serving.close();
}
