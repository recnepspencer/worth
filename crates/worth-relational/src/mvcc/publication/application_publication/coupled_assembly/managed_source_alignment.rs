//! Whether the live assembly left one installed managed Source, and the
//! Parent reads beside it, unchanged at a probed scope.
use super::*;
use crate::transactions::data::primitives::RecordRef;
use crate::validation::candidate_source::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
mod assessment;
mod reach;
mod relevance;
mod source_reach;
pub use super::parent_read_footprint::ManagedIndexParentReadFootprint;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Whether no record this live assembly changed, through the retained
    /// prefix `pin` reads, can reach `program`'s Source at the probed `scope`
    /// or a read of its Parent's `footprint`.
    ///
    /// A relation change is relevant when its kind is a physical Source or a
    /// Parent relation. An entity of a Source or Parent kind is relevant when
    /// created or removed, and when updated in a projected aspect or a Parent
    /// field. A scope entity other than the probed one may be created,
    /// removed or updated when the scope kind is only the Source's root; under
    /// an ordered-events Source a newly created scope may also be born with a
    /// predecessor edge, which only its own chain can follow. Anything Native
    /// cannot classify is relevant, so `true` is never manufactured.
    /// Cost: O(changed records + deltas + program and footprint size).
    pub fn managed_source_unchanged_at(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        scope: crate::identity::data::EntityId,
        footprint: &ManagedIndexParentReadFootprint,
    ) -> Result<bool, Denial> {
        self.managed_source_unchanged(execution, program, pin, scope, footprint, None, None)
    }

    /// As `managed_source_unchanged_at`, judging only `records`: the records
    /// one applied member changed (`RelationalCoupledAppendTouches::records`),
    /// where `earlier` is the retained image right before that member. A read
    /// proved unchanged at `earlier` is still the candidate's image of it
    /// exactly when this holds. A change the kind-level rule finds relevant is
    /// then judged against what the probed scope's Source actually read at
    /// `earlier`, so a same-kind write under another scope is not this one's.
    /// A record the member shares with an earlier writer is judged whole.
    /// Cost: O(member records + changed records + deltas + program size),
    /// plus one membership acquisition when a kind is relevant.
    #[allow(clippy::too_many_arguments)]
    pub fn managed_source_unchanged_by(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        earlier: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        scope: crate::identity::data::EntityId,
        footprint: &ManagedIndexParentReadFootprint,
        records: &[RecordRef],
    ) -> Result<bool, Denial> {
        if self.managed_source_unchanged(
            execution,
            program,
            pin,
            scope,
            footprint,
            Some(records),
            None,
        )? {
            return Ok(true);
        }
        if !pin
            .candidate_identity()
            .includes_assembly_prefix(earlier.candidate_identity())
        {
            return Err(Denial::ForeignSource);
        }
        match self.source_reach(execution, program, scope, earlier)? {
            None => Ok(false),
            reach => self.managed_source_unchanged(
                execution,
                program,
                pin,
                scope,
                footprint,
                Some(records),
                reach,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn managed_source_unchanged(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        scope: crate::identity::data::EntityId,
        footprint: &ManagedIndexParentReadFootprint,
        only: Option<&[RecordRef]>,
        reach: Option<reach::Reach>,
    ) -> Result<bool, Denial> {
        let current = self.read.borrow();
        let Some(current) = current.as_ref() else {
            return Err(Denial::ForeignSource);
        };
        if !current
            .candidate_identity()
            .includes_assembly_prefix(pin.candidate_identity())
        {
            return Err(Denial::ForeignSource);
        }
        assessment::source_is_unchanged(
            execution,
            program,
            scope,
            footprint,
            self.prepared.schema_authority.registry(),
            &self.mutation.state,
            &self.mutation.outcome.effect,
            only,
            reach,
        )
    }
}
