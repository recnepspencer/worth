//! An ordinary query's committed-index selector; it supplies no candidate-state posture.
use super::{ApplicationManagedIndexBinding, ApplicationManagedIndexRef};
use crate::application_query::ApplicationQueryParameterRef;
use crate::application_schema::ApplicationSchema;

/// Descriptive association between a query probe parameter and an index.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryPortableQueryManagedIndexParts {
    pub semantic_version: u16,
    pub index: String,
    pub probe_parameter: String,
}

/// Installed queries bind this selector to their complete source/result and parameter profiles.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationQueryManagedIndexMeaning {
    parts: WorthQueryPortableQueryManagedIndexParts,
    refinement: Option<super::ApplicationManagedIndexRefinementMeaning>,
}
impl ApplicationQueryManagedIndexMeaning {
    pub fn from_untrusted_parts(parts: WorthQueryPortableQueryManagedIndexParts) -> Self {
        Self {
            parts,
            refinement: None,
        }
    }
    pub fn from_refinement_parts(
        parts: WorthQueryPortableQueryManagedIndexParts,
        refinement: super::ApplicationManagedIndexRefinementMeaning,
    ) -> Self {
        Self {
            parts,
            refinement: Some(refinement),
        }
    }
    pub fn refinement(&self) -> Option<&super::ApplicationManagedIndexRefinementMeaning> {
        self.refinement.as_ref()
    }
    pub fn posture(&self) -> super::ApplicationManagedIndexMatchPosture {
        if self.parts.semantic_version == 2 {
            super::ApplicationManagedIndexMatchPosture::Refined
        } else {
            super::ApplicationManagedIndexMatchPosture::Candidates
        }
    }
    pub fn parts(&self) -> &WorthQueryPortableQueryManagedIndexParts {
        &self.parts
    }
    pub fn validate(&self) -> Result<(), super::ApplicationManagedIndexDefinitionDenial> {
        use super::ApplicationManagedIndexDefinitionDenial as Denial;
        if !matches!(self.parts.semantic_version, 1 | 2) {
            return Err(Denial::UnsupportedVersion {
                observed: self.parts.semantic_version,
            });
        }
        match (self.parts.semantic_version, self.refinement.as_ref()) {
            (1, None) => {}
            (2, Some(refinement)) => refinement.validate()?,
            _ => return Err(Denial::UnsupportedRefinement),
        }
        if self.parts.index.is_empty() || self.parts.probe_parameter.is_empty() {
            return Err(Denial::InvalidIdentifier);
        }
        Ok(())
    }
    pub(in crate::application_query) fn typed<S, I, Q, P>(
        index: ApplicationManagedIndexRef<S, I>,
        parameter: ApplicationQueryParameterRef<Q, P, I::ProbeBinding>,
        posture: super::ApplicationManagedIndexMatchPosture,
    ) -> Self
    where
        S: ApplicationSchema,
        I: ApplicationManagedIndexBinding<S>,
    {
        let mut value = Self::from_untrusted_parts(WorthQueryPortableQueryManagedIndexParts {
            semantic_version: if posture == super::ApplicationManagedIndexMatchPosture::Refined {
                2
            } else {
                1
            },
            index: index.name().to_owned(),
            probe_parameter: parameter.name().to_owned(),
        });
        if posture == super::ApplicationManagedIndexMatchPosture::Refined {
            value.refinement = I::REFINEMENT.map(|refinement| refinement.meaning());
        }
        value
    }
}
