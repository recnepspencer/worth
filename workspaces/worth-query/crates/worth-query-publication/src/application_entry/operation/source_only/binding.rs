//! Sealed native dispatch lets generic hosts call the real owner, not supplier callbacks.
use super::*;
use crate::application_entry::mutation::{
    authorization::WorthQueryBindingAdmission, WorthQueryCurrentAuthorizationAssessment,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;

/// Native implementation availability for ordinary source owners. This is not
/// installation, no-output eligibility, authorization or an execution grant.
/// The installed action and every fresh request still pass their native owners.
pub trait WorthQuerySourceOperationBinding<S: ApplicationSchema>:
    ApplicationMutationAdmissionBinding<S> + sealed::Driver<S>
{
}
impl<S, B> WorthQuerySourceOperationBinding<S> for B
where
    S: ApplicationSchema,
    B: ApplicationMutationAdmissionBinding<S> + sealed::Driver<S>,
{
}

pub(super) mod sealed {
    use super::*;
    pub trait Driver<S: ApplicationSchema>: ApplicationMutationAdmissionBinding<S> {
        fn prepare_history<'owner, I, P, A, M>(
            owner: &'owner WorthQuerySourceOnlyOperation<S, I, P, A, M>,
            phase: &WorthQueryAdvancementPhase<'_>,
            request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        ) -> Result<
            (
                crate::application_entry::mutation::authorization::PreparedMutation<S, Self>,
                worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
                    'owner,
                    S,
                >,
            ),
            WorthQueryApplicationRequestMutationDenial,
        >
        where
            S: 'static,
            I: ApplicationMutationIntent<S, Binding = Self>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, Self>,
            M: WorthQuerySourceDeliveryMode,
            Self: Sized;

        fn admit_for_assessment<'owner, I, P, A, M>(
            owner: &'owner WorthQuerySourceOnlyOperation<S, I, P, A, M>,
            phase: &WorthQueryAdvancementPhase<'_>,
            request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        ) -> Result<
            (
                crate::application_entry::mutation::authorization::AdmittedMutation<S, Self>,
                worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
                    'owner,
                    S,
                >,
            ),
            WorthQueryApplicationRequestMutationDenial,
        >
        where
            S: 'static,
            I: ApplicationMutationIntent<S, Binding = Self>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, Self>,
            M: WorthQuerySourceDeliveryMode,
            Self: Sized;

        fn advance<I, P, A, M>(
            owner: WorthQuerySourceOnlyOperation<S, I, P, A, M>,
            request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
            history: Option<std::num::NonZeroUsize>,
        ) -> super::super::advancement::Advance<S, I, P, A, M>
        where
            S: 'static,
            I: ApplicationMutationIntent<S, Binding = Self>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, Self>,
            M: WorthQuerySourceDeliveryMode,
            Self: Sized;
        fn assess<I, P, A, M>(
            owner: &WorthQuerySourceOnlyOperation<S, I, P, A, M>,
            request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        ) -> Result<
            WorthQueryCurrentAuthorizationAssessment,
            WorthQueryApplicationRequestMutationDenial,
        >
        where
            S: 'static,
            I: ApplicationMutationIntent<S, Binding = Self>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, Self>,
            M: WorthQuerySourceDeliveryMode,
            Self: Sized;
    }
    impl<S, B> Driver<S> for B
    where
        S: ApplicationSchema,
        B: ApplicationMutationAdmissionBinding<S>,
        B::Admission: WorthQueryBindingAdmission<S, B>,
        B::Input: Clone + Send + Sync,
        B::ScopeBinding: ApplicationMutationScopeResolution<S, B::PrincipalIdentity>,
    {
        fn prepare_history<'owner, I, P, A, M>(
            owner: &'owner WorthQuerySourceOnlyOperation<S, I, P, A, M>,
            phase: &WorthQueryAdvancementPhase<'_>,
            request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        ) -> Result<
            (
                crate::application_entry::mutation::authorization::PreparedMutation<S, Self>,
                worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
                    'owner,
                    S,
                >,
            ),
            WorthQueryApplicationRequestMutationDenial,
        >
        where
            S: 'static,
            I: ApplicationMutationIntent<S, Binding = Self>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, Self>,
            M: WorthQuerySourceDeliveryMode,
        {
            owner.admission.prepare_history_source(phase, request)
        }

        fn admit_for_assessment<'owner, I, P, A, M>(
            owner: &'owner WorthQuerySourceOnlyOperation<S, I, P, A, M>,
            phase: &WorthQueryAdvancementPhase<'_>,
            request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        ) -> Result<
            (
                crate::application_entry::mutation::authorization::AdmittedMutation<S, Self>,
                worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
                    'owner,
                    S,
                >,
            ),
            WorthQueryApplicationRequestMutationDenial,
        >
        where
            S: 'static,
            I: ApplicationMutationIntent<S, Binding = Self>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, Self>,
            M: WorthQuerySourceDeliveryMode,
        {
            owner.admission.admit_for_assessment(phase, request)
        }

        fn advance<I, P, A, M>(
            owner: WorthQuerySourceOnlyOperation<S, I, P, A, M>,
            request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
            history: Option<std::num::NonZeroUsize>,
        ) -> super::super::advancement::Advance<S, I, P, A, M>
        where
            S: 'static,
            I: ApplicationMutationIntent<S, Binding = Self>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, Self>,
            M: WorthQuerySourceDeliveryMode,
        {
            owner.advance_with_history(request, history)
        }
        fn assess<I, P, A, M>(
            owner: &WorthQuerySourceOnlyOperation<S, I, P, A, M>,
            request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        ) -> Result<
            WorthQueryCurrentAuthorizationAssessment,
            WorthQueryApplicationRequestMutationDenial,
        >
        where
            S: 'static,
            I: ApplicationMutationIntent<S, Binding = Self>,
            P: ApplicationProgramDefinition<S>,
            A: ApplicationAction<S, Self>,
            M: WorthQuerySourceDeliveryMode,
        {
            owner.admission.assess_current_authorization(request)
        }
    }
}
