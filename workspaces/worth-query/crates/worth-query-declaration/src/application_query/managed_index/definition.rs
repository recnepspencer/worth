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
    UnsupportedSourceStructure,
    UnsupportedMembership,
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
    pub structure: super::ApplicationManagedIndexSourceStructure,
    pub membership: super::ApplicationManagedIndexMembership,
    pub source: super::ApplicationManagedIndexSourceMeaning,
    pub entity: String,
    pub probe: ApplicationManagedIndexProbeMeaning,
}

/// A deterministic provider with canonical-source candidate ordering, complete
/// source dependencies, same-admitted-scope/no-source-parameter correspondence
/// and reconstruction from Public committed sources. V1 is Opaque; v2 binds structure;
/// v3 additionally binds the installed candidate-membership execution contract.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ErasedApplicationManagedIndexDefinition {
    parts: WorthQueryPortableManagedIndexParts,
}
impl ErasedApplicationManagedIndexDefinition {
    pub const CURRENT_VERSION: u16 = 3;

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
                    structure: I::STRUCTURE,
                    membership: I::MEMBERSHIP,
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
