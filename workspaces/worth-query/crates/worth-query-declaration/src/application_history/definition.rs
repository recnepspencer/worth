use std::{collections::BTreeSet, marker::PhantomData};

use crate::application_operation::{ApplicationMutationBinding, ApplicationMutationScopeBinding};
use crate::application_query::{
    ApplicationQueryBindingReference, ApplicationQueryDisclosureContract,
};
use crate::application_schema::ApplicationSchema;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    ApplicationHistoryAccessBinding, ApplicationHistoryBinding,
    ApplicationHistoryBindingDescriptor, ApplicationHistoryModel,
    ApplicationHistoryNavigationBinding, ApplicationHistoryNavigationBindingDescriptor,
    ApplicationHistoryNavigationDeclaration,
};

/// One ordinary operation participating in the model's user history.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationHistoryOperation {
    pub binding: String,
    pub label: String,
}

/// The literal model boundary for explicit physical-context refresh.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationHistoryModelDomain {
    /// Every physical component remains part of the expected source.
    WholeComposite,
    /// Exactly the full-GEN Native scope entity record. Relations, other
    /// records and Signal state are not inferred to belong to this domain.
    ScopeEntityRecord,
}

/// Authority-free carriage. Installation must resolve the original typed recipes.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryPortableApplicationHistoryParts {
    pub identity: String,
    pub history_type: WorthQueryPortableTypeIdentity,
    pub access: ApplicationQueryBindingReference,
    pub metadata_disclosure: ApplicationQueryDisclosureContract,
    pub model_domain: ApplicationHistoryModelDomain,
    pub operations: Vec<ApplicationHistoryOperation>,
    pub navigation: Option<ApplicationHistoryNavigationDeclaration>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ErasedApplicationHistoryDefinition {
    parts: WorthQueryPortableApplicationHistoryParts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationHistoryDefinitionDenial {
    InvalidIdentity,
    InvalidAccess,
    InvalidMetadataDisclosure,
    MissingOperations,
    InvalidOperation,
    DuplicateOperation,
    InvalidNavigation,
    UnsupportedNavigationDomain,
}

impl ErasedApplicationHistoryDefinition {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationHistoryParts) -> Self {
        Self { parts }
    }

    pub fn parts(&self) -> &WorthQueryPortableApplicationHistoryParts {
        &self.parts
    }

    pub fn validate(&self) -> Result<(), ApplicationHistoryDefinitionDenial> {
        use ApplicationHistoryDefinitionDenial as Denial;
        let parts = &self.parts;
        if let Some(navigation) = &parts.navigation {
            if !navigation.is_valid() {
                return Err(Denial::InvalidNavigation);
            }
            if parts.model_domain != ApplicationHistoryModelDomain::ScopeEntityRecord {
                return Err(Denial::UnsupportedNavigationDomain);
            }
        }
        if !name_is_valid(&parts.identity) || !parts.history_type.is_valid() {
            return Err(Denial::InvalidIdentity);
        }
        parts
            .metadata_disclosure
            .validate_history_metadata(&parts.history_type)
            .map_err(|_| Denial::InvalidMetadataDisclosure)?;
        let access = parts.access.parts();
        if parts
            .access
            .canonical_fields()
            .iter()
            .any(|(_, value)| !name_is_valid(value))
            || [
                &access.binding,
                &access.input,
                &access.query,
                &access.parameters,
                &access.result,
                &access.scope,
                &access.scope_value,
            ]
            .into_iter()
            .any(|identity| !identity.is_valid())
        {
            return Err(Denial::InvalidAccess);
        }
        if parts.operations.is_empty() {
            return Err(Denial::MissingOperations);
        }
        let mut seen = BTreeSet::new();
        for operation in &parts.operations {
            if !name_is_valid(&operation.binding) || operation.label.trim().is_empty() {
                return Err(Denial::InvalidOperation);
            }
            if !seen.insert(&operation.binding) {
                return Err(Denial::DuplicateOperation);
            }
        }
        Ok(())
    }
}

fn name_is_valid(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
}

/// Declare this binding inside its existing application contribution.
pub struct ApplicationHistoryDefinition<Schema, History> {
    erased: ErasedApplicationHistoryDefinition,
    descriptor: ApplicationHistoryBindingDescriptor,
    marker: PhantomData<fn() -> (Schema, History)>,
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> ApplicationHistoryDefinition<S, H> {
    pub fn for_binding(metadata_disclosure: ApplicationQueryDisclosureContract) -> Self {
        Self {
            erased: ErasedApplicationHistoryDefinition::from_untrusted_parts(
                WorthQueryPortableApplicationHistoryParts {
                    identity: H::IDENTITY.to_owned(),
                    history_type: H::PORTABLE_TYPE_IDENTITY,
                    access: ApplicationQueryBindingReference::of::<
                        S,
                        ApplicationHistoryAccessBinding<S, H>,
                    >(),
                    metadata_disclosure,
                    model_domain: ApplicationHistoryModelDomain::WholeComposite,
                    operations: Vec::new(),
                    navigation: None,
                },
            ),
            descriptor: ApplicationHistoryBindingDescriptor::of::<S, H>(),
            marker: PhantomData,
        }
    }

    /// Select the model boundary. `ScopeEntityRecord` permits bounded context
    /// refresh only when Native proves no intervening authorship of that record.
    /// Related records, relations and Signal state are not included implicitly.
    /// ```
    /// use worth_query_declaration::facade::{application_history::*, application_schema::ApplicationSchema};
    /// fn record_domain<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>(
    ///     definition: ApplicationHistoryDefinition<S, H>,
    /// ) -> ApplicationHistoryDefinition<S, H> {
    ///     definition.model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
    /// }
    /// ```
    pub fn model_domain(mut self, domain: ApplicationHistoryModelDomain) -> Self {
        self.erased.parts.model_domain = domain;
        self
    }

    /// Opt the declared scope entity's authoritative payload into snapshot
    /// navigation. Requires `ScopeEntityRecord`; this does not include live
    /// grants, other records, relations or Signal. Each request must separately
    /// satisfy the installed READ, metadata and navigation capability policies.
    pub fn navigation<N: ApplicationHistoryNavigationBinding<S, H>>(mut self) -> Self {
        let navigation = ApplicationHistoryNavigationBindingDescriptor::of::<S, H, N>();
        self.erased.parts.navigation = Some(navigation.declaration().clone());
        self.descriptor.set_navigation(navigation);
        self
    }

    /// Only operations scoped to the READ binding's actual model type can join.
    /// ```compile_fail,E0271
    /// use worth_query_declaration::facade::{application_history::*, application_operation::ApplicationMutationBinding, application_schema::ApplicationSchema};
    /// fn unchecked<S: ApplicationSchema, H: ApplicationHistoryBinding<S>, M: ApplicationMutationBinding<S>>(definition: ApplicationHistoryDefinition<S, H>) {
    ///     let _ = definition.operation::<M>("Edit");
    /// }
    /// ```
    /// The exact declared model scope supplies the lawful counterpart.
    /// ```
    /// use worth_query_declaration::facade::{application_history::*, application_operation::{ApplicationMutationBinding, ApplicationMutationScopeBinding}, application_schema::ApplicationSchema};
    /// fn joined<S: ApplicationSchema, H: ApplicationHistoryBinding<S>, M: ApplicationMutationBinding<S>>(definition: ApplicationHistoryDefinition<S, H>)
    /// where M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>> {
    ///     let _ = definition.operation::<M>("Edit");
    /// }
    /// ```
    pub fn operation<M>(mut self, label: impl Into<String>) -> Self
    where
        M: ApplicationMutationBinding<S>,
        M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    {
        let operation = M::descriptor();
        self.erased
            .parts
            .operations
            .push(ApplicationHistoryOperation {
                binding: operation.identity().to_owned(),
                label: label.into(),
            });
        self.descriptor.push_operation(operation);
        self
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        ErasedApplicationHistoryDefinition,
        ApplicationHistoryBindingDescriptor,
    ) {
        (self.erased, self.descriptor)
    }
}
