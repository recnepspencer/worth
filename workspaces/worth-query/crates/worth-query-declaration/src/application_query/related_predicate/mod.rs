//! Bounded related-record absence (`NOT EXISTS`) and existence (`EXISTS`)
//! predicates over a declared relation, with optional target-field equality
//! predicates and an optional Foundation `where_filter` over the target entity.
use crate::application_query::{
    ApplicationQueryDefinitionDenial, ApplicationQueryParameterDefinition,
    ApplicationQueryPredicate, ApplicationQueryPredicateExpected,
    ApplicationQueryWhereFilterProgram, RelationDirection,
};
use crate::application_schema::ApplicationFieldPresence;
use crate::archive_decode_authority::ArchiveDecodeAuthority;
use worth_foundational::facade::{AspectValue, ScalarAspectType};

mod authoring;
pub(in crate::application_query) mod validation;

#[cfg(test)]
mod tests;

pub use authoring::ApplicationQueryRelatedPredicate;
pub use validation::validate as validate_related_predicates;

/// Whether a root candidate passes when zero (`Absent` / `NOT EXISTS`) or at
/// least one (`Exists` / `EXISTS`) related target record matches.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryRelatedPolarity {
    Absent,
    Exists,
}

impl ApplicationQueryRelatedPolarity {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Exists => "exists",
        }
    }
}

/// One structured equality predicate on the related target entity, comparing a
/// target field against either a declared runtime parameter or an authored
/// scalar constant.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryRelatedTargetPredicate {
    entity: String,
    aspect: String,
    field: String,
    expected: ApplicationQueryPredicateExpected,
    scalar_family: ScalarAspectType,
    presence: ApplicationFieldPresence,
}

impl ApplicationQueryRelatedTargetPredicate {
    pub fn from_untrusted_parameter(
        _: ArchiveDecodeAuthority,
        entity: String,
        aspect: String,
        field: String,
        parameter: String,
        scalar_family: ScalarAspectType,
        presence: ApplicationFieldPresence,
    ) -> Self {
        Self {
            entity,
            aspect,
            field,
            expected: ApplicationQueryPredicateExpected::Parameter(parameter),
            scalar_family,
            presence,
        }
    }

    pub fn from_untrusted_value(
        _: ArchiveDecodeAuthority,
        entity: String,
        aspect: String,
        field: String,
        value: AspectValue,
        scalar_family: ScalarAspectType,
        presence: ApplicationFieldPresence,
    ) -> Self {
        Self {
            entity,
            aspect,
            field,
            expected: ApplicationQueryPredicateExpected::Value(value),
            scalar_family,
            presence,
        }
    }

    pub fn field(&self) -> (&str, &str, &str) {
        (&self.entity, &self.aspect, &self.field)
    }

    pub fn parameter(&self) -> Option<&str> {
        match &self.expected {
            ApplicationQueryPredicateExpected::Parameter(parameter) => Some(parameter),
            ApplicationQueryPredicateExpected::Value(_) => None,
        }
    }

    pub fn literal_value(&self) -> Option<&AspectValue> {
        match &self.expected {
            ApplicationQueryPredicateExpected::Parameter(_) => None,
            ApplicationQueryPredicateExpected::Value(value) => Some(value),
        }
    }

    pub const fn expected(&self) -> &ApplicationQueryPredicateExpected {
        &self.expected
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub const fn presence(&self) -> ApplicationFieldPresence {
        self.presence
    }

    /// Declaration-owned local check for a target literal value.
    pub fn literal_value_is_valid(value: &AspectValue, scalar_family: ScalarAspectType) -> bool {
        ApplicationQueryPredicate::literal_value_is_valid(value, scalar_family)
    }
}

/// Portable, canonical meaning of one related-record absence/existence
/// predicate attached to an application query definition.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryRelatedPredicateMeaning {
    relation: String,
    from: String,
    to: String,
    direction: RelationDirection,
    polarity: ApplicationQueryRelatedPolarity,
    target_predicates: Vec<ApplicationQueryRelatedTargetPredicate>,
    target_where_filter: Option<ApplicationQueryWhereFilterProgram>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryRelatedPredicateParts {
    pub relation: String,
    pub from: String,
    pub to: String,
    pub direction: RelationDirection,
    pub polarity: ApplicationQueryRelatedPolarity,
    pub target_predicates: Vec<ApplicationQueryRelatedTargetPredicate>,
    pub target_where_filter: Option<ApplicationQueryWhereFilterProgram>,
}

impl ApplicationQueryRelatedPredicateMeaning {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationQueryRelatedPredicateParts,
    ) -> Self {
        Self {
            relation: parts.relation,
            from: parts.from,
            to: parts.to,
            direction: parts.direction,
            polarity: parts.polarity,
            target_predicates: parts.target_predicates,
            target_where_filter: parts.target_where_filter,
        }
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub const fn direction(&self) -> RelationDirection {
        self.direction
    }

    pub const fn polarity(&self) -> ApplicationQueryRelatedPolarity {
        self.polarity
    }

    /// The entity on the root (anchor) side of the traversed relation.
    pub fn root_entity(&self) -> &str {
        match self.direction {
            RelationDirection::Forward => &self.from,
            RelationDirection::Reverse => &self.to,
        }
    }

    /// The entity on the related (neighbor) side of the traversed relation.
    pub fn target_entity(&self) -> &str {
        match self.direction {
            RelationDirection::Forward => &self.to,
            RelationDirection::Reverse => &self.from,
        }
    }

    pub fn target_predicates(&self) -> &[ApplicationQueryRelatedTargetPredicate] {
        &self.target_predicates
    }

    pub fn target_where_filter(&self) -> Option<&ApplicationQueryWhereFilterProgram> {
        self.target_where_filter.as_ref()
    }

    /// Declaration-owned local validation of one untrusted related predicate
    /// against a query's root entity and declared parameters (reused by archive
    /// decoding before constructing an erased definition).
    pub fn validate_locally(
        &self,
        root_entity: &str,
        parameters: &[ApplicationQueryParameterDefinition],
    ) -> Result<(), ApplicationQueryDefinitionDenial> {
        validation::validate_one(self, root_entity, parameters)
    }
}
