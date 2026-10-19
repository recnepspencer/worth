//! Declaration-selected advance signatures; both delegate the same native spine.
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
use worth_query_declaration::facade::application_program::{
    ApplicationRetainedCommit, ApplicationSourceCompletion,
};
impl<S, I, P, A> WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationSourceCompletion>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        super::super::super::mutation::authorization::WorthQueryBindingAdmission<S, I::Binding>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
{
    /// Advances once with fresh authority.
    pub fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> super::advancement::Advance<S, I, P, A, ApplicationSourceCompletion> {
        self.advance_with_history(request, None)
    }
}
impl<S, I, P, A> WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationRetainedCommit>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        super::super::super::mutation::authorization::WorthQueryBindingAdmission<S, I::Binding>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
{
    /// Retained delivery uses the same native history; `None` adds no work ceiling.
    /// ```compile_fail,E0061
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::*, application_program::*};
    /// fn attempt<'r, S, I, P, A>(owner: WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationRetainedCommit>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>, maximum_history_work: Option<std::num::NonZeroUsize>)
    /// where
    ///     S: ApplicationSchema + 'static,
    ///     I: ApplicationMutationIntent<S>,
    ///     P: ApplicationProgramDefinition<S>,
    ///     A: ApplicationAction<S, I::Binding>,
    ///     I::Binding: ApplicationMutationAdmissionBinding<S, Admission = ApplicationConventionalAdmission>,
    ///     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    ///     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
    ///         S,
    ///         <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    ///     >,
    /// { let _ = owner.advance(request); }
    /// ```
    /// ```
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::*, application_program::*};
    /// fn attempt<'r, S, I, P, A>(owner: WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationRetainedCommit>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>, maximum_history_work: Option<std::num::NonZeroUsize>)
    /// where
    ///     S: ApplicationSchema + 'static,
    ///     I: ApplicationMutationIntent<S>,
    ///     P: ApplicationProgramDefinition<S>,
    ///     A: ApplicationAction<S, I::Binding>,
    ///     I::Binding: ApplicationMutationAdmissionBinding<S, Admission = ApplicationConventionalAdmission>,
    ///     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    ///     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
    ///         S,
    ///         <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    ///     >,
    /// { let _ = owner.advance(request, maximum_history_work); }
    /// ```
    pub fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        maximum_history_work: Option<std::num::NonZeroUsize>,
    ) -> super::advancement::Advance<S, I, P, A, ApplicationRetainedCommit> {
        self.advance_with_history(request, maximum_history_work)
    }
}
