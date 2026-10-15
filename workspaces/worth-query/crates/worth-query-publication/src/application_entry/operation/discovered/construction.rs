//! Effect-free discovered construction resolves real installed action, root and projection.
use super::*;
impl<'r, S, I, P, R, A> WorthQueryDiscoveredOperation<'r, S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationDiscoveredAction<S, I::Binding>,
    ActionDelivery<S, I, A>: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
{
    pub(in crate::application_entry::operation) fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &'r WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        Self,
        (
            WorthQueryOperationDescription<S, I>,
            WorthQueryOperationConstructionDenial,
        ),
    >
    where
        <ActionDelivery<S, I, A> as ApplicationDiscoveredActionDelivery<S, I::Binding>>::Projection:
            WorthQueryOutputProjectionInstallation<S>,
    {
        use WorthQueryOperationConstructionDenial as Denial;
        if !runtime
            .runtime()
            .product_runtime()
            .owns_branch_selector(description.branch)
        {
            return Err((description, Denial::ForeignRuntime));
        }
        let installed_action = match runtime
            .installed_program()
            .resolve_discovered_action::<I::Binding, A>()
        {
            Ok(action) => action,
            Err(denial) => return Err((description, Denial::Action(denial))),
        };
        if !runtime.contains_output_root::<R>() {
            return Err((description, Denial::UndeclaredRoot));
        }
        if let Err(denial) = runtime
            .validate_program_discovered_root_artifact_source::<R, I::Binding>(
                &worth_query_execution::publication_boundary::program_publication_access(),
            )
        {
            return Err((description, Denial::OutputSource(denial)));
        }
        if let Err(denial) = <ActionDelivery<S, I, A> as ApplicationDiscoveredActionDelivery<
            S,
            I::Binding,
        >>::Projection::require_installed(
            runtime.runtime().installed_schema()
        ) {
            return Err((description, Denial::Query(denial)));
        }
        Ok(Self {
            runtime,
            installed_action,
            description,
            state: State::Initial,
            initial_cause: None,
            result: None,
            recovered: None,
            prior_cleanup: Vec::new(),
            cleanup: WorthQueryOperationCleanup::default(),
            action: std::marker::PhantomData,
        })
    }
}
