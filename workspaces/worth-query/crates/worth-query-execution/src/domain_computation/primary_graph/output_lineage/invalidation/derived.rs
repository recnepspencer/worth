use std::sync::Arc;

mod exceptional_epoch;

use im::OrdSet;
use worth_relational::facade::{
    mvcc::{CompanionBranchCell, CompanionCellEditStop, CompanionPreflightStop},
    runtime::PositionedRelationalSnapshot,
};

use super::super::input_cutoff::StableEqualityConsequence;
use super::super::RecordedSettlementIdentity;
use super::admission::IndexAdmission;
use super::edit_admission::InvalidationEditAdmission;
use super::index_capacity;
use super::mark_state::{FullVerificationReason, SettlementCurrentness};
use super::owner::SourceInvalidationOwner;
use super::settlement::{
    AdmittedSettlementRegistration, SettlementReadAlignment, SettlementRegistration,
};
use super::source_alignment::{BranchMarkRoot, EqualOutputCurrentness, SnapshotAlignedMarkState};
use super::{equality, retention, settlement};

mod current_registration;
mod prepared;
pub(in crate::domain_computation::primary_graph) use current_registration::{
    CurrentSettlementRegistrationCleanup, PreparedCurrentSettlementRegistration,
};
pub(in crate::domain_computation::primary_graph) use prepared::{
    PreparedSettlementRegistration, SettlementRegistrationCleanup, StoppedSettlementRegistration,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum SettlementRegistrationStop {
    Alignment(FullVerificationReason),
    Admission(CompanionPreflightStop),
    Edit(CompanionCellEditStop),
}

pub(in crate::domain_computation::primary_graph) enum SourceSettlementCurrentness {
    Clean,
    Dirty(OrdSet<usize>),
    PendingUpstream(OrdSet<Arc<RecordedSettlementIdentity>>),
    FullVerificationRequired(FullVerificationReason),
}

/// Only consumed-output evidence may interpret the actor's certified equality
/// consequence. Direct callers still observe the old settlement's own marks.
pub(in crate::domain_computation::primary_graph) enum ConsumedOutputCurrentness {
    Direct(SourceSettlementCurrentness),
    CanonicallyEqualClean(Arc<RecordedSettlementIdentity>),
    PendingEqualSuccessor,
    FullVerificationRequired,
}

enum RegistrationReadBasis {
    CurrentOnly,
    RetainedReadAllowed,
}

impl From<CompanionPreflightStop> for SettlementRegistrationStop {
    fn from(value: CompanionPreflightStop) -> Self {
        Self::Admission(value)
    }
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn edit_admission(
        &self,
    ) -> InvalidationEditAdmission {
        InvalidationEditAdmission::new(self.resources.preflight_budget())
    }

    pub(in crate::domain_computation::primary_graph) fn request_admission(
        &self,
    ) -> InvalidationEditAdmission {
        InvalidationEditAdmission::structurally_bounded_request(
            self.resources.preflight_budget().maximum_preparation_bytes,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn read_admission(
        &self,
        remaining_work: usize,
    ) -> InvalidationEditAdmission {
        let mut budget = self.resources.preflight_budget();
        budget.maximum_work_visits = budget.maximum_work_visits.min(remaining_work as u64);
        InvalidationEditAdmission::new(budget)
    }

    pub(super) fn cell_for_read(
        &self,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<CompanionBranchCell<BranchMarkRoot>>, CompanionPreflightStop> {
        admission.work(1)?;
        // Readers wait for the brief cell-map section, as the writer does: the
        // writer holds it only to select or insert a cell and never re-enters
        // a reader, so a read never surfaces as Unavailable.
        let branches = self
            .branches
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        admission.ordered_read(branches.cells.len())?;
        admission.work(
            (selected.branch_id().0.len() as u64)
                .checked_mul(
                    index_capacity::ordered_navigation_work(branches.cells.len())
                        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
                )
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        Ok(branches.cells.get(selected.branch_id()).cloned())
    }

    /// The source snapshot is selected by the native reader before this method
    /// reads any mark image. There is no latest-head substitution or floor lookup.
    pub(in crate::domain_computation::primary_graph) fn currentness(
        &self,
        selected: &PositionedRelationalSnapshot,
        identity: &RecordedSettlementIdentity,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<SourceSettlementCurrentness, CompanionPreflightStop> {
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Ok(SourceSettlementCurrentness::FullVerificationRequired(
                FullVerificationReason::ForeignSource,
            ));
        }
        let Some(cell) = self.cell_for_read(selected, admission)? else {
            return Ok(SourceSettlementCurrentness::FullVerificationRequired(
                FullVerificationReason::MissingSettlement,
            ));
        };
        let image = cell.read_image();
        admission.ordered_read(image.payload().past.len())?;
        admission.ordered_read(image.payload().past.len())?;
        let aligned = match SnapshotAlignedMarkState::select_image(image, selected) {
            Ok(aligned) => aligned,
            Err(reason) => {
                return Ok(SourceSettlementCurrentness::FullVerificationRequired(
                    reason,
                ))
            }
        };
        admission.ordered_read(aligned.settlement_count())?;
        Ok(match aligned.currentness(identity) {
            SettlementCurrentness::Clean => SourceSettlementCurrentness::Clean,
            SettlementCurrentness::Dirty(ordinals) => {
                SourceSettlementCurrentness::Dirty(ordinals.clone())
            }
            SettlementCurrentness::PendingUpstream(edges) => {
                SourceSettlementCurrentness::PendingUpstream(edges.clone())
            }
            SettlementCurrentness::FullVerificationRequired(reason) => {
                SourceSettlementCurrentness::FullVerificationRequired(reason)
            }
        })
    }

    pub(in crate::domain_computation::primary_graph) fn consumed_output_currentness(
        &self,
        selected: &PositionedRelationalSnapshot,
        identity: &RecordedSettlementIdentity,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputCurrentness, CompanionPreflightStop> {
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Ok(ConsumedOutputCurrentness::FullVerificationRequired);
        }
        let Some(cell) = self.cell_for_read(selected, admission)? else {
            // Nothing committed on this branch since the runtime began, so no
            // mark and no equality consequence exists: the evidence is
            // compared in full.
            return Ok(ConsumedOutputCurrentness::Direct(
                SourceSettlementCurrentness::FullVerificationRequired(
                    FullVerificationReason::MissingSettlement,
                ),
            ));
        };
        let image = cell.read_image();
        let past_len = image.payload().past.len();
        admission.ordered_read(past_len)?;
        let aligned = match SnapshotAlignedMarkState::select_image(image, selected) {
            Ok(aligned) => aligned,
            Err(_) => {
                return Ok(ConsumedOutputCurrentness::FullVerificationRequired);
            }
        };
        Ok(
            match aligned.equal_output_currentness(identity, admission)? {
                EqualOutputCurrentness::NoConsequence => {
                    admission.ordered_read(aligned.settlement_count())?;
                    admission.ordered_read(past_len)?;
                    ConsumedOutputCurrentness::Direct(copy_currentness(
                        aligned.currentness(identity),
                    ))
                }
                EqualOutputCurrentness::CanonicallyEqualClean(successor) => {
                    ConsumedOutputCurrentness::CanonicallyEqualClean(successor)
                }
                EqualOutputCurrentness::Pending => ConsumedOutputCurrentness::PendingEqualSuccessor,
                EqualOutputCurrentness::FullVerificationRequired => {
                    ConsumedOutputCurrentness::FullVerificationRequired
                }
            },
        )
    }

    /// A sealed image comparison publishes this derived edit at the same source
    /// position. A racing native preflight returns a typed retry; it cannot be
    /// overwritten by a settlement that prepared against the previous image.
    pub(in crate::domain_computation::primary_graph) fn register_settlement(
        &self,
        registration: SettlementRegistration,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), SettlementRegistrationStop> {
        let prepared = self.prepare_settlement(registration, admission)?;
        let cleanup = prepared
            .install()
            .map_err(|stopped| SettlementRegistrationStop::Edit(stopped.reason()))?;
        drop(cleanup);
        Ok(())
    }

    /// All fact/index construction and native image custody precede the
    /// consuming installation. A prepared registration binds the exact cell
    /// image, so a racing source publication cannot be overwritten.
    pub(in crate::domain_computation::primary_graph) fn prepare_settlement(
        &self,
        registration: SettlementRegistration,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedSettlementRegistration, SettlementRegistrationStop> {
        self.prepare_settlement_at_basis(
            registration,
            RegistrationReadBasis::RetainedReadAllowed,
            None,
            admission,
        )
    }

    fn prepare_settlement_at_basis(
        &self,
        registration: SettlementRegistration,
        basis: RegistrationReadBasis,
        equality: Option<&StableEqualityConsequence<'_>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedSettlementRegistration, SettlementRegistrationStop> {
        let selected = &registration.read_basis;
        if selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(SettlementRegistrationStop::Alignment(
                FullVerificationReason::ForeignSource,
            ));
        }
        let cell = self.cell_for_read(selected, admission)?.ok_or(
            SettlementRegistrationStop::Alignment(FullVerificationReason::MissingSettlement),
        )?;
        let image = cell.read_image();
        let alignment = if image.root_id() == selected.root_id()
            && image.commit_id() == selected.commit_id()
            && image.position() == selected.position()
        {
            SettlementReadAlignment::Current
        } else {
            if matches!(basis, RegistrationReadBasis::CurrentOnly) {
                return Err(SettlementRegistrationStop::Edit(
                    CompanionCellEditStop::TopologyGenerationChanged,
                ));
            }
            if image.position() < selected.position() {
                return Err(SettlementRegistrationStop::Alignment(
                    FullVerificationReason::BeforeReadBasis,
                ));
            }
            admission.ordered_read(image.payload().past.len())?;
            if !image
                .payload()
                .past
                .get(&selected.position())
                .is_some_and(|basis| {
                    basis.root_id == selected.root_id() && basis.commit_id == selected.commit_id()
                })
            {
                return Err(SettlementRegistrationStop::Alignment(
                    FullVerificationReason::RetainedDeliveryGap,
                ));
            }
            SettlementReadAlignment::Retained
        };
        admission.bytes(
            index_capacity::arc_bytes::<super::mark_state::MarkState>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut state = (*image.payload().current).clone();
        let fact_capacity = super::fact_retention::reserve(
            &registration.facts,
            &registration.read_basis,
            &self.resources,
            admission,
        )?;
        let registered_identity = Arc::clone(&registration.identity);
        let admitted = AdmittedSettlementRegistration {
            input: registration,
            fact_capacity,
        };
        let output_coverage = if equality.is_some() {
            super::mark_state::OutputFactCoverage::Stable
        } else if admitted.input.output_facts.is_some() {
            super::mark_state::OutputFactCoverage::Performed
        } else {
            super::mark_state::OutputFactCoverage::Absent
        };
        settlement::insert(
            &mut state,
            admitted,
            image.payload(),
            alignment,
            output_coverage,
            admission,
        )?;
        if let Some(equality) = equality {
            equality::certify(&mut state, equality, admission)?;
        }
        admission.work(3)?;
        admission.ordered_read(state.settlements.len())?;
        let work_cue = state.settlements.get(&registered_identity).and_then(|row| {
            (row.verification_requirement.is_some()
                || row.delivery_epoch != state.delivery_epoch
                || !row.output_coverage.complete()
                || !row.dirty_ordinals.is_empty()
                || !row.pending_upstream.is_empty())
            .then(|| row.work_membership.as_ref().map(Arc::clone))
            .flatten()
        });
        retention::admit_state(&mut state, &self.resources, admission)?;
        admission.bytes(
            index_capacity::arc_bytes::<BranchMarkRoot>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut root = (**image.payload()).clone();
        root.current = Arc::new(state);
        retention::admit_root(&mut root, &self.resources, admission)?;
        let mut prepared = self.prepare_root_replacement(cell, image, Arc::new(root), admission)?;
        if let Some(membership) = work_cue {
            prepared.retain_work_cue(membership, registered_identity);
        }
        Ok(prepared)
    }
}

fn copy_currentness(currentness: SettlementCurrentness<'_>) -> SourceSettlementCurrentness {
    match currentness {
        SettlementCurrentness::Clean => SourceSettlementCurrentness::Clean,
        SettlementCurrentness::Dirty(ordinals) => {
            SourceSettlementCurrentness::Dirty(ordinals.clone())
        }
        SettlementCurrentness::PendingUpstream(edges) => {
            SourceSettlementCurrentness::PendingUpstream(edges.clone())
        }
        SettlementCurrentness::FullVerificationRequired(reason) => {
            SourceSettlementCurrentness::FullVerificationRequired(reason)
        }
    }
}
