//! A live, complete actor image is the only cheap current-output certificate.

use std::sync::Arc;

use worth_relational::facade::{
    mvcc::{
        CompanionBranchCell, CompanionBranchImage, CompanionCellEditStop, CompanionPreflightStop,
    },
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
    snapshots::SnapshotHandle,
};

use super::super::RecordedSettlementIdentity;
use super::{
    admission::IndexAdmission,
    derived::{
        PreparedSettlementRegistration, SettlementRegistrationCleanup, SettlementRegistrationStop,
        StoppedSettlementRegistration,
    },
    equality, index_capacity,
    mark_state::{FullVerificationReason, MarkState, SettlementCurrentness},
    read_basis_carry, retention,
    source_alignment::{BranchMarkRoot, SnapshotAlignedMarkState},
    InvalidationEditAdmission, SourceInvalidationOwner,
};

pub(in crate::domain_computation::primary_graph) enum PreparedVerifiedCurrent {
    AlreadyCurrent {
        cell: CompanionBranchCell<BranchMarkRoot>,
        image: CompanionBranchImage<BranchMarkRoot>,
    },
    /// The clean row's read basis moves to the live image, and downstream
    /// rows pending on it are discharged.
    LiveEdit(PreparedSettlementRegistration),
}

pub(in crate::domain_computation::primary_graph) struct StoppedVerifiedCurrent {
    reason: CompanionCellEditStop,
    _prepared: Option<StoppedSettlementRegistration>,
    _observed: Option<ObservedCurrentCustody>,
}

pub(in crate::domain_computation::primary_graph) struct VerifiedCurrentCleanup {
    _observed: Option<ObservedCurrentCustody>,
    _edited: Option<SettlementRegistrationCleanup>,
}

pub(in crate::domain_computation::primary_graph) struct ObservedCurrentCustody {
    _cell: CompanionBranchCell<BranchMarkRoot>,
    _expected: CompanionBranchImage<BranchMarkRoot>,
    _current: CompanionBranchImage<BranchMarkRoot>,
}

impl StoppedVerifiedCurrent {
    pub(in crate::domain_computation::primary_graph) fn reason(&self) -> CompanionCellEditStop {
        self.reason
    }
}

impl PreparedVerifiedCurrent {
    pub(in crate::domain_computation::primary_graph) fn install(
        self,
    ) -> Result<VerifiedCurrentCleanup, StoppedVerifiedCurrent> {
        match self {
            Self::AlreadyCurrent { cell, image } => {
                let current = cell.read_image();
                let same = current.root_id() == image.root_id()
                    && current.commit_id() == image.commit_id()
                    && current.position() == image.position()
                    && current.topology_generation() == image.topology_generation()
                    && Arc::ptr_eq(current.payload(), image.payload());
                let custody = ObservedCurrentCustody {
                    _cell: cell,
                    _expected: image,
                    _current: current,
                };
                if same {
                    Ok(VerifiedCurrentCleanup {
                        _observed: Some(custody),
                        _edited: None,
                    })
                } else {
                    Err(StoppedVerifiedCurrent {
                        reason: CompanionCellEditStop::TopologyGenerationChanged,
                        _prepared: None,
                        _observed: Some(custody),
                    })
                }
            }
            Self::LiveEdit(prepared) => prepared
                .install()
                .map(|edited| VerifiedCurrentCleanup {
                    _observed: None,
                    _edited: Some(edited),
                })
                .map_err(|stopped| StoppedVerifiedCurrent {
                    reason: stopped.reason(),
                    _prepared: Some(stopped),
                    _observed: None,
                }),
        }
    }
}

impl SourceInvalidationOwner {
    /// This checks the live image only. Historical Clean does not authorize an
    /// edit to downstream pending marks at the selected current source.
    pub(in crate::domain_computation::primary_graph) fn prepare_verified_current(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        identity: &Arc<RecordedSettlementIdentity>,
        expected_facts: &super::super::ComparableSourceFacts,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<PreparedVerifiedCurrent>, SettlementRegistrationStop> {
        let branch_work = u64::try_from(selected.branch_id().0.len())
            .ok()
            .and_then(|len| len.checked_add(1))
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        admission.work(branch_work)?;
        admission.bytes(selected.branch_id().0.len() as u64)?;
        let actual = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| SettlementRegistrationStop::SourceUnavailable)?;
        if &actual != selected || selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(SettlementRegistrationStop::Foreign);
        }
        let cell = self.cell_for_read(selected, admission)?.ok_or(
            SettlementRegistrationStop::Alignment(FullVerificationReason::MissingSettlement),
        )?;
        let image = cell.read_image();
        if image.root_id() != selected.root_id()
            || image.commit_id() != selected.commit_id()
            || image.position() != selected.position()
        {
            return Ok(None);
        }
        admission.ordered_read(image.payload().past.len())?;
        let aligned = SnapshotAlignedMarkState::observe_image(&image, selected)
            .map_err(SettlementRegistrationStop::Alignment)?;
        admission.ordered_read(aligned.settlement_count())?;
        if !matches!(aligned.currentness(identity), SettlementCurrentness::Clean) {
            return Ok(None);
        }
        let state = &image.payload().current;
        admission.ordered_read(state.settlements.len())?;
        let Some(row) = state.settlements.get(identity) else {
            return Ok(None);
        };
        if !row.output_coverage.complete()
            || !row
                .facts
                .for_comparison()
                .is_some_and(|facts| Arc::ptr_eq(facts.facts(), expected_facts.facts()))
        {
            return Ok(None);
        }

        admission.ordered_read(state.downstream.len())?;
        let mut has_pending = false;
        if let Some(targets) = state.downstream.get(identity) {
            for target in targets {
                admission.work(1)?;
                admission.ordered_read(state.settlements.len())?;
                let Some(target_row) = state.settlements.get(target) else {
                    continue;
                };
                admission.ordered_read(target_row.pending_upstream.len())?;
                if target_row.pending_upstream.contains(identity) {
                    has_pending = true;
                    break;
                }
            }
        }
        // A row found clean here is verified through this image, so its read
        // basis moves to it; otherwise it would leave the retained window.
        let carried = *row.read_basis != *selected;
        if !has_pending && !carried {
            return Ok(Some(PreparedVerifiedCurrent::AlreadyCurrent {
                cell,
                image,
            }));
        }

        let before = admission.charged_bytes();
        admission.bytes(
            index_capacity::arc_bytes::<MarkState>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut next = (**state).clone();
        if carried {
            read_basis_carry::carry_row(
                &mut next, &aligned, selected, &mut None, identity, admission,
            )?;
        }
        if has_pending {
            equality::discharge_selected_downstream(&mut next, identity, admission)?;
        }
        let root = retention::admit_live_replacement(
            image.payload(),
            next,
            before,
            &self.resources,
            admission,
        )?;
        self.prepare_root_replacement(cell, image, Arc::new(root), admission)
            .map(|prepared| Some(PreparedVerifiedCurrent::LiveEdit(prepared)))
    }
}
