//! Root-free exact installed construction and borrowed native SourceOnly handoffs.
use super::super::mutation::authorization::WorthQueryBindingAdmission;
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
use worth_query_declaration::facade::application_program::{
    ApplicationAction, ApplicationRetainedCommit, ApplicationSourceCompletion,
};
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramOwner, WorthQuerySelectedProgramOwner,
};

mod sealed {
    pub trait Mode {}
}
impl sealed::Mode for ApplicationSourceCompletion {}
impl sealed::Mode for ApplicationRetainedCommit {}
/// Sealed no-output delivery choices preserve ordinary versus retained native commit outcomes.
pub trait WorthQuerySourceDeliveryMode: sealed::Mode + 'static {
    const RETAIN_COMMIT: bool;
}
impl WorthQuerySourceDeliveryMode for ApplicationSourceCompletion {
    const RETAIN_COMMIT: bool = false;
}
impl WorthQuerySourceDeliveryMode for ApplicationRetainedCommit {
    const RETAIN_COMMIT: bool = true;
}
/// Current native handoff for the SourceOnly adapter; no request authority is retained.
/// ```compile_fail,E0308
/// use worth_query_publication::facade::application_entry::WorthQuerySourceOperationAdmission;
/// use worth_query_declaration::facade::{application_operation::*, application_program::*, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::{application_installation::WorthQueryProgramApplicationRuntime, primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource}};
/// fn cannot_erase_required_output<S, I, P, R, A>(fixed: worth_query_publication::facade::application_entry::WorthQueryFixedOperation<S, I, P, R, A>) -> WorthQuerySourceOperationAdmission<S, I, P, A, worth_query_declaration::facade::application_program::ApplicationSourceCompletion>
/// where
///     S: ApplicationSchema,
///     I: ApplicationMutationIntent<S>,
///     P: ApplicationProgramDefinition<S>,
///     R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
///     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
///     I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
///     A: ApplicationAction<S, I::Binding>,
///     A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
/// {
///     fixed
/// }
/// ```
/// The fixed twin keeps its real native custody.
/// ```
/// use worth_query_publication::facade::application_entry::WorthQuerySourceOperationAdmission;
/// use worth_query_declaration::facade::{application_operation::*, application_program::*, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::{application_installation::WorthQueryProgramApplicationRuntime, primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource}};
/// fn preserve_fixed_custody<S, I, P, R, A>(fixed: worth_query_publication::facade::application_entry::WorthQueryFixedOperation<S, I, P, R, A>) -> worth_query_publication::facade::application_entry::WorthQueryFixedOperation<S, I, P, R, A>
/// where
///     S: ApplicationSchema,
///     I: ApplicationMutationIntent<S>,
///     P: ApplicationProgramDefinition<S>,
///     R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
///     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
///     I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
///     A: ApplicationAction<S, I::Binding>,
///     A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
/// {
///     fixed
/// }
/// ```
pub struct WorthQuerySourceOperationAdmission<S, I, P, A, Mode>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry) description: WorthQueryOperationDescription<S, I>,
    pub(in crate::application_entry) installed_action:
        worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
    pub(super) workflow: Option<
        worth_query_execution::publication_boundary::workflow_advance::RequiredWorkflowOperation,
    >,
    marker: std::marker::PhantomData<fn() -> (A, Mode)>,
    // Retention outlives captured intent and native workflow custody on abandonment.
    pub(in crate::application_entry) runtime: WorthQueryProgramApplicationRuntime<S, P>,
}
impl<S, I, P, A, Mode> WorthQuerySourceOperationAdmission<S, I, P, A, Mode>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode: WorthQuerySourceDeliveryMode,
{
    pub(super) fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        Self,
        (
            WorthQueryOperationDescription<S, I>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        if !runtime
            .runtime()
            .product_runtime()
            .owns_branch_selector(description.branch)
        {
            return Err((
                description,
                WorthQueryOperationConstructionDenial::ForeignRuntime,
            ));
        }
        let installed_action = match runtime
            .installed_program()
            .resolve_action::<I::Binding, A>()
        {
            Ok(action) => action,
            Err(denial) => {
                return Err((
                    description,
                    WorthQueryOperationConstructionDenial::Action(denial),
                ));
            }
        };
        if installed_action.required_output_source()
            || runtime.owns_output_source(std::any::TypeId::of::<I::Binding>())
        {
            return Err((
                description,
                WorthQueryOperationConstructionDenial::RequiredOutputSource,
            ));
        }
        Ok(Self {
            runtime: runtime.clone(),
            description,
            installed_action: installed_action.clone(),
            workflow: None,
            marker: std::marker::PhantomData,
        })
    }
    pub(super) fn select<'runtime>(
        &'runtime self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
                'runtime,
                S,
            >,
            WorthQuerySelectedProgramOwner<'runtime, S>,
        ),
        WorthQueryApplicationRequestMutationDenial,
    > {
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch);
        }
        let selected = self
            .runtime
            .runtime()
            .on_branch(self.description.branch)
            .select(phase)
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
        let owner = self
            .runtime
            .selected_program_owner(&selected)
            .map_err(super::super::mutation::selected_program::map_selected_program_owner_denial)?;
        owner
            .resolve_declared_action::<I::Binding>(&self.installed_action)
            .map_err(|_| WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired)?;
        if owner.owns_output_source(std::any::TypeId::of::<I::Binding>()) {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        Ok((selected, owner))
    }
}
impl<S, I, P, A, Mode> WorthQuerySourceOperationAdmission<S, I, P, A, Mode>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode: WorthQuerySourceDeliveryMode,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        WorthQueryBindingAdmission<S, I::Binding>,
    MutationInput<S, I>: Clone + Send + Sync,
    MutationScope<S, I>: ApplicationMutationScopeResolution<S, MutationPrincipal<S, I>>,
{
    pub fn execute_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryApplicationRetainedMutationOutcome<
            <I::Binding as ApplicationMutationBinding<S>>::Denial,
            <I::Binding as ApplicationMutationBinding<S>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let (selected, owner) = self.select(phase, request)?;
        let mut native = self
            .description
            .request_view(request, &self.installed_action);
        self.bind_workflow_view(&mut native, true);
        let execution = phase
            .checked_execution_request_for(self.runtime.runtime().product_runtime())
            .map_err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest)?;
        let allocation = execution.allocation_policy();
        if Mode::RETAIN_COMMIT {
            native.execute_retained_with_commit(
                phase,
                |native, identities, staged| <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission::prepare_selected(phase, native, identities, staged, &selected),
                |_, program, binding| {
                    owner.commit_program_action_retained_in_advancement(
                        phase,
                        program,
                        binding.identities(),
                        |idempotency| binding.extension().apply(idempotency),
                        allocation,
                    )
                },
                allocation,
            )
        } else {
            native
                .execute_with_preparation_and_commit(
                    phase,
                    |native, identities, staged| <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission::prepare_selected(phase, native, identities, staged, &selected),
                    |_, program, binding| {
                        owner.commit_program_action_in_advancement(
                            phase,
                            program,
                            binding.identities(),
                            |idempotency| binding.extension().apply(idempotency),
                            allocation,
                        )
                    },
                    allocation,
                )
                .map(WorthQueryApplicationRetainedMutationOutcome::Other)
        }
    }

    pub fn recover_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        recovery: &worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecovery,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryApplicationRecoveryRequestDenial,
    > {
        self.select(phase, request)?;
        let mut native = self
            .description
            .request_view(request, &self.installed_action);
        let prepared = self.authorize_recovery_view(phase, request, &mut native)?;
        self.runtime
            .runtime()
            .recover_admitted_unpublished_source_operation(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                recovery,
                &prepared.admission,
                prepared.idempotency,
            )
            .map_err(WorthQueryApplicationRecoveryRequestDenial::Recovery)
    }
    pub fn resolve_keyed_in_advancement(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryAdmittedIdempotencyRead,
        WorthQueryApplicationRecoveryRequestDenial,
    > {
        self.select(phase, request)?;
        let mut native = self
            .description
            .request_view(request, &self.installed_action);
        let prepared = self.authorize_recovery_view(phase, request, &mut native)?;
        native.read_recovery_idempotency(phase, &prepared)
    }
}

#[path = "workflow/admission.rs"]
mod workflow_admission;
