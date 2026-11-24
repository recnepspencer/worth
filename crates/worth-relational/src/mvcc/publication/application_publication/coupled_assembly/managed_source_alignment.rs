//! Whether the live assembly left one installed managed Source, and the
//! Parent reads beside it, unchanged at a probed scope.
use super::*;
use crate::transactions::data::primitives::RecordRef;
use crate::validation::candidate_source::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::managed_source_alignment::{assessment::Assessment, reach::Reach};
use crate::validation::candidate_source::applied_read::ManagedIndexAppliedSourceRead;
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
        self.source_alignment()
            .unchanged_at(execution, program, pin, scope, footprint)
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
        self.source_alignment().unchanged_by(
            execution,
            program,
            pin,
            earlier,
            scope,
            footprint,
            records,
            || {
                let loan = ManagedIndexAppliedSourceRead::issue(
                    execution,
                    self.identity,
                    earlier.clone(),
                    program,
                    scope,
                )?;
                Reach::acquire(loan.rows(*execution)?)
            },
        )
    }

    fn source_alignment(&self) -> Assessment<'_> {
        Assessment {
            current: &self.read,
            registry: self.prepared.schema_authority.registry(),
            state: &self.mutation.state,
            effect: &self.mutation.outcome.effect,
        }
    }
}
