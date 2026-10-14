//! Effect-free owning construction resolves the exact installed action.
use super::descriptor::WorthQueryOperationDescription;
use super::fixed::FixedState;
use super::*;

#[derive(Debug)]
pub enum WorthQueryOperationConstructionDenial {
    Action(worth_query_installation::facade::WorthQueryActionResolutionDenial),
    UndeclaredRoot,
    ForeignRuntime,
    Query(worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial),
}

impl<Schema: ApplicationSchema, Intent: ApplicationMutationIntent<Schema>>
    WorthQueryApplicationMutationRequest<
        '_,
        '_,
        '_,
        Schema,
        Intent,
        WorthQueryMutationSourcePrepared,
    >
{
    pub fn idempotency_owned(
        self,
        key: <Intent::Binding as ApplicationMutationBinding<Schema>>::IdempotencyKey,
    ) -> WorthQueryOperationDescription<Schema, Intent> {
        WorthQueryOperationDescription {
            intent: self.intent,
            key,
            preconditions: self.preconditions,
            source: self.source,
            branch: self.branch,
        }
    }
}

impl<Schema: ApplicationSchema + 'static, Intent: ApplicationMutationIntent<Schema>>
    WorthQueryOperationDescription<Schema, Intent>
{
    pub fn retain_fixed<'runtime, Program, Action>(
        self,
        program: &'runtime WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryFixedOperation<
            'runtime,
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
        Action: ApplicationFixedAction<Schema, Binding<Schema, Intent>>,
        Delivery<Schema, Intent, Action>:
            ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>>,
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
            .resolve_fixed_action::<Intent::Binding, Action>()
        {
            Ok(action) => action,
            Err(denial) => {
                return Err((self, WorthQueryOperationConstructionDenial::Action(denial)))
            }
        };
        if let Err(denial) = program
            .runtime()
            .installed_schema()
            .installed_query_binding::<Read<Schema, Intent, Action>>()
        {
            return Err((self, WorthQueryOperationConstructionDenial::Query(denial)));
        }
        if !program.contains_output_root::<FixedRoot<Schema, Intent, Action>>() {
            return Err((self, WorthQueryOperationConstructionDenial::UndeclaredRoot));
        }
        Ok(WorthQueryFixedOperation {
            installed_action,
            runtime: program,
            description: self,
            state: FixedState::Initial,
            initial_cause: None,
            recovered: None,
            result: None,
            cleanup: Vec::new(),
            action: std::marker::PhantomData,
        })
    }
}
