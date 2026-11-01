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
    UnsupportedRefinement,
    UnsupportedMaintenance,
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
    refinement: Option<super::ApplicationManagedIndexRefinementMeaning>,
    maintenance: Option<super::ApplicationManagedIndexMaintenance>,
}
impl ErasedApplicationManagedIndexDefinition {
    pub const CURRENT_VERSION: u16 = 5;
    pub const REFINEMENT_VERSION: u16 = 4;
    pub const CANDIDATE_MEMBERSHIP_VERSION: u16 = 3;

    pub fn from_untrusted_parts(parts: WorthQueryPortableManagedIndexParts) -> Self {
        Self {
            parts,
            refinement: None,
            maintenance: None,
        }
    }
    pub fn from_refinement_parts(
        parts: WorthQueryPortableManagedIndexParts,
        refinement: super::ApplicationManagedIndexRefinementMeaning,
    ) -> Self {
        Self {
            parts,
            refinement: Some(refinement),
            maintenance: None,
        }
    }
    /// V5 requires explicit maintenance metadata; legacy constructors preserve absence.
    pub fn from_maintenance_parts(
        parts: WorthQueryPortableManagedIndexParts,
        maintenance: super::ApplicationManagedIndexMaintenance,
        refinement: Option<super::ApplicationManagedIndexRefinementMeaning>,
    ) -> Self {
        Self {
            parts,
            refinement,
            maintenance: Some(maintenance),
        }
    }
    pub const fn maintenance(&self) -> super::ApplicationManagedIndexMaintenance {
        match self.maintenance {
            Some(value) => value,
            None => super::ApplicationManagedIndexMaintenance::WhenRequired,
        }
    }
    pub fn refinement(&self) -> Option<&super::ApplicationManagedIndexRefinementMeaning> {
        self.refinement.as_ref()
    }
    pub fn parts(&self) -> &WorthQueryPortableManagedIndexParts {
        &self.parts
    }
    pub fn validate(&self) -> Result<(), ApplicationManagedIndexDefinitionDenial> {
        super::validation::validate_definition(&self.parts)?;
        if !matches!(
            (self.parts.semantic_version, self.maintenance),
            (1..=4, None) | (5, Some(_))
        ) {
            return Err(ApplicationManagedIndexDefinitionDenial::UnsupportedMaintenance);
        }
        match (self.parts.semantic_version, self.refinement.as_ref()) {
            (4 | 5, Some(refinement))
                if self.parts.membership
                    != super::ApplicationManagedIndexMembership::Candidates =>
            {
                refinement.validate()
            }
            (1..=3 | 5, None) => Ok(()),
            _ => Err(ApplicationManagedIndexDefinitionDenial::UnsupportedRefinement),
        }
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
        let mut value = Self {
            erased: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(
                WorthQueryPortableManagedIndexParts {
                    semantic_version: if I::MAINTENANCE.native_cold().is_some() {
                        ErasedApplicationManagedIndexDefinition::CURRENT_VERSION
                    } else if I::REFINEMENT.is_some() {
                        ErasedApplicationManagedIndexDefinition::REFINEMENT_VERSION
                    } else {
                        ErasedApplicationManagedIndexDefinition::CANDIDATE_MEMBERSHIP_VERSION
                    },
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
        };
        value.erased.refinement = I::REFINEMENT.map(|refinement| refinement.meaning());
        if value.erased.parts.semantic_version
            == ErasedApplicationManagedIndexDefinition::CURRENT_VERSION
        {
            value.erased.maintenance = Some(I::MAINTENANCE);
        }
        value
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
