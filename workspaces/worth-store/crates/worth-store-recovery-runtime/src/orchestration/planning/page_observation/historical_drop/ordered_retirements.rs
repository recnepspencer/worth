//! Store's own reading of a WAL target that ordered retirements emptied.
//!
//! Physics mints the historical-retired witness. Before asking for it, Store
//! reads the same admitted members and verified history itself: every record
//! the target's last image placed must be unrouted under the selected root and
//! named by the derived-directory retirement of an ordered ordinary edge.
//! The members and the history are read once; every target is then a lookup.

use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PersistedPhysicalDataFrameSubject,
    PersistedPhysicalRecoveryOperation, PersistedPhysicalRecoveryProjection,
    PersistedRecordIdentity, RecordFrameCoordinate,
};
use worth_store_recovery_physics::{
    AdmittedPhysicalRedoMembers, PhysicalRedoGroupBinding, PhysicalRedoTarget,
    PhysicalRedoTargetIdentity, RecoveryOperationFate, VerifiedOrderedRootEdge,
    VerifiedOrderedRootHistory,
};

use super::target_record;

#[cfg(test)]
#[path = "ordered_retirements/tests.rs"]
mod tests;

/// The exact C.9 member identity an ordered root step binds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct StepIdentity {
    pub(super) operation: [u8; 32],
    pub(super) group: PhysicalRedoGroupBinding,
    pub(super) fate: RecoveryOperationFate,
    pub(super) redo_sha256: [u8; 32],
}

/// One image of an inline page: its page generation and frame coordinate.
type InlineImage = (u64, u64, u64, RecordFrameCoordinate);

/// The admitted members that wrote one inline image. Only a single writer
/// says which records the image holds.
#[derive(Debug)]
enum Writer {
    One {
        member: usize,
        records: Vec<PersistedRecordIdentity>,
    },
    Several,
}

/// What the selected root routes, what the ordinary edges of the ordered
/// history retired, and what every admitted member placed on the inline
/// images it wrote.
#[derive(Debug, Default)]
pub(super) struct OrderedRetirements {
    routed: BTreeSet<PersistedRecordIdentity>,
    retired: BTreeSet<PersistedRecordIdentity>,
    images: BTreeMap<InlineImage, Writer>,
}

impl OrderedRetirements {
    pub(super) fn of(
        redo: &AdmittedPhysicalRedoMembers,
        history: &VerifiedOrderedRootHistory,
        routes: &[CurrentPhysicalRecordPlacement],
    ) -> Self {
        Self::index(
            redo.admitted_root_step_members().map(|member| {
                (
                    StepIdentity {
                        operation: member.operation(),
                        group: member.group(),
                        fate: member.fate(),
                        redo_sha256: member.canonical_redo_sha256(),
                    },
                    member.materialization(),
                )
            }),
            history.edges().iter().filter_map(|edge| match edge {
                VerifiedOrderedRootEdge::Ordinary(step) => Some(StepIdentity {
                    operation: step.operation(),
                    group: step.group(),
                    fate: step.fate(),
                    redo_sha256: step.redo_sha256(),
                }),
                _ => None,
            }),
            routes.iter().map(|route| route.record()),
        )
    }

    /// A member retires records only when an ordinary edge binds its exact
    /// identity; every member, bound or not, counts as a writer of the images
    /// it framed.
    pub(super) fn index<'a>(
        members: impl Iterator<Item = (StepIdentity, &'a PersistedPhysicalRecoveryProjection)>,
        ordinary_edges: impl Iterator<Item = StepIdentity>,
        routes: impl Iterator<Item = PersistedRecordIdentity>,
    ) -> Self {
        let mut ordinary = BTreeMap::<[u8; 32], Vec<StepIdentity>>::new();
        for edge in ordinary_edges {
            ordinary.entry(edge.operation).or_default().push(edge);
        }
        let mut index = Self {
            routed: routes.collect(),
            ..Self::default()
        };
        for (member, (identity, projection)) in members.enumerate() {
            index.write_images(member, projection);
            let bound = ordinary
                .get(&identity.operation)
                .is_some_and(|edges| edges.contains(&identity));
            if let (
                true,
                PersistedPhysicalRecoveryOperation::DerivedDirectory {
                    retirement: Some(retirement),
                    ..
                },
            ) = (bound, projection.operation())
            {
                index
                    .retired
                    .extend(retirement.dropped_records().iter().copied());
            }
        }
        index
    }

    fn write_images(&mut self, member: usize, projection: &PersistedPhysicalRecoveryProjection) {
        for frame in projection.frames().into_iter().flatten() {
            let PersistedPhysicalDataFrameSubject::InlinePage(cell) = frame.subject() else {
                continue;
            };
            let (segment, page, generation) = (
                cell.segment_id().get(),
                cell.page_id().get(),
                cell.generation().get(),
            );
            let records = projection
                .placements()
                .iter()
                .filter_map(|placement| match placement {
                    CurrentPhysicalRecordPlacement::Inline(inline)
                        if inline.segment().get() == segment
                            && inline.page().get() == page
                            && inline.page_generation() == generation =>
                    {
                        Some(inline.record())
                    }
                    _ => None,
                })
                .collect();
            self.images
                .entry((segment, page, generation, frame.coordinate()))
                .and_modify(|writer| {
                    if !matches!(writer, Writer::One { member: first, .. } if *first == member) {
                        *writer = Writer::Several;
                    }
                })
                .or_insert(Writer::One { member, records });
        }
    }

    /// Whether every record this target's image holds is unrouted under the
    /// selected root and retired by an ordered ordinary edge. An extent chunk
    /// holds its one record; an inline image holds what its single writer
    /// placed on it. An image that holds nothing proves no retirement.
    pub(super) fn emptied(&self, target: &PhysicalRedoTarget) -> bool {
        let retired = |record: &PersistedRecordIdentity| {
            !self.routed.contains(record) && self.retired.contains(record)
        };
        let PhysicalRedoTargetIdentity::InlinePage {
            segment,
            page,
            generation,
        } = target.identity()
        else {
            return target_record(target).is_some_and(|record| retired(&record));
        };
        let Some(coordinate) = RecordFrameCoordinate::new(
            target.artifact(),
            target.artifact_offset(),
            target.artifact_length(),
        ) else {
            return false;
        };
        match self.images.get(&(segment, page, generation, coordinate)) {
            Some(Writer::One { records, .. }) => !records.is_empty() && records.iter().all(retired),
            Some(Writer::Several) | None => false,
        }
    }
}
