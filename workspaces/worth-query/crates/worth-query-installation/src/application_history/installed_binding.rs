use super::{
    catalog::CompiledApplicationHistory, WorthQueryDocumentHistoryInstallationDenial as Denial,
    WorthQueryDocumentHistoryInstallationDenialKind as Kind,
};
use crate::{
    application_query::WorthQueryInstalledApplicationQueryBinding,
    application_schema::WorthQueryInstalledApplicationSchema,
};
use std::{any::TypeId, marker::PhantomData, sync::Arc};
use worth_query_declaration::facade::{
    application_history::{
        ApplicationHistoryAccessBinding, ApplicationHistoryBinding,
        ApplicationHistoryNavigationBinding, ErasedApplicationHistoryDefinition,
    },
    application_operation::ApplicationMutationBinding,
    application_query::ApplicationQueryBinding,
    application_schema::{ApplicationSchema, ApplicationSchemaBindingIdentity},
};

/// Installed typed correspondence; request admission and live head custody are separate.
pub struct WorthQueryInstalledDocumentHistory<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
{
    compiled: Arc<CompiledApplicationHistory>,
    access: WorthQueryInstalledApplicationQueryBinding<S, ApplicationHistoryAccessBinding<S, H>>,
    marker: PhantomData<fn() -> H>,
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryInstalledDocumentHistory<S, H>
{
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.compiled.schema
    }
    pub fn definition(&self) -> &ErasedApplicationHistoryDefinition {
        &self.compiled.definition
    }
    pub fn access_binding(
        &self,
    ) -> &WorthQueryInstalledApplicationQueryBinding<S, ApplicationHistoryAccessBinding<S, H>> {
        &self.access
    }

    /// Compare the original typed recipe and every portable navigation field.
    /// This establishes installation correspondence, never request permission.
    pub fn contains_navigation<N: ApplicationHistoryNavigationBinding<S, H>>(&self) -> bool {
        self.compiled
            .descriptor
            .navigation()
            .is_some_and(|navigation| navigation.matches::<S, H, N>())
    }

    pub fn operation_descriptors(&self) -> &[worth_query_declaration::facade::application_operation::ApplicationMutationBindingDescriptor]{
        self.compiled.descriptor.operations()
    }

    /// Static external-effect declarations on the actual participating operations.
    /// This says nothing about current permission or retained output readiness.
    pub fn has_external_operation(&self) -> bool {
        self.compiled.has_external_operation
    }

    /// Compare the full installed operation meaning, rather than its authored name.
    pub fn contains_operation<M: ApplicationMutationBinding<S>>(&self) -> bool {
        self.operation_label::<M>().is_some()
    }

    pub fn operation_label<M: ApplicationMutationBinding<S>>(&self) -> Option<&str> {
        let descriptor = M::descriptor();
        self.compiled
            .descriptor
            .operations()
            .iter()
            .zip(&self.compiled.definition.parts().operations)
            .find(|(installed, _)| **installed == descriptor)
            .map(|(_, operation)| operation.label.as_str())
    }
}

impl<S: ApplicationSchema> WorthQueryInstalledApplicationSchema<S> {
    pub fn installed_history<H: ApplicationHistoryBinding<S>>(
        &self,
    ) -> Result<WorthQueryInstalledDocumentHistory<S, H>, Denial> {
        let compiled = self
            .document_history_catalog
            .get(H::IDENTITY)
            .ok_or_else(|| Denial::new(Kind::NotInstalled, H::IDENTITY))?;
        if compiled.descriptor.binding_type() != TypeId::of::<H>()
            || compiled.descriptor.history_type() != &H::PORTABLE_TYPE_IDENTITY
        {
            return Err(Denial::new(Kind::BindingMeaningChanged, H::IDENTITY));
        }
        let access = self
            .installed_query_binding::<ApplicationHistoryAccessBinding<S, H>>()
            .map_err(|cause| Denial::access(H::IDENTITY, cause))?;
        if !compiled
            .definition
            .parts()
            .access
            .matches_descriptor(&ApplicationHistoryAccessBinding::<S, H>::descriptor())
        {
            return Err(Denial::new(Kind::AccessBindingMismatch, H::IDENTITY));
        }
        Ok(WorthQueryInstalledDocumentHistory {
            compiled,
            access,
            marker: PhantomData,
        })
    }
}
