//! Fixed history delegates actual complete-output/recovery and close owners.
use super::*;
use crate::application_entry::{
    history::{WorthQueryFixedHistoryEditOutcome, WorthQueryHistoryEditDenial},
    mutation::authorization::WorthQueryBindingAdmission,
};
use driver::*;
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationAdmissionBinding, NoApplicationMutationSource},
    application_program::ApplicationFixedDeliveryMode,
};
use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial as NativeDenial;

type O<S, I, P, R, A> = WorthQueryFixedOperation<S, I, P, R, A>;
type C<S, I, P, R, A> = WorthQuerySettledFixedOperation<S, I, P, R, A>;
type T<S, I, P, R, A> = WorthQueryTerminalFixedOperation<S, I, P, R, A>;
type CT<S, I> = WorthQueryClosedFixedTerminal<S, I>;
type F<S, I, P, R, A> = WorthQueryFixedHistoryEditOutcome<S, I, P, R, A>;

impl<S, I, P, A, R, Q> WorthQueryHistoryOperationMode<S, I, P, A>
    for ApplicationFixedDeliveryMode<R, Q>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R, Projection = Q>,
    Q: WorthQueryOutputProjectionInstallation<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    R::Dependents: crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<S,P,Demand<S,R>>,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>
        + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        WorthQueryBindingAdmission<S, I::Binding>,
    P::Outputs: ApplicationProgramOutputsShape<S>,
    MutationInput<S, I>: Clone + Send + Sync,
    MutationScope<S, I>: ApplicationMutationScopeResolution<S, MutationPrincipal<S, I>>,
    Demand<S, R>: Clone + Send,
    R::Dependents:worth_query_execution::publication_boundary::synchronous_outputs::SynchronousOutputEdges<S,Demand<S,R>>,
    SourceIntent<S, R>: ApplicationQueryIntent<S, Binding = Source<S, R>>,
    SourceScope<S, R>: ApplicationQueryScopeResolution<S, SourcePrincipal<S, R>>,
    SourceValue<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
{
    type Operation = O<S, I, P, R, A>;
    type Completed = C<S, I, P, R, A>;
    type Terminal = T<S, I, P, R, A>;
    type ClosedTerminal = CT<S, I>;
    type Outcome = F<S, I, P, R, A>;
    type GroupClosing = super::super::fixed_group_disposal::WorthQueryClosingFixedGroupOperation<S,I,P,R,A>;
    type GroupCompletion = WorthQueryClosedFixedDelivery<S,I,P,R,A>;

    fn validate_group_outputs(
        application: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), NativeDenial> {
        use crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory;
        application.validate_history_group_output::<Demand<S, R>>(
            &worth_query_execution::publication_boundary::program_publication_access(),
        )?;
        R::Dependents::validate_history_group_outputs(application)
    }

    fn begin_group_disposal(operation:Self::Operation)->Result<Self::GroupClosing,(Self::Operation,WorthQueryOperationCloseDenial)>{Self::GroupClosing::from_operation(operation)}
    fn begin_group_terminal_disposal(terminal:Self::Terminal)->Result<Self::GroupClosing,(Self::Terminal,WorthQueryOperationCloseDenial)>{Self::GroupClosing::from_terminal(terminal)}
    fn group_disposal_target(closing:&Self::GroupClosing)->worth_query_execution::facade::product::WorthQueryProductBranch{closing.target()}
    fn close_group_disposal(closing:Self::GroupClosing,request:&WorthQueryApplicationRequest<'_, '_, '_,S>,aborting:Option<&worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryGroupAborting>)->Result<WorthQueryClosedOperation,(Self::GroupClosing,WorthQueryOperationCloseDenial)>{closing.close(request,aborting)}
    fn finish_group_member(completed:Self::Completed,request:&WorthQueryApplicationRequest<'_, '_, '_,S>)->Result<Self::GroupCompletion,(Self::Completed,WorthQueryOperationCloseDenial)>{
        completed.close_for_delivery(request).map(|mut delivery|{delivery.delivery.description.history=None;delivery})
    }
    fn can_resume_group_cleanup(_: &Self::Operation)->bool{false}
}

impl<S, I, P, A, R, Q>
    Driver<
        S,
        I,
        P,
        A,
        O<S, I, P, R, A>,
        C<S, I, P, R, A>,
        T<S, I, P, R, A>,
        CT<S, I>,
        F<S, I, P, R, A>,
    > for ApplicationFixedDeliveryMode<R, Q>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R, Projection = Q>,
    Q: WorthQueryOutputProjectionInstallation<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    R::Dependents: crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<S,P,Demand<S,R>>,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>
        + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        WorthQueryBindingAdmission<S, I::Binding>,
    P::Outputs: ApplicationProgramOutputsShape<S>,
    MutationInput<S, I>: Clone + Send + Sync,
    MutationScope<S, I>: ApplicationMutationScopeResolution<S, MutationPrincipal<S, I>>,
    Demand<S, R>: Clone + Send,
    R::Dependents:worth_query_execution::publication_boundary::synchronous_outputs::SynchronousOutputEdges<S,Demand<S,R>>,
    SourceIntent<S, R>: ApplicationQueryIntent<S, Binding = Source<S, R>>,
    SourceScope<S, R>: ApplicationQueryScopeResolution<S, SourcePrincipal<S, R>>,
    SourceValue<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
{
    fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        O<S, I, P, R, A>,
        (
            WorthQueryOperationDescription<S, I>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        description.retain_fixed_native::<P, A>(runtime)
    }
    fn prepare<'a>(
        operation: &'a O<S, I, P, R, A>,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            crate::application_entry::mutation::authorization::PreparedMutation<S, I::Binding>,
            worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'a, S>,
        ),
        WorthQueryApplicationRequestMutationDenial,
    > {
        <I::Binding as SourceAssessmentDriver<S>>::prepare_history_description(
            &operation.description,
            &operation.runtime,
            &operation.installed_action,
            phase,
            request,
        )
    }
    fn prepare_completed<'a>(
        completed: &'a C<S, I, P, R, A>,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            crate::application_entry::mutation::authorization::PreparedMutation<S, I::Binding>,
            worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'a, S>,
        ),
        WorthQueryApplicationRequestMutationDenial,
    > {
        <I::Binding as SourceAssessmentDriver<S>>::prepare_history_description(
            &completed.delivery.description,
            &completed.delivery.runtime,
            &completed.delivery.installed_action,
            phase,
            request,
        )
    }
    fn target(
        operation: &O<S, I, P, R, A>,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        operation.description.branch
    }
    fn completed_target(
        completed: &C<S, I, P, R, A>,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        completed.delivery.description.branch
    }
    fn terminal_target(
        terminal: &T<S, I, P, R, A>,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        terminal.operation.description.branch
    }
    fn bind_candidate(
        mut operation: O<S, I, P, R, A>,
        constraint:worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryOperationConstraint,
    ) -> Result<O<S, I, P, R, A>, (O<S, I, P, R, A>, NativeDenial)> {
        if !matches!(operation.state, super::super::fixed::FixedState::Initial)
            || operation.description.history.is_some()
            || operation.description.source.is_some()
            || I::Binding::WORKFLOW_CONTROL
            || I::Binding::REQUIRES_WORKFLOW_AUTHORITY
            || I::Binding::workflow_execution_requirements().is_applicable()
        {
            return Err((operation, NativeDenial::UnsupportedOperation));
        }
        if operation.description.branch != constraint.original_source()
            || !operation
                .runtime
                .runtime()
                .product_runtime()
                .owns_branch_selector(constraint.candidate())
        {
            return Err((operation, NativeDenial::ForeignRecording));
        }
        operation.description.branch = constraint.candidate();
        operation.description.history = Some(constraint);
        Ok(operation)
    }
    fn advance(
        operation: O<S, I, P, R, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> HistoryOperationAdvance<O<S, I, P, R, A>, C<S, I, P, R, A>, T<S, I, P, R, A>> {
        match operation.advance(request) {
            WorthQueryOperationAdvance::Progress { operation, .. } => {
                HistoryOperationAdvance::Progress(operation)
            }
            WorthQueryOperationAdvance::Settled(completed) => {
                HistoryOperationAdvance::Completed(completed)
            }
            WorthQueryOperationAdvance::Refused { operation, denial } => {
                HistoryOperationAdvance::Refused {
                    operation,
                    denial: WorthQueryHistoryEditDenial::Fixed(denial),
                }
            }
            WorthQueryOperationAdvance::Terminal {
                mut operation,
                denial,
            } => {
                operation.advance_denial = Some(denial);
                HistoryOperationAdvance::Terminal(operation)
            }
        }
    }
    fn recording_source<'a>(completed: &'a C<S, I, P, R, A>, phase:&WorthQueryAdvancementPhase<'_>, request:&WorthQueryApplicationRequest<'_, '_, '_, S>) -> Result<Option<HistoryRecordingSource<'a>>,WorthQueryHistoryEditDenial> {
        Self::recording_source_at(completed, phase, request, completed.settlement().observation())
    }
    fn recording_source_at<'a>(completed: &'a C<S, I, P, R, A>, phase:&WorthQueryAdvancementPhase<'_>, request:&WorthQueryApplicationRequest<'_, '_, '_, S>, observation:&crate::application_entry::WorthQueryApplicationReadObservation) -> Result<Option<HistoryRecordingSource<'a>>,WorthQueryHistoryEditDenial> {
        let settlement=completed.settlement();
        let Some(source)=settlement.performed_source() else {return Ok(None)};
        let current=WorthQueryApplicationRequest {application:request.application,principal:request.principal,scope:request.scope,branch:completed.branch()};
        // The original completed observation proposes a basis. Only Native's
        // complete-set validation can qualify it; neither ordinal nor latest
        // candidate selection supplies that evidence.
        let retained=current.at(observation);
        let selected=completed.delivery.selected_action_at(&retained,phase)
            .map_err(WorthQueryHistoryEditDenial::Delivery)?;
        let qualified=selected.qualify_completed_required_outputs(phase,&worth_query_execution::publication_boundary::program_publication_access(),request.scope,
            source.receipt(),source.root_fulfillment(),settlement.root_settlement().retained_settlement(),
            settlement.retained_settlements().skip(1),settlement.dependencies.as_ref())
            .map_err(WorthQueryHistoryEditDenial::OutputQualification)?;
        Ok(Some(HistoryRecordingSource::RequiredOutputs {source:source.receipt(),completed:qualified}))
    }
    fn close_operation(
        operation: O<S, I, P, R, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (O<S, I, P, R, A>, WorthQueryOperationCloseDenial)> {
        operation.close(request)
    }
    fn discard_completed(
        completed: C<S, I, P, R, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), (C<S, I, P, R, A>, WorthQueryOperationCloseDenial)> {
        completed.close_for_delivery(request).map(drop)
    }
    fn finish_recorded(
        completed: C<S, I, P, R, A>,
        step: worth_query_execution::publication_boundary::document_history::WorthQueryHistoryStep,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<F<S, I, P, R, A>, (C<S, I, P, R, A>, WorthQueryOperationCloseDenial)> {
        completed.close_for_delivery(request).map(|mut delivery| {
            delivery.delivery.description.history = None;
            F::Recorded { step, delivery }
        })
    }
    fn finish_already(
        operation: O<S, I, P, R, A>,
        step: worth_query_execution::publication_boundary::document_history::WorthQueryHistoryStep,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<F<S, I, P, R, A>, (O<S, I, P, R, A>, WorthQueryOperationCloseDenial)> {
        operation
            .close(request)
            .map(|closed| F::AlreadyRecorded { step, closed })
    }
    fn close_terminal(
        terminal: T<S, I, P, R, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<CT<S, I>, (T<S, I, P, R, A>, WorthQueryOperationCloseDenial)> {
        terminal.close_with_evidence(request)
    }
    fn retained_terminal(_: &CT<S, I>) -> Option<&T<S, I, P, R, A>> {
        None
    }
    fn finish_terminal(terminal: CT<S, I>) -> F<S, I, P, R, A> {
        F::NotRecorded { terminal }
    }
}
