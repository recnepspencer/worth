//! Request-local required graph; its parent is proposed truth, never settlement.
use super::super::coupled_publication::{PlannedOutput, PreparedCoupledInputs};
use super::super::CoupledApplicationAssembly;
use super::execution::CoupledSourceReadStop;
use crate::domain_computation::primary_graph::application_attempt::coupled_outputs::DeclaredCoupledOutputDemand;
use worth_query_declaration::facade::{
    application_program::{ApplicationOutputLeaf, WorthQueryApplicationOutputDemand},
    application_schema::ApplicationSchema,
};
use worth_relational::facade::{
    mvcc::RelationalAuthoredCoupledCandidate,
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
};
mod capture;
pub(in crate::domain_computation::primary_graph) enum CapturedSynchronousOutputs {
    Prepared {
        root: Box<dyn std::any::Any + Send>,
        source_recipe: Option<
            Result<
                Vec<u8>,
                crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
            >,
        >,
        _memory: Option<worth_execution::ExecutionMemoryReservation>,
    },
    Refused(retention::CaptureRefusal),
}
mod discovered;
mod retention;
mod retention_observation;
#[doc(hidden)]
pub use retention_observation::{
    SynchronousCaptureRetentionCountsForTest, SynchronousCaptureRetentionForTest,
};
mod root;
pub use discovered::SynchronousDiscoveredOutputRoot;
mod discovery;
mod edges;
mod input_preparation;
mod occurrences;
pub(in crate::domain_computation::primary_graph::application_contribution) mod rounds;
pub(in crate::domain_computation::primary_graph::application_contribution) mod source;
mod wave;
pub use input_preparation::PrerequisiteInputs;
pub use rounds::default_producer_round_bounds;
pub use wave::SynchronousOutputWave;
pub(in crate::domain_computation::primary_graph) use wave::{
    observe_scheduling_work, visited_retained_reads,
};

/// The actual authored prefix read by root discovery, without producer settlement.
struct AppliedAuthoredDiscovery {
    source: worth_relational::facade::identity::EntityId,
    pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
}
pub struct AppliedOutputParent {
    input: Option<worth_relational::facade::mvcc::RelationalRequiredOutputInput>,
    source: worth_relational::facade::identity::EntityId,
    pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    /// The assembly's identity for this output: a retained verified parent or
    /// a route's derived output.
    read: Option<crate::domain_computation::primary_graph::provider::ParentRead>,
}
impl AppliedOutputParent {
    pub(super) fn source(&self) -> worth_relational::facade::identity::EntityId {
        self.source
    }
    /// The Native input this output supplies, when it is an owned output.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn input(
        &self,
    ) -> Option<&worth_relational::facade::mvcc::RelationalRequiredOutputInput> {
        self.input.as_ref()
    }
    /// The image this output was read at.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn pin(
        &self,
    ) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
        &self.pin
    }
    /// The assembly's identity for this output.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn read(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::provider::ParentRead> {
        self.read
    }
    pub(in crate::domain_computation::primary_graph::application_contribution) fn new(
        pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        input: Option<worth_relational::facade::mvcc::RelationalRequiredOutputInput>,
    ) -> Self {
        Self {
            pin,
            source,
            input,
            read: None,
        }
    }
    pub(in crate::domain_computation::primary_graph::application_contribution) fn identified(
        mut self,
        read: crate::domain_computation::primary_graph::provider::ParentRead,
    ) -> Self {
        self.read = Some(read);
        self
    }
}
pub struct SynchronousOutputFailure(
    pub(in crate::domain_computation::primary_graph::application_contribution) CoupledSourceReadStop,
);
impl From<CoupledSourceReadStop> for SynchronousOutputFailure {
    fn from(cause: CoupledSourceReadStop) -> Self {
        Self(cause)
    }
}
pub struct SynchronousOutputAssembly<'a, 'owner, 'run, 'phase, S: ApplicationSchema> {
    pub(in crate::domain_computation::primary_graph::application_contribution) owner:
        &'a CoupledApplicationAssembly<'owner, 'run, 'phase, S>,
    pub(in crate::domain_computation::primary_graph::application_contribution) execution:
        &'a worth_execution::ExecutionRequest<'a, 'a>,
    pub(in crate::domain_computation::primary_graph::application_contribution) before:
        &'a RelationalPinnedRead,
    pub(in crate::domain_computation::primary_graph::application_contribution) order:
        &'a worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
}
mod sealed {
    pub trait Edges {}
}
pub trait SynchronousOutputEdges<S, D>: sealed::Edges
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S>,
{
    fn occurrence_families(_visit: &mut impl FnMut(std::any::TypeId)) {}
    fn occurrence_context(
        _family: std::any::TypeId,
    ) -> Result<Option<std::any::TypeId>, SynchronousOutputFailure> {
        Ok(None)
    }
    fn has_static_continuation() -> bool {
        false
    }
    fn prerequisites(
        _context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        _candidate: &RelationalAuthoredCoupledCandidate<'_>,
        _pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        _source: worth_relational::facade::identity::EntityId,
        _demand: &D,
        _inputs: &PrerequisiteInputs<'_>,
        _next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        Ok(())
    }
    fn declared_families(
        parent: std::any::TypeId,
        visit: &mut impl FnMut(std::any::TypeId, std::any::TypeId),
    );
    fn assemble(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
    fn assemble_static(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        Self::assemble(context, candidate, parent, demand, next)
    }
    fn assemble_occurrences(
        _context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        _candidate: &RelationalAuthoredCoupledCandidate<'_>,
        _parent: &AppliedOutputParent,
        _demand: &D,
        _next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        Ok(())
    }
}
impl sealed::Edges for ApplicationOutputLeaf {}
impl<S: ApplicationSchema, D: WorthQueryApplicationOutputDemand<S>> SynchronousOutputEdges<S, D>
    for ApplicationOutputLeaf
{
    fn declared_families(
        _: std::any::TypeId,
        _: &mut impl FnMut(std::any::TypeId, std::any::TypeId),
    ) {
    }
    fn assemble(
        _: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        _: &RelationalAuthoredCoupledCandidate<'_>,
        _: &AppliedOutputParent,
        _: &D,
        _: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        Ok(())
    }
}

impl<S: ApplicationSchema> SynchronousOutputAssembly<'_, '_, '_, '_, S> {
    fn checkpoint(&self, subject: &str) -> Result<(), SynchronousOutputFailure> {
        checkpoint(self.execution, subject, 1)
    }
}
fn checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    subject: &str,
    work: u64,
) -> Result<(), SynchronousOutputFailure> {
    use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Denial;
    let denied = |cause: worth_execution::WorkCeilingDenial| {
        SynchronousOutputFailure(CoupledSourceReadStop::capture(
            Denial::relational_read_stopped(cause.into(), subject),
        ))
    };
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, work)
        })
        .map_err(denied)?
        .0
        .map_err(|cause| denied(worth_execution::WorkCeilingDenial::Stopped(cause)))
}

impl CapturedSynchronousOutputs {
    pub(in crate::domain_computation::primary_graph) fn source_recipe(
        &self,
    ) -> Option<
        &Result<
            Vec<u8>,
            crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
        >,
    > {
        match self {
            Self::Prepared { source_recipe, .. } => source_recipe.as_ref(),
            Self::Refused(_) => None,
        }
    }
}
