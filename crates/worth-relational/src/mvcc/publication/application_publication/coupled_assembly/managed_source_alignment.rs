//! Whether the live assembly left one installed managed Source, and the
//! Parent reads beside it, unchanged at a probed scope.
use super::*;
use crate::validation::candidate_source::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
pub(crate) use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::managed_source_alignment::assessment;
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
        )
    }
}
