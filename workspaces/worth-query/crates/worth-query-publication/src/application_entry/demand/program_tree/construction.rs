//! Scope-escaping original construction custody, including an already issued basis.
use super::*;
/// Why construction could not admit the original complete output-tree request.
#[derive(Debug)]
pub enum WorthQueryProgramOutputConstructionDenial {
    FreshRequestMismatch,
    UndeclaredRoot,
    Execution(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial),
    Observation(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    Program(worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwnerDenial),
}
/// Owns refused construction inputs and any original issued basis for retry or close.
#[must_use]
pub struct WorthQueryProgramOutputConstructionRefusal<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) denial: WorthQueryProgramOutputConstructionDenial,
    pub(super) core: Core<S, P, R>,
}
impl<S, P, R> WorthQueryProgramOutputConstructionRefusal<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn denial(&self) -> &WorthQueryProgramOutputConstructionDenial {
        &self.denial
    }
    /// The original issued basis, if native observation admission succeeded.
    pub fn source_observation(&self) -> Option<&WorthQueryApplicationReadObservation> {
        self.core.custody.basis.as_ref()
    }
    /// Recovers the actual original inputs. No request authority is retained.
    pub fn into_parts(
        self,
    ) -> (
        Demand<S, R>,
        WorthQueryOutputDemandControls,
        WorthQueryProductBranch,
        Option<WorthQueryApplicationReadObservation>,
        WorthQueryProgramApplicationRuntime<S, P>,
    ) {
        (
            self.core.custody.demand,
            self.core.custody.controls,
            self.core.branch,
            self.core.custody.basis,
            self.core.runtime,
        )
    }
}
impl<S, P, R> WorthQueryProgramOutputConstructionRefusal<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone,
    SourceIntent<S, R>: ApplicationQueryIntent<S, Binding = Source<S, R>>,
    SourceScope<S, R>: ApplicationQueryScopeResolution<S, SourcePrincipal<S, R>>,
    Value<S, R>: WorthQueryApplicationProjection<S, Query<S, R>> + Clone,
    R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
{
    pub fn retry(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryProgramOutputOperation<S, P, R>, Self> {
        construct(self.core, request)
    }
}
fn construct<S, P, R>(
    mut core: Core<S, P, R>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> Result<
    WorthQueryProgramOutputOperation<S, P, R>,
    WorthQueryProgramOutputConstructionRefusal<S, P, R>,
>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone,
    SourceIntent<S, R>: ApplicationQueryIntent<S, Binding = Source<S, R>>,
    SourceScope<S, R>: ApplicationQueryScopeResolution<S, SourcePrincipal<S, R>>,
    Value<S, R>: WorthQueryApplicationProjection<S, Query<S, R>> + Clone,
    R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
{
    use WorthQueryProgramOutputConstructionDenial as Denial;
    let result = if !core.matches(request) {
        Err(Denial::FreshRequestMismatch)
    } else if !core.runtime.contains_output_root::<R>() {
        Err(Denial::UndeclaredRoot)
    } else {
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                phase
                    .checked_execution_request_for(request.application.product_runtime())
                    .map_err(Denial::Execution)?;
                if core.custody.basis.is_none() {
                    core.custody.basis =
                        Some(request.retain_read(&phase).map_err(Denial::Observation)?);
                }
                core.runtime
                    .validate_selected_output_root::<R>(
                        &phase,
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        &core
                            .custody
                            .basis
                            .as_ref()
                            .expect("issued basis remains owned")
                            .retained,
                    )
                    .map_err(Denial::Program)
            })
            .map_err(Denial::Execution)
            .and_then(|result| result)
    };
    match result {
        Ok(()) => Ok(WorthQueryProgramOutputOperation { core }),
        Err(denial) => Err(WorthQueryProgramOutputConstructionRefusal { core, denial }),
    }
}
impl<S, D> WorthQueryApplicationOutputDemandRequest<'_, '_, '_, S, D>
where
    S: ApplicationSchema + 'static,
    D: WorthQueryApplicationOutputDemand<S> + Clone,
    <D::OutputFamily as WorthQueryProducerOutputFamily<S>>::Source: ApplicationQueryBinding<S>,
{
    /// Retains the original complete tree read request, never just its root demand.
    pub fn retain_program_outputs<P, R>(
        self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        WorthQueryProgramOutputOperation<S, P, R>,
        WorthQueryProgramOutputConstructionRefusal<S, P, R>,
    >
    where
        P: ApplicationProgramDefinition<S>,
        R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S, Demand = D>,
        SourceIntent<S, R>: ApplicationQueryIntent<S, Binding = Source<S, R>>,
        SourceScope<S, R>: ApplicationQueryScopeResolution<S, SourcePrincipal<S, R>>,
        Value<S, R>: WorthQueryApplicationProjection<S, Query<S, R>> + Clone,
        R::Dependents: ProgramOutputContinuationFactory<S, P, D>,
    {
        let request = WorthQueryApplicationRequest {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            branch: self.branch,
        };
        let core = Core {
            custody: ReadCustody {
                cleanup: None,
                tree: None,
                basis: self
                    .observation
                    .map(WorthQueryApplicationReadObservation::new),
                demand: self.demand,
                controls: self.controls.unwrap_or_default(),
            },
            branch: self.branch,
            runtime: program.clone(),
        };
        construct(core, &request)
    }
}
