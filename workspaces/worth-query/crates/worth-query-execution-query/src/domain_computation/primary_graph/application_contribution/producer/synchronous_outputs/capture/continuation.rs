//! How a required demand continues after its own supply: its declared static
//! edges, or the complete occurrence component carried as a value.
use super::super::*;
use std::marker::PhantomData;

pub trait Continuation<S: ApplicationSchema, D>:
    Clone + Send + 'static
{
    fn occurrence_context(
        &self,
        family: std::any::TypeId,
    ) -> Result<Option<std::any::TypeId>, SynchronousOutputFailure>;
    fn has_static_continuation(&self) -> bool;
    fn prerequisites(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        demand: &D,
        inputs: &PrerequisiteInputs<'_>,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
    fn assemble(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
    fn assemble_static(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
    fn assemble_occurrences(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
}

/// The declared static edges `E` of a demand, dispatched by type.
pub struct Static<E>(PhantomData<fn() -> E>);
impl<E> Static<E> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}
impl<E> Clone for Static<E> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<S, D, E> Continuation<S, D> for Static<E>
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S>,
    E: SynchronousOutputEdges<S, D> + 'static,
{
    fn occurrence_context(
        &self,
        family: std::any::TypeId,
    ) -> Result<Option<std::any::TypeId>, SynchronousOutputFailure> {
        E::occurrence_context(family)
    }
    fn has_static_continuation(&self) -> bool {
        E::has_static_continuation()
    }
    fn prerequisites(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        demand: &D,
        inputs: &PrerequisiteInputs<'_>,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        E::prerequisites(context, candidate, pin, source, demand, inputs, next)
    }
    fn assemble(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        E::assemble(context, candidate, parent, demand, next)
    }
    fn assemble_static(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        E::assemble_static(context, candidate, parent, demand, next)
    }
    fn assemble_occurrences(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        E::assemble_occurrences(context, candidate, parent, demand, next)
    }
}
