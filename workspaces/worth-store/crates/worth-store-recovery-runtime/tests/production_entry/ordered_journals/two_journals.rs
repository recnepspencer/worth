//! Two ordered journals in one home keep independent contiguous sequences:
//! interleaved publications read, range and resolve per journal, the next
//! position is each journal's own head plus one, neither journal ever answers
//! with the other's entry, and both survive abandoned runtimes.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, OrderedJournal, OrderedJournalName, OrderedJournalNameDenial,
    OrderedJournalPublication, OrderedJournalPublicationFailure, OrderedJournalPublished,
    OrderedJournalResolution, ServingPhysicalRuntime, RELATIONAL_BRANCH_TRANSITIONS,
    RELATIONAL_CANONICAL_COMMITS,
};

use super::canonical_journal_oracle::{placement, serve};
use super::*;

/// One of the home's two ordered journals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Journal {
    Commits,
    Transitions,
}

impl Journal {
    pub(crate) const BOTH: [Self; 2] = [Self::Commits, Self::Transitions];

    pub(crate) fn of(self, serving: &ServingPhysicalRuntime) -> OrderedJournal<'_> {
        match self {
            Self::Commits => serving.canonical_commits(),
            Self::Transitions => serving.branch_transitions(),
        }
        .expect("journal access")
    }

    const fn other(self) -> Self {
        match self {
            Self::Commits => Self::Transitions,
            Self::Transitions => Self::Commits,
        }
    }

    /// Deterministic entry bytes, distinct per journal and position.
    pub(crate) fn payload(self, position: u64) -> Vec<u8> {
        let (seed, extra) = match self {
            Self::Commits => (0x2c_u8, 0),
            Self::Transitions => (0xb7_u8, 17),
        };
        let length = 90 + extra + (position as usize * 733) % 5_000;
        (0..length)
            .map(|index| seed ^ position as u8 ^ (index as u8).rotate_left(3))
            .collect()
    }
}

pub(crate) fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Publishes `journal`'s own entry at `position`.
pub(crate) fn publish(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    journal: Journal,
    position: u64,
) -> Result<OrderedJournalPublished, OrderedJournalPublicationFailure> {
    publish_bytes(
        serving,
        placement,
        journal,
        position,
        journal.payload(position),
    )
}

fn publish_bytes(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    journal: Journal,
    position: u64,
    bytes: Vec<u8>,
) -> Result<OrderedJournalPublished, OrderedJournalPublicationFailure> {
    journal.of(serving).publish(OrderedJournalPublication::new(
        position,
        bytes,
        placement,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    ))
}

/// Requires `journal` to hold exactly its own entries `1..=head`: point
/// reads, `resolve`, range reads and an unoccupied next position. The other
/// journal's entry at each position is a conflict, never this journal's.
pub(crate) fn assert_holds(serving: &ServingPhysicalRuntime, journal: Journal, head: u64) {
    let access = journal.of(serving);
    let observed = access
        .head()
        .expect("head read")
        .map(|entry| entry.position());
    assert_eq!(observed, (head > 0).then_some(head), "{journal:?} head");
    for position in 1..=head {
        let read = access
            .read(position, 1 << 20)
            .unwrap_or_else(|failure| panic!("{journal:?} {position}: {failure:?}"));
        assert_eq!(read.entry().position(), position);
        assert!(
            read.bytes() == journal.payload(position),
            "{journal:?} entry {position} changed"
        );
        assert!(matches!(
            access.resolve(position, digest(&journal.payload(position))).unwrap(),
            OrderedJournalResolution::Committed(entry) if entry.position() == position
        ));
        let foreign = digest(&journal.other().payload(position));
        assert!(matches!(
            access.resolve(position, foreign).unwrap(),
            OrderedJournalResolution::Conflict(_)
        ));
    }
    assert_eq!(
        access
            .resolve(head + 1, digest(&journal.payload(head + 1)))
            .unwrap(),
        OrderedJournalResolution::NotCommitted { head }
    );
    let positions: Vec<_> = access
        .range_descending(u64::MAX, usize::MAX)
        .expect("range read")
        .iter()
        .map(|entry| entry.position())
        .collect();
    assert_eq!(positions, (1..=head).rev().collect::<Vec<_>>());
    if head >= 3 {
        let page: Vec<_> = access
            .range_descending(head - 1, 2)
            .expect("bounded range read")
            .iter()
            .map(|entry| entry.position())
            .collect();
        assert_eq!(page, [head - 1, head - 2]);
    }
}

/// Interleaved publications: neither journal's position moves the other's.
const SCRIPT: [(Journal, u64); 9] = [
    (Journal::Commits, 1),
    (Journal::Transitions, 1),
    (Journal::Commits, 2),
    (Journal::Commits, 3),
    (Journal::Transitions, 2),
    (Journal::Transitions, 3),
    (Journal::Commits, 4),
    (Journal::Transitions, 4),
    (Journal::Transitions, 5),
];

fn publish_script(serving: &ServingPhysicalRuntime, placement: AdmittedRecordPlacementPolicy) {
    for (journal, position) in SCRIPT {
        let published = publish(serving, placement, journal, position)
            .unwrap_or_else(|failure| panic!("{journal:?} {position}: {failure:?}"));
        assert!(matches!(
            published,
            OrderedJournalPublished::Committed(entry) if entry.position() == position
        ));
    }
}

#[test]
fn two_journals_keep_independent_sequences_reads_and_resolutions() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("two-journals")
        .expect("journal store");
    let serving = world.serving();
    for journal in Journal::BOTH {
        assert_holds(serving, journal, 0);
    }
    publish_script(serving, world.placement());
    assert_holds(serving, Journal::Commits, 4);
    assert_holds(serving, Journal::Transitions, 5);

    // Each journal admits only its own head plus one; refusals write nothing.
    let quiet = serving.physical_mutation_observation();
    for (journal, requested, head) in [(Journal::Commits, 6, 4), (Journal::Transitions, 5 + 2, 5)] {
        assert!(matches!(
            publish(serving, world.placement(), journal, requested),
            Err(OrderedJournalPublicationFailure::PositionGap { head: observed, requested: r })
                if observed == head && r == requested
        ));
    }
    assert!(matches!(
        publish(serving, world.placement(), Journal::Commits, 3),
        Ok(OrderedJournalPublished::AlreadyCommitted(entry)) if entry.position() == 3
    ));
    // The other journal's entry at an occupied position is a conflict.
    let foreign = Journal::Commits.payload(2);
    assert!(matches!(
        publish_bytes(serving, world.placement(), Journal::Transitions, 2, foreign),
        Err(OrderedJournalPublicationFailure::PositionConflict(entry)) if entry.position() == 2
    ));
    assert_eq!(
        serving.physical_mutation_observation(),
        quiet,
        "refusals wrote"
    );

    // The named door reaches the same journals as the named accessors.
    for (text, journal) in [
        (RELATIONAL_CANONICAL_COMMITS, Journal::Commits),
        (RELATIONAL_BRANCH_TRANSITIONS, Journal::Transitions),
    ] {
        let name = OrderedJournalName::new(text).expect("a registered journal");
        let by_name = serving.ordered_journal(name).expect("journal access");
        assert_eq!(by_name.name().as_str(), text);
        assert_eq!(by_name.head().unwrap(), journal.of(serving).head().unwrap());
    }
    assert_eq!(
        OrderedJournalName::new("relational.home"),
        Err(OrderedJournalNameDenial::Unregistered)
    );
    assert_eq!(
        OrderedJournalName::new("Relational Commits"),
        Err(OrderedJournalNameDenial::Malformed)
    );
}

/// Both journals recover from an abandoned runtime, keep publishing, and
/// recover again from a second abandoned runtime.
#[test]
fn two_journals_survive_abandoned_runtimes_twice() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("two-journals-abandoned")
        .expect("journal store");
    publish_script(world.serving(), world.placement());
    let retained = world.retained_root();
    drop(world);

    let recovered = serve(retained.path(), "first abandoned runtime");
    assert_holds(&recovered, Journal::Commits, 4);
    assert_holds(&recovered, Journal::Transitions, 5);
    for (journal, position) in [(Journal::Transitions, 6), (Journal::Commits, 5)] {
        publish(&recovered, placement(), journal, position).expect("publication after recovery");
    }
    let _ = recovered.abort();

    let again = serve(retained.path(), "second abandoned runtime");
    assert_holds(&again, Journal::Commits, 5);
    assert_holds(&again, Journal::Transitions, 6);
    for (journal, position) in [(Journal::Commits, 6), (Journal::Transitions, 7)] {
        publish(&again, placement(), journal, position).expect("publication after recovery");
    }
    let _ = again.close();

    let reopened = serve(retained.path(), "clean reopen");
    assert_holds(&reopened, Journal::Commits, 6);
    assert_holds(&reopened, Journal::Transitions, 7);
    let _ = reopened.close();
}
