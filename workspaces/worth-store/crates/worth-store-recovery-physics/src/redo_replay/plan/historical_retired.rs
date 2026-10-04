//! An older WAL image whose record a later verified root edge removed.
//!
//! A V3 drop's replacement directory frame, for example, can be superseded by
//! a later derived-directory retirement before the selected root. The target
//! is then neither routed nor below-frontier absent: the ordered history
//! proves its member published the record and a later edge removed it.
//! Released manifest drops keep their own selected-control witness.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PersistedPhysicalRecoveryOperation, PersistedRecordIdentity,
    RecordFrameCoordinate,
};

use super::historical_drop::{
    history_anchors_selection, target_record, unique_indeterminate_target_operation,
};
use super::*;
use crate::{
    HistoricalRetiredTargetWitness, PhysicalSourceSelection, VerifiedOrderedRootEdge,
    VerifiedOrderedRootHistory,
};

impl AdmittedPhysicalRedoMembers {
    /// Requires the target's own member to be an ordered edge that placed the
    /// exact extent record, exactly one later ordinary edge whose admitted
    /// derived-directory retirement drops that record (the step itself proved
    /// the record was routed in its source and absent from its result), and a
    /// selected root that does not route it.
    pub fn admit_historical_retired_target_with_ordered_history(
        &self,
        selection: &PhysicalSourceSelection,
        target: &PhysicalRedoTarget,
        history: &VerifiedOrderedRootHistory,
    ) -> Option<HistoricalRetiredTargetWitness> {
        let old_operation = unique_indeterminate_target_operation(&self.members, target)?;
        let record = target_record(target)?;
        let selected_root_identity = unrouted_under_anchored_root(
            history_anchors_selection(history, selection),
            selection
                .page_facts()
                .placements()
                .iter()
                .map(|route| route.record()),
            record,
        )?;
        let PhysicalRedoTargetIdentity::ExtentChunk {
            extent, generation, ..
        } = target.identity()
        else {
            return None;
        };
        let retiring_operation = self.retiring_operation(
            old_operation,
            record,
            (extent, generation),
            history.edges().iter().filter_map(EdgeIdentity::of),
        )?;
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

    /// The operation of the unique ordinary edge, ordered after the unique
    /// edge of `old_operation` that placed this exact extent record, whose
    /// admitted derived-directory retirement drops that record.
    pub(super) fn retiring_operation(
        &self,
        old_operation: [u8; 32],
        record: PersistedRecordIdentity,
        (extent, generation): (u64, u64),
        edges: impl Iterator<Item = EdgeIdentity> + Clone,
    ) -> Option<[u8; 32]> {
        let mut publishing = edges.clone().enumerate().filter(|(_, edge)| {
            self.edge_member(*edge).is_some_and(|member| {
                member.operation == old_operation
                    && member.projection.placements().iter().any(|placement| {
                        matches!(placement, CurrentPhysicalRecordPlacement::Extent(placed)
                            if placed.record() == record
                                && placed.extent().get() == extent
                                && placed.extent_generation() == generation)
                    })
            })
        });
        let (published, _) = publishing.next()?;
        if publishing.next().is_some() {
            return None;
        }
        let mut retiring = edges
            .enumerate()
            .filter(|(_, edge)| edge.ordinary && self.edge_retires(*edge, record));
        let (retired, edge) = retiring.next()?;
        if retiring.next().is_some() || retired <= published {
            return None;
        }
        Some(edge.operation)
    }

    /// The unique admitted member bound to this edge's exact C.9 identity.
    fn edge_member(&self, edge: EdgeIdentity) -> Option<&AdmittedPhysicalRedoMember> {
        let mut members = self.members.iter().filter(|member| {
            member.operation == edge.operation
                && member.group == edge.group
                && member.fate == edge.fate
                && member.canonical_redo_sha256 == edge.redo_sha256
        });
        let member = members.next()?;
        members.next().is_none().then_some(member)
    }

    fn edge_retires(&self, edge: EdgeIdentity, record: PersistedRecordIdentity) -> bool {
        self.edge_member(edge).is_some_and(|member| {
            matches!(
                member.projection.operation(),
                PersistedPhysicalRecoveryOperation::DerivedDirectory {
                    retirement: Some(retirement),
                    ..
                } if retirement.dropped_records().binary_search(&record).is_ok()
            )
        })
    }
}

/// The anchored selected root identity, only when that root does not route
/// the record: a still-routed record was never retired.
pub(super) fn unrouted_under_anchored_root(
    anchored: Option<[u8; 32]>,
    mut selected_routes: impl Iterator<Item = PersistedRecordIdentity>,
    record: PersistedRecordIdentity,
) -> Option<[u8; 32]> {
    let identity = anchored?;
    (!selected_routes.any(|routed| routed == record)).then_some(identity)
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
