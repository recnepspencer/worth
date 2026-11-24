//! Persistent group facts: what a decision's tracked group read rested on.
//!
//! A group read rests on the derived name index's group state, which no
//! record lifecycle tracks: regrouping live members changes no record. The
//! fact therefore names the read itself and records its exact answer, and
//! every seal re-reads it at the sealing basis; any difference stales the
//! decision.

/// One tracked group read of the installed group kind `group_kind`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PersistentGroupRead {
    pub group_kind: String,
    /// The entity kind whose member names the read rests on; the read is
    /// admitted under the operation's declared read of that kind.
    pub member_kind: String,
    pub subject: PersistentGroupSubject,
}

/// What the read asked about.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PersistentGroupSubject {
    /// Which live group of the kind holds this member name.
    Member([u8; 32]),
    /// Where this held group name's meaning lives now.
    Resolution([u8; 32]),
}

/// The exact answer a group read observed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PersistentGroupAnswer {
    /// The group holding the member, or none.
    Member(Option<[u8; 32]>),
    /// Every live group carrying the held name's meaning, each with its
    /// member names, in canonical order.
    Live(Vec<([u8; 32], Vec<[u8; 32]>)>),
    /// The held name and every successor retired by this commit.
    Retired(u64),
    /// The held name has no birth visible at the basis.
    Unknown,
}

use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameObservationView, PersistentNameResolution,
    PersistentNameResolutionDenial, PersistentNameSelection, PersistentNameSubjectKind,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

/// Why a group fact could not be observed: its kind has no installed group
/// scheme, or the name index refused at the basis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistentGroupUnobservable;

impl PersistentGroupRead {
    /// Observes this read at `snapshot`, the head of its branch.
    ///
    /// Complexity: O(group lineage path), never O(names) or O(commits).
    pub fn observe_at(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<PersistentGroupAnswer, PersistentGroupUnobservable> {
        let scheme = runtime
            .persistent_group_scheme_id(&self.group_kind)
            .ok_or(PersistentGroupUnobservable)?;
        runtime
            .with_persistent_names_at_snapshot(snapshot, |view| self.observe_in(&view, &scheme))
            .map_err(|_| PersistentGroupUnobservable)?
    }

    /// Observes this read in `view` under the kind's installed `scheme`.
    pub fn observe_in(
        &self,
        view: &PersistentNameObservationView<'_>,
        scheme: &str,
    ) -> Result<PersistentGroupAnswer, PersistentGroupUnobservable> {
        match self.subject {
            PersistentGroupSubject::Member(member) => {
                let group = match view.recorded_name(PersistentNameSubjectKind::Entity, &member) {
                    Some(PersistentNameKey::Entity(identity)) => view
                        .group_of_member(scheme, &identity)
                        .map(|group| *group.digest()),
                    _ => None,
                };
                Ok(PersistentGroupAnswer::Member(group))
            }
            PersistentGroupSubject::Resolution(held) => {
                let Some(key) = view.recorded_name(PersistentNameSubjectKind::Group, &held) else {
                    return Ok(PersistentGroupAnswer::Unknown);
                };
                let selection = PersistentNameSelection::AllDescendants;
                let live = match view.resolve(key, &selection).result {
                    Ok(PersistentNameResolution::Retired { at_commit }) => {
                        return Ok(PersistentGroupAnswer::Retired(at_commit.0));
                    }
                    Ok(PersistentNameResolution::Exact(target)) => vec![target.key()],
                    Ok(PersistentNameResolution::Plural(targets)) => {
                        targets.iter().map(|target| target.key()).collect()
                    }
                    Err(PersistentNameResolutionDenial::UnknownName { .. }) => {
                        return Ok(PersistentGroupAnswer::Unknown);
                    }
                    Err(_) => return Err(PersistentGroupUnobservable),
                };
                let mut groups = Vec::with_capacity(live.len());
                for group in live {
                    let mut members: Vec<[u8; 32]> = view
                        .group_members(group)
                        .ok_or(PersistentGroupUnobservable)?
                        .iter()
                        .map(|member| *member.as_bytes())
                        .collect();
                    members.sort_unstable();
                    groups.push((*group.digest(), members));
                }
                groups.sort();
                Ok(PersistentGroupAnswer::Live(groups))
            }
        }
    }

    /// Whether `answer` is still what this read observes at `snapshot`.
    pub fn remains_equal(
        &self,
        answer: &PersistentGroupAnswer,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, PersistentGroupUnobservable> {
        Ok(self.observe_at(runtime, snapshot)? == *answer)
    }

    /// The read's locator: its kind and subject digest.
    pub fn write_locator(&self, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let (subject, digest) = match self.subject {
            PersistentGroupSubject::Member(member) => ("member", member),
            PersistentGroupSubject::Resolution(held) => ("resolution", held),
        };
        write!(
            output,
            "application-persistent-group:{}:{}:{subject}:",
            self.group_kind, self.member_kind
        )?;
        digest
            .iter()
            .try_for_each(|byte| write!(output, "{byte:02x}"))
    }
}

impl PersistentGroupAnswer {
    /// Heap bytes this answer retains.
    pub fn charged_bytes(&self) -> u64 {
        let digests = match self {
            Self::Live(groups) => groups
                .iter()
                .map(|(_, members)| members.capacity().saturating_add(1))
                .sum::<usize>(),
            Self::Member(_) | Self::Retired(_) | Self::Unknown => 0,
        };
        u64::try_from(digests.saturating_mul(32)).unwrap_or(u64::MAX)
    }
}
