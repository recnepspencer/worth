use worth_query_declaration::facade::{
    application_operation::{
        ApplicationCandidateRequirements, ApplicationMutationBinding,
        ApplicationMutationBindingDescriptor, ApplicationMutationScopeContract,
    },
    application_schema::{ApplicationOperationRef, ApplicationSchema},
};

use crate::{
    application_principal_binding::WorthQueryInstalledPrincipalBinding,
    application_schema::WorthQueryInstalledApplicationSchema,
};

use super::super::{
    WorthQueryApplicationOperationInstallationDenial,
    WorthQueryApplicationOperationInstallationDenialKind as DenialKind,
    WorthQueryInstalledApplicationOperation,
};
use super::WorthQueryCompiledApplicationMutationBinding;

pub type WorthQueryInstalledBoundMutationOperation<Schema, Binding> =
    WorthQueryInstalledApplicationOperation<
        Schema,
        <Binding as ApplicationMutationBinding<Schema>>::Operation,
        <Binding as ApplicationMutationBinding<Schema>>::Input,
    >;

pub type WorthQueryInstalledBoundMutationPrincipal<Schema, Binding> =
    WorthQueryInstalledPrincipalBinding<
        Schema,
        <Binding as ApplicationMutationBinding<Schema>>::PrincipalBinding,
        <Binding as ApplicationMutationBinding<Schema>>::Mapping,
        <Binding as ApplicationMutationBinding<Schema>>::Principal,
        <Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        <Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentityBinding,
    >;

/// Callback-free installed meaning for one exact application mutation binding.
pub struct WorthQueryInstalledApplicationMutationBinding<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    operation: WorthQueryInstalledBoundMutationOperation<Schema, Binding>,
    principal: WorthQueryInstalledBoundMutationPrincipal<Schema, Binding>,
    meaning: Arc<WorthQueryCompiledApplicationMutationBinding>,
}

impl<Schema, Binding> WorthQueryInstalledApplicationMutationBinding<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub fn identity(&self) -> &str {
        Binding::IDENTITY
    }

    pub fn operation(&self) -> &WorthQueryInstalledBoundMutationOperation<Schema, Binding> {
        &self.operation
    }

    pub fn into_operation(self) -> WorthQueryInstalledBoundMutationOperation<Schema, Binding> {
        self.operation
    }

    pub fn principal_binding(&self) -> &WorthQueryInstalledBoundMutationPrincipal<Schema, Binding> {
        &self.principal
    }

    pub fn scope_contract(&self) -> &ApplicationMutationScopeContract {
        self.meaning.scope()
    }

    pub fn candidate_requirements(&self) -> ApplicationCandidateRequirements {
        self.meaning.candidates()
    }

    pub fn result_identity(&self) -> &str {
        self.meaning.descriptor().result_identity().as_str()
    }

    /// Stable handler identity used by the execution owner to bind custody.
    /// Installation retains no handler value or callback.
    pub fn handler_identity(&self) -> &str {
        self.meaning.descriptor().handler().identity()
    }

    pub fn idempotency_identity(&self) -> &str {
        self.meaning.descriptor().idempotency().identity()
    }
}

impl<Schema> WorthQueryInstalledApplicationSchema<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn installed_mutation_binding<Binding>(
        &self,
    ) -> Result<
        WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        WorthQueryApplicationOperationInstallationDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let requested = Binding::descriptor();
        let installed = self
            .mutation_catalog
            .get_binding(Binding::IDENTITY)
            .ok_or_else(|| denial(DenialKind::MutationBindingNotInstalled, Binding::IDENTITY))?;
        validate_requested_meaning(installed.descriptor(), &requested)?;
        let operation = self.installed_operation(ApplicationOperationRef::<
            Schema,
            Binding::Operation,
            Binding::Input,
        >::from_declaration())?;
        let principal = self
            .principal_binding(Binding::principal_binding())
            .map_err(|_| {
                denial(
                    DenialKind::MutationBindingPrincipalMeaningChanged,
                    Binding::IDENTITY,
                )
            })?;
        Ok(WorthQueryInstalledApplicationMutationBinding {
            operation,
            principal,
            meaning: installed,
        })
    }
}

fn validate_requested_meaning(
    installed: &ApplicationMutationBindingDescriptor,
    requested: &ApplicationMutationBindingDescriptor,
) -> Result<(), WorthQueryApplicationOperationInstallationDenial> {
    if installed.operation_name() != requested.operation_name()
        || installed.operation_type() != requested.operation_type()
        || installed.input_identity() != requested.input_identity()
        || installed.input_binding_type() != requested.input_binding_type()
        || installed.input_type() != requested.input_type()
    {
        return Err(denial(
            DenialKind::MutationBindingOperationMeaningChanged,
            requested.identity(),
        ));
    }
    if installed.principal() != requested.principal() {
        return Err(denial(
            DenialKind::MutationBindingPrincipalMeaningChanged,
            requested.identity(),
        ));
    }
    if installed.result_identity() != requested.result_identity()
        || installed.result_binding_type() != requested.result_binding_type()
        || installed.result_type() != requested.result_type()
    {
        return Err(denial(
            DenialKind::MutationBindingResultMeaningChanged,
            requested.identity(),
        ));
    }
    if installed.scope_entity() != requested.scope_entity()
        || installed.scope() != requested.scope()
    {
        return Err(denial(
            DenialKind::MutationBindingScopeMeaningChanged,
            requested.identity(),
        ));
    }
    if installed.candidates() != requested.candidates() {
        return Err(denial(
            DenialKind::MutationBindingCandidateMeaningChanged,
            requested.identity(),
        ));
    }
    if installed.handler() != requested.handler()
        || installed.denial_binding_type() != requested.denial_binding_type()
        || installed.description().denial_identity() != requested.description().denial_identity()
    {
        return Err(denial(
            DenialKind::MutationBindingHandlerMeaningChanged,
            requested.identity(),
        ));
    }
    if installed.output_contract_type() != requested.output_contract_type()
        || installed.output_roles() != requested.output_roles()
    {
        return Err(denial(
            DenialKind::MutationBindingOutputMeaningChanged,
            requested.identity(),
        ));
    }
    if installed.idempotency() != requested.idempotency() {
        return Err(denial(
            DenialKind::MutationBindingIdempotencyMeaningChanged,
            requested.identity(),
        ));
    }
    if installed != requested {
        return Err(denial(
            DenialKind::MutationBindingMeaningChanged,
            requested.identity(),
        ));
    }
    Ok(())
}

fn denial(kind: DenialKind, subject: &str) -> WorthQueryApplicationOperationInstallationDenial {
    WorthQueryApplicationOperationInstallationDenial::new(kind, subject)
}
use std::sync::Arc;
