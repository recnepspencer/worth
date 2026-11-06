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
    pub fn group_resources(
        &self,
    ) -> worth_query_declaration::facade::application_history::ApplicationHistoryGroupResources
    {
        self.compiled.descriptor.group_resources()
    }
    pub fn access_binding(
        &self,
    ) -> &WorthQueryInstalledApplicationQueryBinding<S, ApplicationHistoryAccessBinding<S, H>> {
        &self.access
    }

    pub fn navigation_resources(
        &self,
    ) -> worth_query_declaration::facade::application_history::ApplicationHistoryNavigationResources
    {
        self.compiled.descriptor.navigation_resources()
    }

    pub fn retention_resources(
        &self,
    ) -> worth_query_declaration::facade::application_history::ApplicationHistoryRetentionResources
    {
        self.compiled.descriptor.retention_resources()
    }

    pub fn contains_window<
        N: worth_query_declaration::facade::application_history::ApplicationHistoryWindowBinding<S, H>,
    >(
        &self,
    ) -> bool {
        self.compiled
            .descriptor
            .window()
            .is_some_and(|policy| policy.matches::<S, H, N>())
    }

    pub fn contains_bookmarks<
        N: worth_query_declaration::facade::application_history::ApplicationHistoryBookmarkBinding<
            S,
            H,
        >,
    >(
        &self,
    ) -> bool {
        self.compiled
            .descriptor
            .bookmarks()
            .is_some_and(|policy| policy.matches::<S, H, N>())
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
        #[expect(
            clippy::disallowed_methods,
            reason = "Rust type identity checks exact installed binding equality; it never orders history"
        )]
        let binding_type = TypeId::of::<H>();
        if compiled.descriptor.binding_type() != binding_type
            || compiled.descriptor.history_type() != &H::PORTABLE_TYPE_IDENTITY
            || compiled.descriptor.group_resources() != H::GROUP_RESOURCES
            || compiled.descriptor.navigation_resources() != H::NAVIGATION_RESOURCES
            || compiled.descriptor.retention_resources() != H::RETENTION_RESOURCES
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
