//! Portable derived relation meaning; no stored-edge deletion or write authority.
use crate::application_schema::{ApplicationGraphMemberMeaning, ApplicationRelationGraphLaws};

use super::denial::ApplicationDerivedRelationDefinitionDenial as Denial;
use super::expression::ApplicationDerivedRelationExpression;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
/// Untrusted owned recipe, endpoint and law meaning for fresh readmission.
pub struct WorthQueryPortableDerivedRelationParts {
    pub relation: String,
    pub from: ApplicationGraphMemberMeaning,
    pub to: ApplicationGraphMemberMeaning,
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
        for identifier in [
            self.parts.relation.as_str(),
            self.parts.from.name(),
            self.parts.to.name(),
        ] {
            super::super::identifier_validation::validate_simple_identifier(identifier)
                .map_err(|_| Denial::InvalidIdentifier)?;
        }
        for node in &self.parts.expression.nodes {
            if let super::ApplicationDerivedRelationNode::CarrierLift {
                domain,
                carrier,
                case,
            } = node
            {
                for identifier in [domain, carrier, case] {
                    super::super::identifier_validation::validate_simple_identifier(identifier)
                        .map_err(|_| Denial::InvalidIdentifier)?;
                }
            }
            if let super::ApplicationDerivedRelationNode::Source(source) = node {
                if source.kind == super::ApplicationDerivedRelationSourceKind::Stored
                    && (source.from.entity().is_none() || source.to.entity().is_none())
                {
                    return Err(Denial::MissingSource);
                }
                for identifier in [
                    source.relation.as_str(),
                    source.from.name(),
                    source.to.name(),
                ] {
                    super::super::identifier_validation::validate_simple_identifier(identifier)
                        .map_err(|_| Denial::InvalidIdentifier)?;
                }
            }
        }
        let endpoints = self.parts.expression.validate()?;
        if endpoints != (self.parts.from.borrowed(), self.parts.to.borrowed()) {
            return Err(Denial::WrongDefinitionEndpoints);
        }
        if !self
            .parts
            .laws
            .borrowed()
            .is_valid(self.parts.from.name(), self.parts.to.name())
            || (self.parts.laws.orbit && self.parts.from != self.parts.to)
        {
            return Err(Denial::InvalidGraphLaws);
        }
        Ok(())
    }

    /// Exact legacy wire eligibility; extended meaning has one distinct grammar.
    pub fn is_legacy_shape(&self) -> bool {
        self.parts.from.entity().is_some()
            && self.parts.to.entity().is_some()
            && self.parts.expression.is_legacy_shape()
    }
}
