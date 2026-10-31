use std::{collections::BTreeSet, marker::PhantomData};

use crate::application_operation::{ApplicationMutationBinding, ApplicationMutationScopeBinding};
use crate::application_query::ApplicationQueryBindingReference;
use crate::application_schema::ApplicationSchema;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    ApplicationHistoryAccessBinding, ApplicationHistoryBinding,
    ApplicationHistoryBindingDescriptor, ApplicationHistoryModel,
};

/// One ordinary operation participating in the model's user history.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationHistoryOperation {
    pub binding: String,
    pub label: String,
}

/// Authority-free carriage. Installation must resolve the original typed recipes.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryPortableApplicationHistoryParts {
    pub identity: String,
    pub history_type: WorthQueryPortableTypeIdentity,
    pub access: ApplicationQueryBindingReference,
    pub operations: Vec<ApplicationHistoryOperation>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ErasedApplicationHistoryDefinition {
    parts: WorthQueryPortableApplicationHistoryParts,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationHistoryDefinitionDenial {
    InvalidIdentity,
    InvalidAccess,
    MissingOperations,
    InvalidOperation,
    DuplicateOperation,
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
        if !name_is_valid(&parts.identity) || !parts.history_type.is_valid() {
            return Err(Denial::InvalidIdentity);
        }
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
    pub fn for_binding() -> Self {
        Self {
            erased: ErasedApplicationHistoryDefinition::from_untrusted_parts(
                WorthQueryPortableApplicationHistoryParts {
                    identity: H::IDENTITY.to_owned(),
                    history_type: H::PORTABLE_TYPE_IDENTITY,
                    access: ApplicationQueryBindingReference::of::<
                        S,
                        ApplicationHistoryAccessBinding<S, H>,
                    >(),
                    operations: Vec::new(),
                },
            ),
            descriptor: ApplicationHistoryBindingDescriptor::of::<S, H>(),
            marker: PhantomData,
        }
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
