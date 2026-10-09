//! What the record walk saw of the records the selected root routes.

use std::collections::BTreeSet;

use worth_foundational::PhysicalArtifactFamily as Family;

use super::super::blob_record::FrameKind;
use super::super::record_walk::route_inventory::RouteClass;
use super::super::OfflineUnknownPhysicalReason as Unknown;
use super::{BlobRecordWalk, ChildExpectation, ChildScope, Outcome, Selected};

/// Whether the walk visited every record the selected root routes. A record
/// that no selected row answers for is absent from the store only under
/// `Complete`. A walk is cut short only by what hides records it cannot name:
/// - it stopped at the entry bound with expectations still queued;
/// - a routing block was not read intact (unreadable, aliased, damaged,
///   missing, routed twice differently, or holding more children than the
///   walk may queue), so no record below it was visited.
///
/// What hides one record that the walk can name leaves that record `Unread`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Coverage {
    #[default]
    Complete,
    /// Carries what a claim on a record that was not visited is reported as.
    CutShort(Outcome),
}

impl Coverage {
    /// The first cause stands.
    pub(super) fn cut_short(&mut self, cause: &Outcome) {
        if *self != Self::Complete {
            return;
        }
        *self = Self::CutShort(match cause {
            // Damage above a record is not damage of a claim on that record.
            Outcome::Intact | Outcome::Damaged(_) => {
                Outcome::Unknown(Unknown::ParentScopeUnavailable)
            }
            unobserved => unobserved.clone(),
        });
    }

    /// What a claim on a record with no row is reported as, when the walk
    /// cannot say that the record is absent.
    pub(super) fn unvisited(&self) -> Option<&Outcome> {
        match self {
            Self::Complete => None,
            Self::CutShort(outcome) => Some(outcome),
        }
    }
}

/// A routed record whose extent manifest or first frame could not be read.
/// Its route alone says whether it is a blob record and of which kind, so it
/// gets its row once the route inventory is known.
pub(super) struct Unread {
    record: [u8; 24],
    path: String,
    generation: u64,
    outcome: Outcome,
}

impl BlobRecordWalk {
    /// The record walk's outcome for one expectation under the selected root.
    pub(crate) fn note_outcome(&mut self, expected: &ChildExpectation, outcome: &Outcome) {
        if *outcome == Outcome::Intact {
            return;
        }
        match expected.scope {
            ChildScope::ExtentChunk {
                record,
                logical_offset,
                ..
            } => {
                let pending = self.pending.as_mut();
                if let Some(pending) = pending.filter(|pending| pending.record == record) {
                    if pending.interruption.is_none() {
                        pending.interruption = Some(outcome.clone());
                    }
                } else if logical_offset == 0 {
                    self.note_unread(record, expected, outcome);
                }
            }
            ChildScope::ExtentManifest { record, .. } => {
                self.note_unread(record, expected, outcome);
            }
            ChildScope::Tree { .. } if expected.family == Family::RootRoutingBlock => {
                self.coverage.cut_short(outcome);
            }
            ChildScope::Tree { .. } | ChildScope::FreeSpace { .. } | ChildScope::Page { .. } => {}
        }
    }

    /// Unread is not absent: the route names the record.
    fn note_unread(&mut self, record: [u8; 24], expected: &ChildExpectation, outcome: &Outcome) {
        self.unread.push(Unread {
            record,
            path: expected.path.clone(),
            generation: expected.generation,
            outcome: outcome.clone(),
        });
    }

    /// The record walk stopped at `bound` with expectations still queued.
    pub(crate) fn note_walk_stopped(&mut self, bound: &Outcome) {
        self.coverage.cut_short(bound);
    }

    /// Give each unread record its row, unless its route says that it is not a
    /// blob record or the record already has a row.
    pub(super) fn admit_unread(&mut self) {
        if self.unread.is_empty() {
            return;
        }
        let mut rows: BTreeSet<[u8; 24]> = self.selected.iter().map(|row| row.record).collect();
        for unread in std::mem::take(&mut self.unread) {
            let kind = match self.routes.classes.get(&unread.record) {
                Some(RouteClass::Opaque | RouteClass::DerivedDirectory | RouteClass::BTreeNode) => {
                    continue;
                }
                Some(RouteClass::Blob(kind)) => FrameKind::declared(*kind),
                Some(RouteClass::UnknownLegacy) | None => None,
            };
            if rows.insert(unread.record) {
                self.selected.push(Selected {
                    record: unread.record,
                    path: unread.path,
                    generation: unread.generation,
                    kind,
                    fact: None,
                    outcome: unread.outcome,
                    route: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests;
