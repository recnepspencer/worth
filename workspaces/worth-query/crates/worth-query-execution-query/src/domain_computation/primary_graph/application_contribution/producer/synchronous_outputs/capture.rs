//! Immutable, sendable required demand carried into its source's assembly.
use super::*;
use source::{Source, SourceInput};
use worth_query_execution_attempt::facade::ports::{
    CompleteReadSetInternalPort as _, EffectProgramInternalPort as _,
};
mod continuation;
mod tree;
pub use continuation::{Continuation, Static};
pub use tree::RequiredTree;
use tree::Tree;
use worth_query_declaration::facade::{
    application_program::WorthQueryApplicationRequiredOutputConnection,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
};
type Scope<S, D> = <Source<S, D> as ApplicationQueryBinding<S>>::ScopeBinding;
type Principal<S, D> = <Source<S, D> as ApplicationQueryBinding<S>>::PrincipalIdentity;

pub fn schedule<S, D, E>(
    context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
    wave: &mut SynchronousOutputWave<S>,
    connection: &'static str,
    position: worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
    demand: D,
) -> Result<(), SynchronousOutputFailure>
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S> + Clone + Send,
    E: SynchronousOutputEdges<S, D> + 'static,
    SourceInput<S, D>: ApplicationQueryIntent<S, Binding = Source<S, D>>,
    Scope<S, D>: ApplicationQueryScopeResolution<S, Principal<S, D>>,
{
    schedule_continued(
        context,
        wave,
        connection,
        position,
        demand,
        Static::<E>::new(),
        None,
    )
}
/// Schedule `demand` with an explicit continuation value.
pub fn schedule_continued<S, D, K>(
    context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
    wave: &mut SynchronousOutputWave<S>,
    connection: &'static str,
    position: worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
    demand: D,
    continuation: K,
    expected_source: Option<worth_relational::facade::identity::EntityId>,
) -> Result<(), SynchronousOutputFailure>
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S> + Clone + Send,
    K: Continuation<S, D>,
    SourceInput<S, D>: ApplicationQueryIntent<S, Binding = Source<S, D>>,
    Scope<S, D>: ApplicationQueryScopeResolution<S, Principal<S, D>>,
{
    wave.push(
        context,
        connection,
        &position,
        std::mem::size_of::<Tree<D, K>>(),
        || {
            Box::new(Tree::<D, K> {
                connection,
                position: position.clone(),
                order: context.order.clone(),
                demand,
                continuation,
                expected_source,
            })
        },
    )
}
impl CapturedSynchronousOutputs {
    pub fn fixed<S, D, E>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        connection: &'static str,
        demand: D,
        position: worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
        order: &worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
    ) -> Self
    where
        S: ApplicationSchema,
        D: WorthQueryApplicationOutputDemand<S> + Clone + Send,
        E: SynchronousOutputEdges<S, D> + 'static,
        SourceInput<S, D>: ApplicationQueryIntent<S, Binding = Source<S, D>>,
        Scope<S, D>: ApplicationQueryScopeResolution<S, Principal<S, D>>,
    {
        Self::capture(
            execution,
            connection,
            size_of::<Tree<D, Static<E>>>() + size_of::<root::Root<S>>(),
            || {
                Box::new(root::Root::Fixed(Box::new(Tree::<D, Static<E>> {
                    connection,
                    position,
                    order: order.clone(),
                    demand,
                    continuation: Static::new(),
                    expected_source: None,
                })))
            },
        )
    }
    pub fn assemble<S: ApplicationSchema>(
        roots: &[&Self],
        owner: &CoupledApplicationAssembly<'_, '_, '_, S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), CoupledSourceReadStop> {
        let mut wave = SynchronousOutputWave::new();
        let order = Self::seed(roots, owner, execution, before, candidate, &mut wave)?;
        let Some(order) = order else { return Ok(()) };
        let context = SynchronousOutputAssembly {
            owner,
            execution,
            before,
            order: &order,
        };
        wave.drive(&context, candidate, None)
            .map_err(|cause| cause.0)
    }
    pub fn seed<S: ApplicationSchema>(
        roots: &[&Self],
        owner: &CoupledApplicationAssembly<'_, '_, '_, S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        wave: &mut SynchronousOutputWave<S>,
    ) -> Result<
        Option<
            worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
        >,
        CoupledSourceReadStop,
    > {
        let mut order = None;
        for captured in roots {
            let Self::Prepared { root, _memory, .. } = captured else {
                let Self::Refused(cause) = captured else {
                    unreachable!()
                };
                return Err(CoupledSourceReadStop::capture(cause.denial()));
            };
            let tree = root.downcast_ref::<root::Root<S>>().ok_or_else(|| {
                super::super::WorthQueryOutputDemandDenial::new(
                    super::super::WorthQueryOutputDemandDenialKind::ForeignDemand,
                    "required graph schema",
                )
            })?;
            retention_observation::record(
                execution.has_accounting(),
                tree.container_bytes(),
                _memory
                    .as_ref()
                    .map(worth_execution::ExecutionMemoryReservation::bytes),
            );
            let context = SynchronousOutputAssembly {
                owner,
                execution,
                before,
                order: tree.order(),
            };
            order = Some(tree.order().clone());
            tree.seed(&context, candidate, wave)
                .map_err(|cause| cause.0)?;
        }
        Ok(order)
    }
}
pub trait EffectProgramSynchronousOutputsExt<S: ApplicationSchema, O, I, Scope>: Sized {
    fn with_synchronous_required_outputs<C, E>(
        self,
        _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        demand: C::Demand,
        position: worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
        order: &worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
    ) -> Self
    where
        C: WorthQueryApplicationRequiredOutputConnection<S>,
        C::Demand: Clone + Send,
        E: SynchronousOutputEdges<S, C::Demand> + 'static,
        SourceInput<S, C::Demand>: ApplicationQueryIntent<S, Binding = Source<S, C::Demand>>,
        <Source<S, C::Demand> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <Source<S, C::Demand> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >;
}

impl<S: ApplicationSchema, O, I, Scope> EffectProgramSynchronousOutputsExt<S, O, I, Scope>
    for crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<S, O, I, Scope>
{
    fn with_synchronous_required_outputs<C, E>(
        mut self,
        _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        demand: C::Demand,
        position: worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
        order: &worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
    ) -> Self
    where
        C: WorthQueryApplicationRequiredOutputConnection<S>,
        C::Demand: Clone + Send,
        E: SynchronousOutputEdges<S, C::Demand> + 'static,
        SourceInput<S, C::Demand>: ApplicationQueryIntent<S, Binding = Source<S, C::Demand>>,
        <Source<S, C::Demand> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <Source<S, C::Demand> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
    {
        self.synchronous_outputs_mut().push(
            worth_query_execution_attempt::facade::ports::OpaqueSynchronousOutputs::new(
                CapturedSynchronousOutputs::capture(
                    execution,
                    C::IDENTITY,
                    std::mem::size_of::<Tree<C::Demand, Static<E>>>()
                        + std::mem::size_of::<root::Root<S>>(),
                    || {
                        let tree: Box<dyn RequiredTree<S>> =
                            Box::new(Tree::<C::Demand, Static<E>> {
                                connection: C::IDENTITY,
                                position,
                                order: order.clone(),
                                demand,
                                continuation: Static::new(),
                                expected_source: None,
                            });
                        Box::new(root::Root::Fixed(tree))
                    },
                ),
            ),
        );
        self
    }
}
