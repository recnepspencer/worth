//! Persistent name history facts: what a decision's tracked entity or
//! relation name read rested on beyond live record lifecycles.
//!
//! A resolution that walked ended names (which a later `Restore` revives),
//! and a read that found a held name or carrier not yet born, rest on the
//! derived name index's history, which no record lifecycle carries. The fact
//! therefore names the read itself and records its exact answer: the
//! Relational outcome of the recorded name under the exact Relational
//! selection the read resolved, with every lineage that answer rested on, or
//! whether a digest's birth is recorded. A seal re-observes it at the sealing
//! basis, and a committed lineage event that ends, splits, merges, replaces
//! or restores one of those lineages, or births that digest, keys it for
//! re-verification; any difference stales the decision.
//!
//! A query row's projected group name (`#[worth_query(group_name)]`) rests
//! on its member's group membership, which regrouping changes without
//! touching any record: the `MemberGroups` question records every
//! `(group kind, group)` holding the member, so a regrouping of that member
//! moves the fact and an unrelated regrouping leaves it equal.

use worth_relational::facade::identity::LineageId;
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameObservationView, PersistentNameResolution,
    PersistentNameResolutionDenial, PersistentNameSelection, PersistentNameSubjectKind,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

/// One tracked name-history read, admitted under the operation's declared
/// read of `kind` (an entity kind for entity names, a relation for relation
/// names).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PersistentNameHistoryRead {
    pub kind: String,
    pub subject: PersistentNameSubjectKind,
    pub question: PersistentNameHistoryQuestion,
}

/// What the read asked the name index.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PersistentNameHistoryQuestion {
    /// Where the recorded name's meaning lives now under `selection`.
    Resolution {
        held: PersistentNameKey,
        selection: PersistentNameSelection,
    },
    /// Whether a name with this digest has a recorded birth.
    Birth([u8; 32]),
    /// Every `(group kind, group)` holding this entity member name.
    MemberGroups([u8; 32]),
}

/// The exact answer a name-history read observed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PersistentNameHistoryAnswer {
    Resolution {
        result: Result<PersistentNameResolution, PersistentNameResolutionDenial>,
        /// Every lineage the answer rested on, live or ended, ascending.
        rests_on: Vec<LineageId>,
    },
    Birth {
        recorded: bool,
    },
    /// `(group kind, group)` pairs holding the member, ascending.
    MemberGroups(Vec<(String, [u8; 32])>),
}

/// Why a name-history fact could not be observed: the name index refused at
/// the basis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistentNameHistoryUnobservable;

impl PersistentNameHistoryRead {
    /// Observes this read at `snapshot`.
    ///
    /// Complexity: O(lineage path) for a resolution, O(1) expected for a
    /// birth; never O(names) or O(commits).
    pub fn observe_at(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<PersistentNameHistoryAnswer, PersistentNameHistoryUnobservable> {
        runtime
            .with_persistent_names_at_snapshot(snapshot, |view| self.observe_in(runtime, &view))
            .map_err(|_| PersistentNameHistoryUnobservable)
    }

    /// Observes this read in `view`, naming group schemes by the installed
    /// group kind `runtime` records for them.
    pub fn observe_in(
        &self,
        runtime: &RelationalRuntime,
        view: &PersistentNameObservationView<'_>,
    ) -> PersistentNameHistoryAnswer {
        match &self.question {
            PersistentNameHistoryQuestion::MemberGroups(member) => {
                let key = view.recorded_name(PersistentNameSubjectKind::Entity, member);
                let Some(PersistentNameKey::Entity(identity)) = key else {
                    return PersistentNameHistoryAnswer::MemberGroups(Vec::new());
                };
                let mut groups: Vec<(String, [u8; 32])> = view
                    .groups_of_member(&identity)
                    .into_iter()
                    .filter_map(|(scheme, group)| {
                        let (_, kind) = runtime.persistent_name_scheme_subject(scheme)?;
                        Some((kind, *group.digest()))
                    })
                    .collect();
                groups.sort();
                PersistentNameHistoryAnswer::MemberGroups(groups)
            }
            PersistentNameHistoryQuestion::Resolution { held, selection } => {
                let outcome = view.resolve(*held, selection);
                let live = outcome.footprint.live.iter();
                let mut rests_on: Vec<LineageId> = live
                    .filter_map(|target| target.lineage())
                    .chain(outcome.footprint.ended_lineages.iter().copied())
                    .collect();
                rests_on.sort_unstable();
                rests_on.dedup();
                PersistentNameHistoryAnswer::Resolution {
                    result: outcome.result,
                    rests_on,
                }
            }
            PersistentNameHistoryQuestion::Birth(digest) => {
                // Entity and relation births only, exactly what posting keys
                // (`visit_lineage`) and selection lowering address.
                let subjects = [
                    PersistentNameSubjectKind::Entity,
                    PersistentNameSubjectKind::Relation,
                ];
                let recorded = subjects
                    .into_iter()
                    .any(|subject| view.recorded_name(subject, digest).is_some());
                PersistentNameHistoryAnswer::Birth { recorded }
            }
        }
    }

    /// Whether `answer` is still what this read observes at `snapshot`.
    pub fn remains_equal(
        &self,
        answer: &PersistentNameHistoryAnswer,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, PersistentNameHistoryUnobservable> {
        Ok(self.observe_at(runtime, snapshot)? == *answer)
    }

    /// The read's locator: its subject, kind and question digest.
    pub fn write_locator(&self, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
        let (question, digest) = match &self.question {
            PersistentNameHistoryQuestion::Resolution { held, .. } => {
                ("resolution", key_digest(held))
            }
            PersistentNameHistoryQuestion::Birth(digest) => ("birth", *digest),
            PersistentNameHistoryQuestion::MemberGroups(member) => ("member-groups", *member),
        };
        write!(
            output,
            "application-persistent-name-history:{:?}:{}:{question}:",
            self.subject, self.kind
        )?;
        digest
            .iter()
            .try_for_each(|byte| write!(output, "{byte:02x}"))
    }

    /// Heap bytes this read retains.
    pub fn charged_bytes(&self) -> u64 {
        let selection = match &self.question {
            PersistentNameHistoryQuestion::Resolution { selection, .. } => match selection {
                PersistentNameSelection::BoundedByCarriers { carriers } => carriers.len() * 32,
                PersistentNameSelection::KeepSide { discriminator } => discriminator.capacity(),
                PersistentNameSelection::Exact | PersistentNameSelection::AllDescendants => 0,
            },
            PersistentNameHistoryQuestion::Birth(_)
            | PersistentNameHistoryQuestion::MemberGroups(_) => 0,
        };
        u64::try_from(self.kind.capacity().saturating_add(selection)).unwrap_or(u64::MAX)
    }
}

impl PersistentNameHistoryAnswer {
    /// Heap bytes this answer retains.
    pub fn charged_bytes(&self) -> u64 {
        let bytes = match self {
            Self::Resolution { result, rests_on } => {
                let targets = match result {
                    Ok(PersistentNameResolution::Plural(targets)) => targets.capacity(),
                    _ => 0,
                };
                rests_on
                    .capacity()
                    .saturating_add(targets)
                    .saturating_mul(32)
            }
            Self::Birth { .. } => 0,
            Self::MemberGroups(groups) => groups
                .iter()
                .map(|(kind, _)| kind.capacity().saturating_add(32))
                .sum(),
        };
        u64::try_from(bytes).unwrap_or(u64::MAX)
    }

    /// The digests whose birth would change this answer: the held name a
    /// resolution found unknown, or the read digest itself.
    pub fn awaited_birth(&self, read: &PersistentNameHistoryRead) -> Option<[u8; 32]> {
        match (&read.question, self) {
            (PersistentNameHistoryQuestion::Birth(digest), _) => Some(*digest),
            (
                PersistentNameHistoryQuestion::Resolution { held, .. },
                Self::Resolution {
                    result: Err(PersistentNameResolutionDenial::UnknownName { .. }),
                    ..
                },
            ) => Some(key_digest(held)),
            _ => None,
        }
    }

    /// Every lineage this answer rested on.
    pub fn rests_on(&self) -> &[LineageId] {
        match self {
            Self::Resolution { rests_on, .. } => rests_on,
            Self::Birth { .. } | Self::MemberGroups(_) => &[],
        }
    }

    /// Whether committed lineage events alone address every change to this
    /// answer; group membership changes only through group lineage.
    pub const fn keyed_by_lineage(&self) -> bool {
        !matches!(self, Self::MemberGroups(_))
    }
}

fn key_digest(key: &PersistentNameKey) -> [u8; 32] {
    match key {
        PersistentNameKey::Entity(identity) | PersistentNameKey::Relation(identity) => {
            *identity.as_bytes()
        }
        PersistentNameKey::Group(group) => *group.bytes(),
    }
}
