//! Supplier preparation delegates execution and inspection to the actual Query owner.
use crate::{
    WorthServerAdmittedTransportCaller, WorthServerProductAdapterExecutionError,
    WorthServerProductOperationPayload,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
    },
    application_entry::*,
    application_installation::WorthQueryProgramApplicationRuntime,
    declaration::{
        application_operation::*, application_program::*, application_schema::ApplicationSchema,
    },
};

/// One installed source action and its input preparation, not an execution engine.
/// Native lifecycle and authorization calls are made by Server directly; a
/// supplier cannot replace them with an independent callback implementation.
pub trait WorthServerQuerySourceAdapter: Send + Sync + 'static {
    type Schema: ApplicationSchema + Send + Sync + 'static;
    type Program: ApplicationProgramDefinition<Self::Schema> + Send + Sync;
    type Binding: WorthQuerySourceOperationBinding<Self::Schema>
        + ApplicationMutationAdmissionBinding<
            Self::Schema,
            Input: Clone + Send + Sync,
            ScopeBinding: ApplicationMutationScopeResolution<
                Self::Schema,
                <Self::Binding as ApplicationMutationBinding<Self::Schema>>::PrincipalIdentity,
            >,
        >;
    type Intent: ApplicationMutationIntent<Self::Schema, Binding = Self::Binding>;
    type Delivery: ApplicationActionDelivery<
        Self::Schema,
        Self::Binding,
        Mode = ApplicationSourceCompletion,
    >;
    type Action: ApplicationAction<Self::Schema, Self::Binding, Delivery = Self::Delivery>;
    type Input: serde::de::DeserializeOwned + schemars::JsonSchema;
    /// Safe supplier data projected from the completed owner's exact original input.
    /// It is not a native result, receipt or execution authority.
    type CompletionData: serde::Serialize + schemars::JsonSchema;
    type Preparation;
    const INPUT: &'static str;
    const RESULT: &'static str;
    const NAMESPACE: &'static str;

    fn program(&self) -> &WorthQueryProgramApplicationRuntime<Self::Schema, Self::Program>;
    fn authenticate(
        &self,
        caller: &WorthServerAdmittedTransportCaller,
        scope: &WorthQueryRequestScope,
        payload: &WorthServerProductOperationPayload,
    ) -> Result<
        WorthQueryAuthenticatedExternalPrincipal<Self::Schema>,
        WorthServerProductAdapterExecutionError,
    >;
    fn begin(input: Self::Input, original_basis: Option<&str>) -> Self::Preparation;
    fn preparation(
        preparation: &Self::Preparation,
    ) -> (WorthServerSourcePreparationPhase, bool, Option<String>);
    /// Some preparations already have an ordinary original-input authorization
    /// entry. None means no native authorization observation is available, not allow.
    fn assess_preparation(
        &self,
        preparation: &Self::Preparation,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Self::Schema>,
    ) -> Option<
        Result<
            WorthQueryCurrentAuthorizationAssessment,
            WorthQueryApplicationRequestMutationDenial,
        >,
    >;
    fn prepare(
        &self,
        preparation: Self::Preparation,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Self::Schema>,
        scope: &WorthQueryRequestScope,
        key: &str,
    ) -> Result<WorthServerSourceOperation<Self>, Box<Self::Preparation>>
    where
        Self: Sized;
    /// Safe domain correspondence only. Never include record values or credentials.
    fn domain_diagnostic(
        denial: &<Self::Binding as ApplicationMutationBinding<Self::Schema>>::Denial,
    ) -> super::diagnostic::WorthServerSourceDiagnostic;

    fn completion_data(
        input: &<Self::Binding as ApplicationMutationBinding<Self::Schema>>::Input,
    ) -> Self::CompletionData;
}

pub type WorthServerSourceOperation<C> = WorthQuerySourceOnlyOperation<
    <C as WorthServerQuerySourceAdapter>::Schema,
    <C as WorthServerQuerySourceAdapter>::Intent,
    <C as WorthServerQuerySourceAdapter>::Program,
    <C as WorthServerQuerySourceAdapter>::Action,
    ApplicationSourceCompletion,
>;
pub(super) type WorthServerCompletedSource<C> = WorthQueryCompletedSourceOperation<
    <C as WorthServerQuerySourceAdapter>::Schema,
    <C as WorthServerQuerySourceAdapter>::Intent,
    <C as WorthServerQuerySourceAdapter>::Program,
    <C as WorthServerQuerySourceAdapter>::Action,
    ApplicationSourceCompletion,
>;
pub(super) type WorthServerTerminalSource<C> = WorthQueryTerminalSourceOperation<
    <C as WorthServerQuerySourceAdapter>::Schema,
    <C as WorthServerQuerySourceAdapter>::Intent,
    <C as WorthServerQuerySourceAdapter>::Program,
    <C as WorthServerQuerySourceAdapter>::Action,
    ApplicationSourceCompletion,
>;
#[derive(serde::Serialize, schemars::JsonSchema)]
pub enum WorthServerSourcePreparationPhase {
    Resolving,
    Captured,
    Admitted,
    ConstructionRefused,
    SourceObservation,
}

pub(super) fn operation<C: WorthServerQuerySourceAdapter>() -> &'static str {
    use worth_query_host::facade::declaration::application_operation::ApplicationMutationBinding;
    C::Binding::IDENTITY
}
