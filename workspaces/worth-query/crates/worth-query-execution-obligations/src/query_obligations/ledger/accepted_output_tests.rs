use super::*;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableWorldOwnerIdentity, HomeIdentity,
    WorldOwnerSlotIdentity,
};

fn accepted_row(
    owner: DurableWorldOwnerIdentity,
    ordinal: u64,
    payload: &[u8],
) -> QueryAcceptedOutputRecord {
    QueryAcceptedOutputRecord {
        commit: DurableCompositeCommitIdentity::from_recorded(owner, ordinal),
        branch: "main".to_owned(),
        producer: "producer.a".to_owned(),
        scope: (1, 2, 3),
        partition: [7; 32],
        payload: payload.to_vec(),
    }
}

#[test]
fn delayed_older_accepted_output_cannot_overwrite_newer_same_slot_commit() {
    let home = HomeIdentity::from_observed_bytes([0x11; 16]).unwrap();
    let owner =
        DurableWorldOwnerIdentity::from_recorded(home, WorldOwnerSlotIdentity::from_recorded(1));
    let mut ledger = QueryObligationLedger::empty(home);
    let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);

    let newer = accepted_row(owner, 5, b"commit-5");
    let frame5 = ledger.prepare(newer.clone(), &mut generations).unwrap();
    ledger.readmit(frame5).unwrap();

    // Delayed older commit for the same slot must not regress the ledger.
    let older = accepted_row(owner, 3, b"commit-3");
    let frame3 = ledger.prepare(older, &mut generations).unwrap();
    ledger.readmit(frame3).unwrap();

    let key = ("main".to_owned(), "producer.a".to_owned(), (1, 2, 3), [7; 32]);
    let retained = ledger
        .output
        .accepted
        .get(&key)
        .expect("slot remains present");
    assert_eq!(retained.commit.ordinal(), 5);
    assert_eq!(retained.payload, b"commit-5");

    // A strictly newer commit for the same slot advances the ledger.
    let newest = accepted_row(owner, 7, b"commit-7");
    let frame7 = ledger.prepare(newest, &mut generations).unwrap();
    ledger.readmit(frame7).unwrap();

    let retained = ledger
        .output
        .accepted
        .get(&key)
        .expect("slot remains present");
    assert_eq!(retained.commit.ordinal(), 7);
    assert_eq!(retained.payload, b"commit-7");
}
