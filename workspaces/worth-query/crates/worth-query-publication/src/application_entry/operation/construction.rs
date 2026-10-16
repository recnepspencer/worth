//! Effect-free owning construction resolves the exact installed action.
use super::descriptor::WorthQueryOperationDescription;
use super::fixed::FixedState;
use super::*;

#[derive(Debug)]
/// Explains construction refusal; the caller receives its original operation description.
pub enum WorthQueryOperationConstructionDenial {
    Action(worth_query_installation::facade::WorthQueryActionResolutionDenial),
    UndeclaredRoot,
    RequiredOutputSource,
    OutputSource(worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial),
    ForeignRuntime,
    Query(worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial),
}

impl<Schema: ApplicationSchema, Intent: ApplicationMutationIntent<Schema>>
    crate::application_entry::WorthQueryOperationBuilder<
        '_,
        '_,
        '_,
        Schema,
        Intent,
        WorthQueryMutationSourcePrepared,
    >
{
    /// Freezes one typed payload under this initiating request's native resources.
    /// Refusal returns the entire original builder and owned key before any
    /// business effect or workflow-slot transfer. Retry requires a fresh opening.
    /// Shared encoded inputs keep this payload's charge through their last owner;
    /// nested caller Clone heaps and Arc metadata are outside that native quote.
    pub fn idempotency_owned(
        self,
        key: <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    ) -> Result<
        WorthQueryOperationDescription<Schema, Intent>,
        (
            Self,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
            super::WorthQueryOperationInputCaptureDenial,
        ),
    >
    where
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    {
        use super::WorthQueryOperationInputCaptureDenial as Denial;
        let runtime = self.native.application;
        let captured = runtime.with_application_advancement(self.native.scope, |phase| {
            runtime.capture_operation_input_in_advancement::<<Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding>(
                &phase, &worth_query_execution::publication_boundary::program_publication_access(),
                self.native.scope, self.native.intent.input())
        }).map_err(Denial::Execution).and_then(|result| result);
        let encoded_input = match captured {
            Ok(input) => input,
            Err(denial) => return Err((self, key, denial)),
        };
        let self_native = self.native;
        Ok(WorthQueryOperationDescription {
            intent: self_native.intent,
            encoded_input: Ok(encoded_input),
            key,
            preconditions: self_native.preconditions,
            source: self_native.source,
            branch: self_native.branch,
        })
    }
}

impl<Schema: ApplicationSchema + 'static, Intent: ApplicationMutationIntent<Schema>>
    WorthQueryOperationDescription<Schema, Intent>
{
    pub(super) fn retain_fixed_native<Program, Action>(
        self,
        program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryFixedOperation<
            Schema,
            Intent,
            Program,
            FixedRoot<Schema, Intent, Action>,
            Action,
        >,
        (Self, WorthQueryOperationConstructionDenial),
    >
    where
        Schema: ApplicationSchema + 'static,
        Intent: ApplicationMutationIntent<Schema>,
        Program: ApplicationProgramDefinition<Schema>,
        FixedRoot<Schema, Intent, Action>:
            ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
        Connection<Schema, FixedRoot<Schema, Intent, Action>>:
            WorthQueryApplicationRequiredOutputConnection<Schema>,
        Binding<Schema, Intent>: WorthQueryApplicationRequiredOutputSource<
            Schema,
            Connection<Schema, FixedRoot<Schema, Intent, Action>>,
        >,
        Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
        Delivery<Schema, Intent, Action>:
            ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>>,
        <Delivery<Schema, Intent, Action> as ApplicationFixedActionDelivery<
            Schema,
            Binding<Schema, Intent>,
        >>::Projection: WorthQueryOutputProjectionInstallation<Schema>,
    {
        if !program
            .runtime()
            .product_runtime()
            .owns_branch_selector(self.branch)
        {
            return Err((self, WorthQueryOperationConstructionDenial::ForeignRuntime));
        }
        let installed_action = match program
            .installed_program()
            .resolve_action::<Intent::Binding, Action>()
        {
            Ok(action) => action,
            Err(denial) => {
                return Err((self, WorthQueryOperationConstructionDenial::Action(denial)));
            }
        };
        if !installed_action.required_output_source()
            || installed_action
                .delivery()
                .and_then(|delivery| delivery.fixed_output())
                .is_none()
        {
            return Err((self, WorthQueryOperationConstructionDenial::Action(
                worth_query_installation::facade::WorthQueryActionResolutionDenial::DeliveryMismatch,
            )));
        }
        if let Err(denial) =
            <Delivery<Schema, Intent, Action> as ApplicationFixedActionDelivery<
                Schema,
                Binding<Schema, Intent>,
            >>::Projection::require_installed(program.runtime().installed_schema())
        {
            return Err((self, WorthQueryOperationConstructionDenial::Query(denial)));
        }
        if !program.contains_output_root::<FixedRoot<Schema, Intent, Action>>() {
            return Err((self, WorthQueryOperationConstructionDenial::UndeclaredRoot));
        }
        Ok(WorthQueryFixedOperation {
            installed_action: installed_action.clone(),
            runtime: program.clone(),
            description: self,
            state: FixedState::Initial,
            initial_cause: None,
            recovered: None,
            result: None,
            cleanup: WorthQueryOperationCleanup::default(),
            action: std::marker::PhantomData,
        })
    }
}
