//! Versioned provider/source meaning. Sources are explicit complete committed queries.
use super::{ApplicationManagedIndexBinding, ApplicationManagedIndexProbeMeaning};
use crate::application_query::ApplicationQueryBindingReference;
use crate::application_schema::{ApplicationEntityMarkerIdentity, ApplicationSchema};

/// Refusals while describing or resolving a managed index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationManagedIndexDefinitionDenial {
    UnsupportedVersion { observed: u16 },
    InvalidIdentifier,
    UnsupportedProbeProfile,
    UnsupportedSourcePosture,
    UnsupportedSourceShape,
    SourceDependencyCycle,
    MissingSource,
    ForeignSourceBinding,
    ForeignScope,
    ForeignProbe,
    DuplicateIndex,
    ConflictingRootSelection,
}

/// Versioned complete provider/source/probe association, without executable state.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryPortableManagedIndexParts {
    pub semantic_version: u16,
    pub name: String,
    pub provider: String,
    pub provider_version: u32,
    pub source: super::ApplicationManagedIndexSourceMeaning,
    pub entity: String,
    pub probe: ApplicationManagedIndexProbeMeaning,
}

/// A deterministic provider with canonical-source candidate ordering, complete
/// source dependencies, same-admitted-scope/no-source-parameter correspondence
/// and reconstruction from Public committed sources. Version1 fixes these policies.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ErasedApplicationManagedIndexDefinition {
    parts: WorthQueryPortableManagedIndexParts,
}
impl ErasedApplicationManagedIndexDefinition {
    pub const CURRENT_VERSION: u16 = 1;

    pub fn from_untrusted_parts(parts: WorthQueryPortableManagedIndexParts) -> Self {
        Self { parts }
    }
    pub fn parts(&self) -> &WorthQueryPortableManagedIndexParts {
        &self.parts
    }
    pub fn validate(&self) -> Result<(), ApplicationManagedIndexDefinitionDenial> {
        super::validation::validate_definition(&self.parts)
    }
}

/// Typed authoring of the provider's exact source and canonical probe contracts.
pub struct ApplicationManagedIndexDefinition<Schema, Index> {
    erased: ErasedApplicationManagedIndexDefinition,
    _marker: std::marker::PhantomData<fn() -> (Schema, Index)>,
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    ApplicationManagedIndexDefinition<S, I>
{
    pub fn new() -> Self {
        Self {
            erased: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
                WorthQueryPortableManagedIndexParts {
                    semantic_version: ErasedApplicationManagedIndexDefinition::CURRENT_VERSION,
                    name: I::NAME.to_owned(),
                    provider: I::PROVIDER.to_owned(),
                    provider_version: I::PROVIDER_VERSION,
                    source:
                        super::ApplicationManagedIndexSourceMeaning::same_scope_without_parameters(
                            ApplicationQueryBindingReference::of::<S, I::SourceBinding>(),
                        ),
                    entity: I::Entity::IDENTIFIER.to_owned(),
                    probe: ApplicationManagedIndexProbeMeaning::of::<I::ProbeBinding>(),
                },
            ),
            _marker: std::marker::PhantomData,
        }
    }
    pub(crate) fn into_erased(self) -> ErasedApplicationManagedIndexDefinition {
        self.erased
    }
}
impl<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>> Default
    for ApplicationManagedIndexDefinition<S, I>
{
    fn default() -> Self {
        Self::new()
    }
}
