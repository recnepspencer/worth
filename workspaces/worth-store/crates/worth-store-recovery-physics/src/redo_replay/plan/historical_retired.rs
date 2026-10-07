//! An older WAL image whose record a later verified root edge removed.
//!
//! A V3 drop's replacement directory frame, for example, can be superseded by
//! a later derived-directory retirement before the selected root. The target
//! is then neither routed nor below-frontier absent: the ordered history
//! proves its member published the record and a later edge removed it.
//! An inline page is retired when that holds for every record on it.
//! Released manifest drops keep their own selected-control witness.
//!
//! The history is read once into [`HistoricalRetirements`]; every target of
//! one classification is then a lookup.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PersistedPhysicalRecoveryOperation, PersistedRecordIdentity,
    RecordFrameCoordinate,
};

use super::historical_drop::{
    history_anchors_selection, target_record, unique_indeterminate_target_operation,
};
use super::supersession::{page_claims, PageClaims};
use super::*;
use crate::{
    HistoricalRetiredTargetWitness, PhysicalSourceSelection, VerifiedOrderedRootEdge,
    VerifiedOrderedRootHistory,
};

#[path = "historical_retired/inline_page.rs"]
mod inline_page;

impl AdmittedPhysicalRedoMembers {
    /// What the edges of the ordered history wrote and retired, and what the
    /// selected root it must end at still routes.
    pub fn historical_retirements<'a>(
        &'a self,
        selection: &PhysicalSourceSelection,
        history: &VerifiedOrderedRootHistory,
    ) -> HistoricalRetirements<'a> {
        HistoricalRetirements::index(
            &self.members,
            history
                .edges()
                .iter()
                .filter_map(EdgeIdentity::of)
                .collect(),
            history_anchors_selection(history, selection),
            selection
                .page_facts()
                .placements()
                .iter()
                .map(|route| route.record()),
        )
    }
}

/// The admitted members bound to the edges of one ordered history, and the
/// records its ordinary edges retired.
#[derive(Debug)]
pub struct HistoricalRetirements<'a> {
    members: &'a [AdmittedPhysicalRedoMember],
    edges: Vec<EdgeIdentity>,
    /// The selected root identity, only when the history is anchored on it.
    anchored: Option<[u8; 32]>,
    routed: BTreeSet<PersistedRecordIdentity>,
    by_operation: BTreeMap<[u8; 32], Vec<usize>>,
    /// Per admitted member, the ordered edge bound to it.
    written: Vec<Bound>,
    /// Per record, the ordinary edge whose admitted derived-directory
    /// retirement drops it.
    retiring: BTreeMap<PersistedRecordIdentity, Bound>,
    /// Every admitted inline image, unless two members claim one.
    claims: Option<PageClaims<'a>>,
}

/// The ordered edges that bind one member or retire one record. Only exactly
/// one is a proof: none proves nothing and several are ambiguous.
#[derive(Debug, Clone, Copy, Default)]
enum Bound {
    #[default]
    None,
    One(usize),
    Many,
}

impl Bound {
    fn bind(&mut self, position: usize) {
        *self = match self {
            Self::None => Self::One(position),
            Self::One(_) | Self::Many => Self::Many,
        };
    }

    const fn unique(self) -> Option<usize> {
        match self {
            Self::One(position) => Some(position),
            Self::None | Self::Many => None,
        }
    }
}

impl<'a> HistoricalRetirements<'a> {
    pub(super) fn index(
        members: &'a [AdmittedPhysicalRedoMember],
        edges: Vec<EdgeIdentity>,
        anchored: Option<[u8; 32]>,
        selected_routes: impl Iterator<Item = PersistedRecordIdentity>,
    ) -> Self {
        let mut by_operation = BTreeMap::<[u8; 32], Vec<usize>>::new();
        for (index, member) in members.iter().enumerate() {
            by_operation
                .entry(member.operation)
                .or_default()
                .push(index);
        }
        let mut written = vec![Bound::None; members.len()];
        let mut retiring = BTreeMap::<PersistedRecordIdentity, Bound>::new();
        for (position, edge) in edges.iter().enumerate() {
            let Some(member) = edge_member(members, &by_operation, *edge) else {
                continue;
            };
            written[member].bind(position);
            let PersistedPhysicalRecoveryOperation::DerivedDirectory {
                retirement: Some(retirement),
                ..
            } = members[member].projection.operation()
            else {
                continue;
            };
            if edge.ordinary {
                for record in retirement.dropped_records() {
                    retiring.entry(*record).or_default().bind(position);
                }
            }
        }
        Self {
            members,
            edges,
            anchored,
            routed: selected_routes.collect(),
            by_operation,
            written,
            retiring,
            claims: page_claims(members).ok(),
        }
    }

    /// Requires the target's own member to be an ordered edge that placed the
    /// exact extent record, exactly one later ordinary edge whose admitted
    /// derived-directory retirement drops that record (the step itself proved
    /// the record was routed in its source and absent from its result), and a
    /// selected root that does not route it. An inline target must be the
    /// last image of a page on which every record meets that rule.
    pub fn admit(&self, target: &PhysicalRedoTarget) -> Option<HistoricalRetiredTargetWitness> {
        let old_operation = unique_indeterminate_target_operation(self.members, target)?;
        let (selected_root_identity, retiring_operation) = match target.identity() {
            PhysicalRedoTargetIdentity::ExtentChunk {
                extent, generation, ..
            } => {
                let record = target_record(target)?;
                (
                    self.unrouted(record)?,
                    self.retiring_operation(old_operation, record, (extent, generation))?,
                )
            }
            PhysicalRedoTargetIdentity::InlinePage { .. } => self.retired_page(target)?,
        };
        Some(HistoricalRetiredTargetWitness {
            selected_root_identity,
            retiring_operation,
            old_operation,
            target: target.identity(),
            wal_target_digest: target.resulting_digest(),
            target_coordinate: RecordFrameCoordinate::new(
                target.artifact(),
                target.artifact_offset(),
                target.artifact_length(),
            )?,
        })
    }

    /// The anchored selected root identity, only when that root does not
    /// route the record: a still-routed record was never retired.
    pub(super) fn unrouted(&self, record: PersistedRecordIdentity) -> Option<[u8; 32]> {
        let identity = self.anchored?;
        (!self.routed.contains(&record)).then_some(identity)
    }

    /// The operation of the unique ordinary edge, ordered after the unique
    /// edge of `old_operation` that placed this exact extent record, whose
    /// admitted derived-directory retirement drops that record.
    pub(super) fn retiring_operation(
        &self,
        old_operation: [u8; 32],
        record: PersistedRecordIdentity,
        (extent, generation): (u64, u64),
    ) -> Option<[u8; 32]> {
        let mut published = Bound::None;
        for member in self.by_operation.get(&old_operation)? {
            let places = self.members[*member]
                .projection
                .placements()
                .iter()
                .any(|placement| {
                    matches!(placement, CurrentPhysicalRecordPlacement::Extent(placed)
                        if placed.record() == record
                            && placed.extent().get() == extent
                            && placed.extent_generation() == generation)
                });
            match self.written[*member] {
                Bound::One(position) if places => published.bind(position),
                Bound::Many if places => published = Bound::Many,
                Bound::None | Bound::One(_) | Bound::Many => {}
            }
        }
        let published = published.unique()?;
        let retired = self.retiring.get(&record)?.unique()?;
        (retired > published).then(|| self.edges[retired].operation)
    }
}

/// The unique admitted member bound to this edge's exact C.9 identity.
fn edge_member(
    members: &[AdmittedPhysicalRedoMember],
    by_operation: &BTreeMap<[u8; 32], Vec<usize>>,
    edge: EdgeIdentity,
) -> Option<usize> {
    let mut bound = by_operation
        .get(&edge.operation)?
        .iter()
        .copied()
        .filter(|member| {
            let member = &members[*member];
            member.group == edge.group
                && member.fate == edge.fate
                && member.canonical_redo_sha256 == edge.redo_sha256
        });
    let member = bound.next()?;
    bound.next().is_none().then_some(member)
}

/// The exact C.9 member identity an ordered root edge binds.
#[derive(Debug, Clone, Copy)]
pub(super) struct EdgeIdentity {
    pub(super) operation: [u8; 32],
    pub(super) group: PhysicalRedoGroupBinding,
    pub(super) fate: RecoveryOperationFate,
    pub(super) redo_sha256: [u8; 32],
    pub(super) ordinary: bool,
}

impl EdgeIdentity {
    /// A retirement edge binds no C.9 member, so it has no identity here.
    fn of(edge: &VerifiedOrderedRootEdge) -> Option<Self> {
        Some(match edge {
            VerifiedOrderedRootEdge::Ordinary(step) => Self {
                operation: step.operation(),
                group: step.group(),
                fate: step.fate(),
                redo_sha256: step.redo_sha256(),
                ordinary: true,
            },
            VerifiedOrderedRootEdge::Released(edge) => Self {
                operation: edge.operation(),
                group: edge.group(),
                fate: edge.fate(),
                redo_sha256: edge.redo_sha256(),
                ordinary: false,
            },
            VerifiedOrderedRootEdge::Retirement(_) => return None,
        })
    }
}
