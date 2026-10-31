//! Fresh enrollment and cached-step lookup use declared metadata authority.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_history::ApplicationHistoryModel,
    application_operation::{ApplicationMutationScopeBinding, NoApplicationMutationSource},
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};

impl<'runtime, S, H, I, P, A, Payload: WorthQueryHistoryPayloadMode<S, H>>
    WorthQueryHistoryEdit<'runtime, S, H, I, P, A, Payload>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: crate::application_entry::WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
        ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    /// Supply current metadata authority for fresh attempt enrollment or lookup.
    /// Later phases retain their original operation and result custody.
    pub fn advance_with_metadata<C, O, Input>(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        capability: ApplicationCapabilityRef<S, C>,
        operation: ApplicationOperationRef<S, O, Input>,
        input: Input,
    ) -> WorthQueryHistoryEditAdvance<Self>
    where
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<S, C, Scope = ApplicationHistoryModel<S, H>> + 'static,
    {
        if !std::ptr::eq(self.document.owner.application(), request.application) {
            return WorthQueryHistoryEditAdvance::Refused {
                edit: self,
                denial: WorthQueryHistoryEditDenial::History(
                    WorthQueryApplicationHistoryDenial::ForeignRuntime,
                ),
            };
        }
        if matches!(&self.phase, Some(Phase::Admission(_))) {
            self.advance_admission_with(request, |phase, runtime, read| {
                read.with_metadata_capability(runtime, phase, capability, operation, input)
            })
        } else {
            self.advance(request)
        }
    }
}
