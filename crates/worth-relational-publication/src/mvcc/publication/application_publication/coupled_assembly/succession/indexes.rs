//! Which live member wrote, and which names, each record of a coupled
//! candidate (#119). Every member's entries are replaceable: a superseded
//! successor round's are removed when its chain retracts, so no superseded
//! effect can ever cause a refusal.
use contracts::transactions::data::primitives::RecordRef;
use std::collections::BTreeMap;
use worth_relational_contracts::internal_port as contracts;

/// One member of the candidate.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemberId {
    Prefix,
    Stage(usize),
    Part(usize),
}

#[derive(Default)]
struct MemberEntries {
    ordinal: u64,
    writes: Vec<RecordRef>,
    references: Vec<RecordRef>,
}

/// The candidate's live write and reference indexes. Each member's ordinal is
/// its physical append position: the authored prefix is 0 and every append,
/// a successor round included, takes the next.
#[derive(Default)]
pub struct MemberIndexes {
    writes: BTreeMap<RecordRef, BTreeMap<MemberId, u64>>,
    references: BTreeMap<RecordRef, BTreeMap<MemberId, u32>>,
    members: BTreeMap<MemberId, MemberEntries>,
    next_ordinal: u64,
}

impl MemberIndexes {
    /// The ordinal the next appended member takes.
    pub fn next_ordinal(&self) -> u64 {
        self.next_ordinal
    }

    /// Install `member`'s live writes and references at the next ordinal,
    /// replacing any entries it had. O((writes + references) · log n).
    pub fn insert(
        &mut self,
        member: MemberId,
        writes: &[RecordRef],
        references: impl IntoIterator<Item = RecordRef>,
    ) -> u64 {
        self.remove(member);
        let ordinal = self.next_ordinal;
        self.next_ordinal += 1;
        let mut entries = MemberEntries {
            ordinal,
            writes: writes.to_vec(),
            references: references.into_iter().collect(),
        };
        entries.writes.sort_unstable();
        entries.writes.dedup();
        for record in &entries.writes {
            self.writes
                .entry(record.clone())
                .or_default()
                .insert(member, ordinal);
        }
        for record in &entries.references {
            *self
                .references
                .entry(record.clone())
                .or_default()
                .entry(member)
                .or_default() += 1;
        }
        self.members.insert(member, entries);
        ordinal
    }

    /// The ordinal of `member`'s live entries.
    pub fn ordinal(&self, member: MemberId) -> Option<u64> {
        self.members.get(&member).map(|entries| entries.ordinal)
    }

    /// Remove every entry of `member`. O((writes + references) · log n).
    pub fn remove(&mut self, member: MemberId) {
        let Some(entries) = self.members.remove(&member) else {
            return;
        };
        for record in &entries.writes {
            if let Some(writers) = self.writes.get_mut(record) {
                writers.remove(&member);
                if writers.is_empty() {
                    self.writes.remove(record);
                }
            }
        }
        for record in &entries.references {
            if let Some(holders) = self.references.get_mut(record) {
                holders.remove(&member);
                if holders.is_empty() {
                    self.references.remove(record);
                }
            }
        }
    }

    /// Whether a live member other than `member`, appended after `ordinal`,
    /// wrote `record`. O(log n + writers).
    pub fn written_after(&self, record: &RecordRef, member: MemberId, ordinal: u64) -> bool {
        self.writes.get(record).is_some_and(|writers| {
            writers
                .iter()
                .any(|(writer, at)| *writer != member && *at > ordinal)
        })
    }

    /// Whether a live member other than `member` names `record`. O(log n).
    pub fn referenced_by_other(&self, record: &RecordRef, member: MemberId) -> bool {
        self.references
            .get(record)
            .is_some_and(|holders| holders.keys().any(|holder| *holder != member))
    }

    /// Every live member's writes other than `member`'s, for the anchors a
    /// retraction must leave stamped.
    pub fn other_writes(&self, member: MemberId) -> impl Iterator<Item = &RecordRef> {
        self.members
            .iter()
            .filter(move |(id, _)| **id != member)
            .flat_map(|(_, entries)| entries.writes.iter())
    }
}
