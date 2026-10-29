//! Compiler-local typed correspondence, independent of transported meaning.
use super::{
    ApplicationManagedIndexBinding, ApplicationManagedIndexDefinition,
    ErasedApplicationManagedIndexDefinition,
};
use crate::application_query::{ApplicationQueryBinding, ApplicationQueryBindingDescriptor};
use crate::application_schema::ApplicationSchema;
use std::any::TypeId;

/// The actual declaring schema's provider and probe associations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationManagedIndexBindingDescriptor {
    meaning: ErasedApplicationManagedIndexDefinition,
    source: ApplicationQueryBindingDescriptor,
    index_type: TypeId,
    probe_binding_type: TypeId,
}

impl ApplicationManagedIndexBindingDescriptor {
    pub(crate) fn of<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>() -> Self {
        Self {
            meaning: ApplicationManagedIndexDefinition::<S, I>::new().into_erased(),
            source: I::SourceBinding::descriptor(),
            index_type: TypeId::of::<I>(),
            probe_binding_type: TypeId::of::<I::ProbeBinding>(),
        }
    }

    pub(super) fn index_type(&self) -> TypeId {
        self.index_type
    }
    pub(super) fn probe_binding_type(&self) -> TypeId {
        self.probe_binding_type
    }

    pub fn meaning(&self) -> &ErasedApplicationManagedIndexDefinition {
        &self.meaning
    }

    pub fn source(&self) -> &ApplicationQueryBindingDescriptor {
        &self.source
    }

    /// Type equality supplements, and never replaces, installed canonical meaning.
    pub fn matches<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>(&self) -> bool {
        self.index_type == TypeId::of::<I>()
            && self.probe_binding_type == TypeId::of::<I::ProbeBinding>()
            && self.meaning == ApplicationManagedIndexDefinition::<S, I>::new().into_erased()
            && self.source == I::SourceBinding::descriptor()
    }
}
