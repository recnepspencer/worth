//! Two ordered journals in one home keep independent contiguous sequences:
//! interleaved publications read, range and resolve per journal, the next
//! position is each journal's own head plus one, neither journal ever answers
//! with the other's entry, and both survive abandoned runtimes.

pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {worth_store::physical_runtime::AdmittedRecordPlacementPolicy, worth_store::physical_runtime::OrderedJournal, worth_store::physical_runtime::OrderedJournalName, worth_store::physical_runtime::OrderedJournalNameDenial, worth_store::physical_runtime::OrderedJournalPublication, worth_store::physical_runtime::OrderedJournalPublicationFailure, worth_store::physical_runtime::OrderedJournalPublished, worth_store::physical_runtime::OrderedJournalResolution, worth_store::physical_runtime::ServingPhysicalRuntime, worth_store::physical_runtime::RELATIONAL_BRANCH_TRANSITIONS, worth_store::physical_runtime::RELATIONAL_CANONICAL_COMMITS};

pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
pub(crate) use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

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

    pub(crate) const fn other(self) -> Self {
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
    sha2::Sha256::digest(bytes).into()
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

pub(crate) fn publish_bytes(
    serving: &ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    journal: Journal,
    position: u64,
    bytes: Vec<u8>,
) -> Result<OrderedJournalPublished, OrderedJournalPublicationFailure> {
    journal.of(serving).publish(worth_store::physical_runtime::OrderedJournalPublication::new(
        position,
        bytes,
        placement,
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
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
            worth_store::physical_runtime::OrderedJournalResolution::Committed(entry) if entry.position() == position
        ));
        let foreign = digest(&journal.other().payload(position));
        assert!(matches!(
            access.resolve(position, foreign).unwrap(),
            worth_store::physical_runtime::OrderedJournalResolution::Conflict(_)
        ));
    }
    assert_eq!(
        access
            .resolve(head + 1, digest(&journal.payload(head + 1)))
            .unwrap(),
        worth_store::physical_runtime::OrderedJournalResolution::NotCommitted { head }
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
pub(crate) const SCRIPT: [(Journal, u64); 9] = [
    (crate::production_entry::ordered_journals::two_journals::Journal::Commits, 1),
    (crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 1),
    (crate::production_entry::ordered_journals::two_journals::Journal::Commits, 2),
    (crate::production_entry::ordered_journals::two_journals::Journal::Commits, 3),
    (crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 2),
    (crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 3),
    (crate::production_entry::ordered_journals::two_journals::Journal::Commits, 4),
    (crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 4),
    (crate::production_entry::ordered_journals::two_journals::Journal::Transitions, 5),
];

pub(crate) fn publish_script(serving: &ServingPhysicalRuntime, placement: AdmittedRecordPlacementPolicy) {
    for (journal, position) in SCRIPT {
        let published = publish(serving, placement, journal, position)
            .unwrap_or_else(|failure| panic!("{journal:?} {position}: {failure:?}"));
        assert!(matches!(
            published,
            worth_store::physical_runtime::OrderedJournalPublished::Committed(entry) if entry.position() == position
        ));
    }
}