//! Declared ReBAC paths: typed traversals from the admitted principal to a
//! scope, with field predicates and principal-field constraints.
use worth_foundational::facade::AspectValue;

mod builder;
mod condition;
mod expansion;
mod portable_parts;
mod rule;

pub use builder::{
    ApplicationAuthorizationActor, ApplicationAuthorizationPathBuilder, AuthoredPath, ExactPath,
};
pub use condition::{
    ApplicationAuthorizationCondition, ApplicationAuthorizationFieldConditions,
    ApplicationOrderFaithfulBinding, NoActor, PrincipalActor,
};
pub use portable_parts::{
    WorthQueryPortableApplicationAuthorizationFieldConstraintParts,
    WorthQueryPortableApplicationAuthorizationPathParts,
    WorthQueryPortableApplicationAuthorizationPredicateParts,
    WorthQueryPortableApplicationAuthorizationTraversalParts,
};
pub use rule::{ApplicationAuthorizationPathSet, ApplicationAuthorizationRule};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationAuthorizationPathEffect {
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationAuthorizationTraversalDirection {
    Forward,
    Reverse,
}

/// The kernel comparison a declared predicate or constraint applies.
///
/// Every comparison is false when the observed value is absent or belongs to
/// another scalar family. Authored `ne` lowers to `StrictlyLess` or
/// `StrictlyGreater`; ordered comparisons are authored only on order-faithful
/// bindings (see [`ApplicationOrderFaithfulBinding`]).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationAuthorizationComparison {
    #[default]
    Equal,
    AtMost,
    AtLeast,
    StrictlyLess,
    StrictlyGreater,
}

impl ApplicationAuthorizationComparison {
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Equal => "equal",
            Self::AtMost => "at-most",
            Self::AtLeast => "at-least",
            Self::StrictlyLess => "strictly-less",
            Self::StrictlyGreater => "strictly-greater",
        }
    }

    pub(crate) const fn symbol(self) -> &'static str {
        match self {
            Self::Equal => "=",
            Self::AtMost => "<=",
            Self::AtLeast => ">=",
            Self::StrictlyLess => "<",
            Self::StrictlyGreater => ">",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationAuthorizationTraversal {
    relation: String,
    from: String,
    to: String,
    direction: ApplicationAuthorizationTraversalDirection,
}

impl ApplicationAuthorizationTraversal {
    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub const fn direction(&self) -> ApplicationAuthorizationTraversalDirection {
        self.direction
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationAuthorizationPredicate {
    traversal_ordinal: usize,
    entity: String,
    aspect: String,
    field: String,
    value: AspectValue,
    comparison: ApplicationAuthorizationComparison,
}

impl ApplicationAuthorizationPredicate {
    pub const fn traversal_ordinal(&self) -> usize {
        self.traversal_ordinal
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &str {
        &self.aspect
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn value(&self) -> &AspectValue {
        &self.value
    }

    pub const fn comparison(&self) -> ApplicationAuthorizationComparison {
        self.comparison
    }
}

/// Compares a field of the entity reached at `traversal_ordinal` with a field
/// of the admitted principal (path position 0).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationAuthorizationFieldConstraint {
    traversal_ordinal: usize,
    entity: String,
    aspect: String,
    field: String,
    comparison: ApplicationAuthorizationComparison,
    principal_aspect: String,
    principal_field: String,
}

impl ApplicationAuthorizationFieldConstraint {
    pub const fn traversal_ordinal(&self) -> usize {
        self.traversal_ordinal
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &str {
        &self.aspect
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub const fn comparison(&self) -> ApplicationAuthorizationComparison {
        self.comparison
    }

    pub fn principal_aspect(&self) -> &str {
        &self.principal_aspect
    }

    pub fn principal_field(&self) -> &str {
        &self.principal_field
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationAuthorizationPath {
    effect: ApplicationAuthorizationPathEffect,
    principal_entity: String,
    scope_entity: String,
    traversals: Vec<ApplicationAuthorizationTraversal>,
    predicates: Vec<ApplicationAuthorizationPredicate>,
    field_constraints: Vec<ApplicationAuthorizationFieldConstraint>,
}

impl ApplicationAuthorizationPath {
    pub const fn effect(&self) -> ApplicationAuthorizationPathEffect {
        self.effect
    }

    pub fn principal_entity(&self) -> &str {
        &self.principal_entity
    }

    pub fn scope_entity(&self) -> &str {
        &self.scope_entity
    }

    pub fn traversals(&self) -> &[ApplicationAuthorizationTraversal] {
        &self.traversals
    }

    pub fn predicates(&self) -> &[ApplicationAuthorizationPredicate] {
        &self.predicates
    }

    /// Principal-field constraints; empty for every path authored before them.
    pub fn field_constraints(&self) -> &[ApplicationAuthorizationFieldConstraint] {
        &self.field_constraints
    }

    /// Whether the path uses anything beyond equality predicates. Legacy
    /// encodings and identities apply exactly when this is false.
    pub fn uses_extended_conditions(&self) -> bool {
        !self.field_constraints.is_empty()
            || self
                .predicates
                .iter()
                .any(|predicate| predicate.comparison != ApplicationAuthorizationComparison::Equal)
    }
}
