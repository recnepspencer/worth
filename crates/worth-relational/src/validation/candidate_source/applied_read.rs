//! A compiled Source loan over one real, unpublished coupled assembly prefix.
mod publication;
use super::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
    ManagedIndexSourceProjection, ManagedIndexSourceRows, NativeManagedSourceInterpretation,
};
use crate::derived_relation::{DerivedRelationDenial as Definition, DerivedRelationReadDenial};
use crate::identity::data::EntityId;
use crate::runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead};
use crate::visibility::materialization::read_records::VisibilityProjectionView;
pub use publication::ManagedIndexAppliedSourcePublication;
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};
use worth_execution::ExecutionRequest;

/// Physical Source custody for an exact applied image. It grants neither
/// principal READ admission nor a positioned Source/currentness/settlement.
/// Only the coupled assembly owner can issue it; every row operation borrows
/// the caller's original request and the compiled Source projection machinery.
pub struct ManagedIndexAppliedSourceRead<'candidate> {
    _owner: &'candidate crate::mvcc::RelationalMutationProposalIdentity,
    pub(crate) projection: VisibilityProjectionView,
    pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    pub(super) affinity: Arc<AppliedSourceAffinity>,
    interpretation: Arc<NativeManagedSourceInterpretation>,
}

pub(super) struct AppliedSourceAffinity {
    pub(super) candidate: RelationalAppliedCandidateReadIdentity,
    pub(super) program: ManagedIndexSourceProgram,
    pub(super) scope: EntityId,
    generation: u64,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl<'candidate> ManagedIndexAppliedSourceRead<'candidate> {
    pub(crate) fn issue(
        execution: &ExecutionRequest<'_, '_>,
        owner: &'candidate crate::mvcc::RelationalMutationProposalIdentity,
        pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<Self, Denial> {
        execution
            .check_live()
            .map_err(|cause| DerivedRelationReadDenial::Request(cause.into()))?;
        if !owner.same_proposal_sequence(pin.candidate_identity().proposal()) {
            return Err(Denial::ForeignSource);
        }
        let interpretation = pin
            .candidate_identity()
            .proposal()
            .source_interpretation()
            .ok_or(Denial::ForeignSource)?;
        let generation = interpretation.current_registration_generation()?;
        let projection = pin.project_candidate();
        let metadata = execution
            .reserve_retained_memory(size_of::<AppliedSourceAffinity>() as u64)
            .map_err(|cause| DerivedRelationReadDenial::Request(cause.into()))?;
        let source = Self {
            _owner: owner,
            projection,
            affinity: Arc::new(AppliedSourceAffinity {
                candidate: pin.candidate_identity().clone(),
                program: program.clone(),
                scope,
                generation,
                _memory: metadata,
            }),
            interpretation,
            pin,
        };
        // Missing/foreign scope is a refusal, never an empty Source.
        if source.rows(*execution)?.projection.entity_kind(scope)? != program.scope_kind() {
            return Err(Definition::InvalidMemberDomain.into());
        }
        Ok(source)
    }

    /// Retain the real loan's publication affinity. This is still unpublished
    /// Source custody; only observation of its actual commit can position it.
    pub fn publication_support(&self) -> ManagedIndexAppliedSourcePublication {
        ManagedIndexAppliedSourcePublication::from_loan(self)
    }

    pub fn candidate_identity(&self) -> &RelationalAppliedCandidateReadIdentity {
        &self.affinity.candidate
    }
    /// The exact image already borrowed by this Source loan, without any
    /// committed position or completed Query read authority.
    pub fn candidate_pin(&self) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
        &self.pin
    }
    pub fn program(&self) -> &ManagedIndexSourceProgram {
        &self.affinity.program
    }
    pub fn scope(&self) -> EntityId {
        self.affinity.scope
    }
    pub fn rows<'a>(
        &'a self,
        execution: ExecutionRequest<'a, 'a>,
    ) -> Result<ManagedIndexSourceRows<'a>, Denial> {
        if self.interpretation.current_registration_generation()? != self.affinity.generation {
            return Err(Denial::Publication(
                crate::mvcc::CompanionPreflightStop::RegistrationChanged,
            ));
        }
        execution
            .check_live()
            .map_err(|cause| DerivedRelationReadDenial::Request(cause.into()))?;
        Ok(ManagedIndexSourceRows {
            scope: self.scope(),
            projection: ManagedIndexSourceProjection {
                program: self.program(),
                backing: super::projection::SourceProjectionBacking::Applied {
                    loan: self,
                    execution,
                    memory: RefCell::new(None),
                    bytes: Cell::new(0),
                },
            },
        })
    }
}
