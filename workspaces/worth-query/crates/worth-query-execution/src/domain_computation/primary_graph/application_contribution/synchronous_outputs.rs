//! Outer-runtime wrapper trait for synchronous discovered output roots.

use std::sync::Arc;
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationDiscoveredOutputGraph, ApplicationDiscoveredOutputRoot,
    ApplicationOutputGraphShape, ApplicationProgramConnectionOrder,
};
use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::SynchronousDiscoveredOutputRoot as InnerSynchronousDiscoveredOutputRoot;
use worth_query_installation::facade::ApplicationSchema;

use super::authored_edit::CapturedAuthoredEditStage;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationEffectProgram,
    WorthQueryOutputDemandDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

mod sealed {
    pub trait Root {}
}
impl<C, E> sealed::Root for ApplicationDiscoveredOutputGraph<C, E> {}

pub trait SynchronousDiscoveredOutputRoot<S>:
    ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot + sealed::Root
where
    S: ApplicationSchema,
    <Self::RootConnection as ApplicationConnectionShape<S>>::Binding:
        WorthQueryApplicationDiscoveredOutputConnection<S>,
{
    fn attach<O, I, Scope>(
        program: WorthQueryApplicationEffectProgram<S, O, I, Scope>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        recipe: Arc<
            <<Self::RootConnection as ApplicationConnectionShape<S>>::Binding as WorthQueryApplicationDiscoveredOutputConnection<S>>::Discovery,
        >,
        order: &ApplicationProgramConnectionOrder,
    ) -> WorthQueryApplicationEffectProgram<S, O, I, Scope>;

    fn dependencies(
        stage: CapturedAuthoredEditStage<S>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        recipe: Arc<
            <<Self::RootConnection as ApplicationConnectionShape<S>>::Binding as WorthQueryApplicationDiscoveredOutputConnection<S>>::Discovery,
        >,
        order: &ApplicationProgramConnectionOrder,
    ) -> Result<CapturedAuthoredEditStage<S>, WorthQueryOutputDemandDenial>;
}

impl<S, C, E> SynchronousDiscoveredOutputRoot<S> for ApplicationDiscoveredOutputGraph<C, E>
where
    S: ApplicationSchema,
    ApplicationDiscoveredOutputGraph<C, E>: InnerSynchronousDiscoveredOutputRoot<S>,
    <Self::RootConnection as ApplicationConnectionShape<S>>::Binding:
        WorthQueryApplicationDiscoveredOutputConnection<S>,
{
    fn attach<O, I, Scope>(
        program: WorthQueryApplicationEffectProgram<S, O, I, Scope>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        recipe: Arc<
            <<Self::RootConnection as ApplicationConnectionShape<S>>::Binding as WorthQueryApplicationDiscoveredOutputConnection<S>>::Discovery,
        >,
        order: &ApplicationProgramConnectionOrder,
    ) -> WorthQueryApplicationEffectProgram<S, O, I, Scope> {
        <Self as InnerSynchronousDiscoveredOutputRoot<S>>::attach(
            program, access, execution, runtime, recipe, order,
        )
    }

    fn dependencies(
        stage: CapturedAuthoredEditStage<S>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        recipe: Arc<
            <<Self::RootConnection as ApplicationConnectionShape<S>>::Binding as WorthQueryApplicationDiscoveredOutputConnection<S>>::Discovery,
        >,
        order: &ApplicationProgramConnectionOrder,
    ) -> Result<CapturedAuthoredEditStage<S>, WorthQueryOutputDemandDenial> {
        <Self as InnerSynchronousDiscoveredOutputRoot<S>>::dependencies(
            stage.0,
            application,
            recipe,
            order,
        )
        .map(CapturedAuthoredEditStage)
    }
}
