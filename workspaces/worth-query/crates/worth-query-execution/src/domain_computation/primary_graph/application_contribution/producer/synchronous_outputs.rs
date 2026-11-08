//! Request-local required graph; its parent is proposed truth, never settlement.
use super::coupled_assembly::DeclaredProducerAssembly;
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
pub(in crate::domain_computation::primary_graph) use capture::CapturedSynchronousOutputs;
mod discovery;
mod edges;
mod source;

pub struct AppliedOutputParent {
    writer: usize,
    pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
}
impl AppliedOutputParent {
    pub(super) fn writer(&self) -> usize {
        self.writer
    }

    pub(super) fn new(
        pin: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        writer: usize,
    ) -> Self {
        Self { pin, writer }
    }
}
pub struct SynchronousOutputFailure(CoupledSourceReadStop);
impl From<CoupledSourceReadStop> for SynchronousOutputFailure {
    fn from(cause: CoupledSourceReadStop) -> Self {
        Self(cause)
    }
}
pub struct SynchronousOutputAssembly<'a, 'owner, 'run, 'phase, S: ApplicationSchema> {
    owner: &'a DeclaredProducerAssembly<'owner, 'run, 'phase, S>,
    execution: &'a worth_execution::ExecutionRequest<'a, 'a>,
    before: &'a RelationalPinnedRead,
}
mod sealed {
    pub trait Edges {}
}
pub trait SynchronousOutputEdges<S, D>: sealed::Edges
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S>,
{
    fn assemble(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        demand: &D,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), SynchronousOutputFailure>;
}
impl sealed::Edges for ApplicationOutputLeaf {}
impl<S: ApplicationSchema, D: WorthQueryApplicationOutputDemand<S>> SynchronousOutputEdges<S, D>
    for ApplicationOutputLeaf
{
    fn assemble(
        _: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        _: &AppliedOutputParent,
        _: &D,
        _: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), SynchronousOutputFailure> {
        Ok(())
    }
}

impl<S: ApplicationSchema> SynchronousOutputAssembly<'_, '_, '_, '_, S> {
    fn checkpoint(&self, subject: &str) -> Result<(), SynchronousOutputFailure> {
        use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Denial;
        let denied = |cause: worth_execution::WorkCeilingDenial| {
            SynchronousOutputFailure(CoupledSourceReadStop::capture(
                Denial::relational_read_stopped(cause.into(), subject),
            ))
        };
        self.execution
            .run_native_with_checkpoint(|context| {
                crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)
            })
            .map_err(denied)?
            .0
            .map_err(|cause| denied(worth_execution::WorkCeilingDenial::Stopped(cause)))
    }
}
