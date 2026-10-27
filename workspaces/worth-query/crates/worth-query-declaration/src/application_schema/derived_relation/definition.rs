//! Portable derived relation meaning; no stored-edge deletion or write authority.
use crate::application_schema::ApplicationRelationGraphLaws;

use super::denial::ApplicationDerivedRelationDefinitionDenial as Denial;
use super::expression::ApplicationDerivedRelationExpression;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Untrusted owned recipe, endpoint and law meaning for fresh readmission.
pub struct WorthQueryPortableDerivedRelationParts {
    pub relation: String,
    pub from: String,
    pub to: String,
    pub expression: ApplicationDerivedRelationExpression,
    pub laws: ApplicationRelationGraphLaws,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Owned descriptive definition; validation does not supply a runtime source.
pub struct ErasedApplicationDerivedRelationDefinition {
    parts: WorthQueryPortableDerivedRelationParts,
}

impl ErasedApplicationDerivedRelationDefinition {
    /// Retains transport meaning without granting installed authority.
    pub fn from_untrusted_parts(parts: WorthQueryPortableDerivedRelationParts) -> Self {
        Self { parts }
    }

    /// Borrows the complete owned definition.
    pub fn parts(&self) -> &WorthQueryPortableDerivedRelationParts {
        &self.parts
    }

    /// Validates local syntax, endpoint continuity and graph-law shape.
    pub fn validate(&self) -> Result<(), Denial> {
        for identifier in [&self.parts.relation, &self.parts.from, &self.parts.to] {
            super::super::identifier_validation::validate_simple_identifier(identifier)
                .map_err(|_| Denial::InvalidIdentifier)?;
        }
        for node in &self.parts.expression.nodes {
            if let super::ApplicationDerivedRelationNode::Source(source) = node {
                for identifier in [&source.relation, &source.from, &source.to] {
                    super::super::identifier_validation::validate_simple_identifier(identifier)
                        .map_err(|_| Denial::InvalidIdentifier)?;
                }
            }
        }
        let endpoints = self.parts.expression.validate()?;
        if endpoints != (self.parts.from.as_str(), self.parts.to.as_str()) {
            return Err(Denial::WrongDefinitionEndpoints);
        }
        if !self
            .parts
            .laws
            .borrowed()
            .is_valid(&self.parts.from, &self.parts.to)
        {
            return Err(Denial::InvalidGraphLaws);
        }
        Ok(())
    }
}
