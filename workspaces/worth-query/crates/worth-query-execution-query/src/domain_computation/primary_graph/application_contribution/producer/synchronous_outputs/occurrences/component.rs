//! One complete occurrence component, carried as a `'static` value.
//!
//! A component is named by its declared [`ApplicationOccurrenceComponent`]
//! type. Its structural connections are walked once per component into fixed
//! per-connection entry points; every nested or recursive placement carries
//! the same table, so execution names never spell the complete component.
use super::*;
use std::any::TypeId;
use worth_query_declaration::facade::application_program::ApplicationOccurrenceConnections;

type Step = Result<(), SynchronousOutputFailure>;
type Visit<'v, S> = dyn FnMut(&'static LeafExecution<S>) -> Step + 'v;

/// The executable entry points of one typed connection.
pub struct LeafExecution<S: ApplicationSchema> {
    pub outgoing: fn(
        &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        &RelationalAuthoredCoupledCandidate<'_>,
        &AppliedOutputParent,
        &dyn Any,
        &mut SynchronousOutputWave<S>,
        &'static ComponentExecution<S>,
    ) -> Step,
    pub incoming: fn(
        &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        &RelationalAuthoredCoupledCandidate<'_>,
        &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        worth_relational::facade::identity::EntityId,
        &dyn Any,
        &PrerequisiteInputs<'_>,
        &mut SynchronousOutputWave<S>,
        &'static ComponentExecution<S>,
    ) -> Step,
}

/// The complete component: its identity and its connections in declaration
/// order (the order of `ApplicationOccurrenceConnections::append`).
pub struct ComponentExecution<S: ApplicationSchema> {
    context: fn() -> TypeId,
    families: fn(&mut dyn FnMut(TypeId)),
    each: fn(&mut Visit<'_, S>) -> Step,
}
impl<S: ApplicationSchema> ComponentExecution<S> {
    pub fn identity(&self) -> TypeId {
        (self.context)()
    }
    fn occurrence_context(&self, family: TypeId) -> Option<TypeId> {
        let mut member = false;
        (self.families)(&mut |candidate| member |= candidate == family);
        member.then(self.context)
    }
}

/// Structural composition inside one component; never a component identity.
trait Component<S: ApplicationSchema>: ApplicationOccurrenceConnections<S> {
    fn families(visit: &mut dyn FnMut(TypeId));
    fn each(visit: &mut Visit<'_, S>) -> Step;
}
macro_rules! tuple {
    ($($c:ident),+) => {
        impl<S: ApplicationSchema, $($c: ConnectionExecution<S>),+> Component<S> for ($($c,)+) {
            fn families(visit: &mut dyn FnMut(TypeId)) { $($c::families(visit);)+ }
            fn each(visit: &mut Visit<'_, S>) -> Step { $(visit($c::LEAF)?;)+ Ok(()) }
        }
    }
}
tuple!(A);
tuple!(A, B);
tuple!(A, B, C);
tuple!(A, B, C, D);
tuple!(A, B, C, D, E);
tuple!(A, B, C, D, E, F);
tuple!(A, B, C, D, E, F, G);
tuple!(A, B, C, D, E, F, G, H);
impl<S, L, R> Component<S> for ApplicationOccurrenceGroup<L, R>
where
    S: ApplicationSchema,
    L: Component<S>,
    R: Component<S>,
{
    fn families(visit: &mut dyn FnMut(TypeId)) {
        L::families(visit);
        R::families(visit);
    }
    fn each(visit: &mut Visit<'_, S>) -> Step {
        L::each(visit)?;
        R::each(visit)
    }
}

/// A component's identity is its named type, for point equality only.
fn identity<N: 'static>() -> TypeId {
    crate::equality_type_id::EqualityTypeId::of::<N>().into_external()
}
/// The one table of a named component.
trait Named<S: ApplicationSchema>: ApplicationOccurrenceComponent<S> {
    const EXECUTION: &'static ComponentExecution<S>;
}
impl<S, N> Named<S> for N
where
    S: ApplicationSchema,
    N: ApplicationOccurrenceComponent<S>,
    N::Connections: Component<S>,
{
    const EXECUTION: &'static ComponentExecution<S> = &ComponentExecution {
        context: identity::<N>,
        families: <N::Connections as Component<S>>::families,
        each: <N::Connections as Component<S>>::each,
    };
}

/// A scheduled occurrence continues with the complete component it was
/// discovered in, exactly as the component's root placement does.
impl<S, D> capture::Continuation<S, D> for &'static ComponentExecution<S>
where
    S: ApplicationSchema,
    D: 'static,
{
    fn occurrence_context(
        &self,
        family: TypeId,
    ) -> Result<Option<TypeId>, SynchronousOutputFailure> {
        Ok(ComponentExecution::occurrence_context(self, family))
    }
    fn has_static_continuation(&self) -> bool {
        false
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
    ) -> Step {
        let whole = *self;
        (whole.each)(&mut |leaf| {
            (leaf.incoming)(context, candidate, pin, source, demand, inputs, next, whole)
        })
    }
    fn assemble(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Step {
        let whole = *self;
        (whole.each)(&mut |leaf| (leaf.outgoing)(context, candidate, parent, demand, next, whole))
    }
    fn assemble_static(
        &self,
        _: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        _candidate: &RelationalAuthoredCoupledCandidate<'_>,
        _: &AppliedOutputParent,
        _: &D,
        _: &mut SynchronousOutputWave<S>,
    ) -> Step {
        Ok(())
    }
    fn assemble_occurrences(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Step {
        self.assemble(context, candidate, parent, demand, next)
    }
}

impl<T> sealed::Edges for ApplicationOutputOccurrences<T> {}
/// A root placement runs the same table as every scheduled occurrence.
impl<S, D, N> SynchronousOutputEdges<S, D> for ApplicationOutputOccurrences<N>
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S>,
    N: Named<S>,
{
    fn occurrence_families(visit: &mut impl FnMut(TypeId)) {
        (N::EXECUTION.families)(visit);
    }
    fn occurrence_context(family: TypeId) -> Result<Option<TypeId>, SynchronousOutputFailure> {
        capture::Continuation::<S, D>::occurrence_context(&N::EXECUTION, family)
    }
    fn declared_families(_: TypeId, _: &mut impl FnMut(TypeId, TypeId)) {
        /* Internal arcs have actual occurrence identities, not a declaration DAG. */
    }
    fn prerequisites(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        demand: &D,
        inputs: &PrerequisiteInputs<'_>,
        next: &mut SynchronousOutputWave<S>,
    ) -> Step {
        capture::Continuation::prerequisites(
            &N::EXECUTION,
            context,
            candidate,
            pin,
            source,
            demand,
            inputs,
            next,
        )
    }
    fn assemble(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Step {
        capture::Continuation::assemble(&N::EXECUTION, context, candidate, parent, demand, next)
    }
    fn assemble_static(
        _: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        _candidate: &RelationalAuthoredCoupledCandidate<'_>,
        _: &AppliedOutputParent,
        _: &D,
        _: &mut SynchronousOutputWave<S>,
    ) -> Step {
        Ok(())
    }
    fn assemble_occurrences(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Step {
        capture::Continuation::assemble(&N::EXECUTION, context, candidate, parent, demand, next)
    }
}
