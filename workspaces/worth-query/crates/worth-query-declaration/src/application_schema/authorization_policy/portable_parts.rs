use crate::archive_decode_authority::ArchiveDecodeAuthority;
use worth_foundational::facade::AspectValue;

use super::{
    ApplicationAuthorizationComparison, ApplicationAuthorizationFieldConstraint,
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect,
    ApplicationAuthorizationPredicate, ApplicationAuthorizationTraversal,
    ApplicationAuthorizationTraversalDirection,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationAuthorizationTraversalParts {
    pub relation: String,
    pub from: String,
    pub to: String,
    pub direction: ApplicationAuthorizationTraversalDirection,
}

impl ApplicationAuthorizationTraversal {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationAuthorizationTraversalParts,
    ) -> Self {
        Self {
            relation: parts.relation,
            from: parts.from,
            to: parts.to,
            direction: parts.direction,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationAuthorizationTraversalParts {
        WorthQueryPortableApplicationAuthorizationTraversalParts {
            relation: self.relation.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
            direction: self.direction,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationAuthorizationPredicateParts {
    pub traversal_ordinal: usize,
    pub entity: String,
    pub aspect: String,
    pub field: String,
    pub value: AspectValue,
    pub comparison: ApplicationAuthorizationComparison,
}

impl ApplicationAuthorizationPredicate {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationAuthorizationPredicateParts,
    ) -> Self {
        Self {
            traversal_ordinal: parts.traversal_ordinal,
            entity: parts.entity,
            aspect: parts.aspect,
            field: parts.field,
            value: parts.value,
            comparison: parts.comparison,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationAuthorizationPredicateParts {
        WorthQueryPortableApplicationAuthorizationPredicateParts {
            traversal_ordinal: self.traversal_ordinal,
            entity: self.entity.clone(),
            aspect: self.aspect.clone(),
            field: self.field.clone(),
            value: self.value.clone(),
            comparison: self.comparison,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationAuthorizationFieldConstraintParts {
    pub traversal_ordinal: usize,
    pub entity: String,
    pub aspect: String,
    pub field: String,
    pub comparison: ApplicationAuthorizationComparison,
    pub principal_aspect: String,
    pub principal_field: String,
}

impl ApplicationAuthorizationFieldConstraint {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationAuthorizationFieldConstraintParts,
    ) -> Self {
        Self {
            traversal_ordinal: parts.traversal_ordinal,
            entity: parts.entity,
            aspect: parts.aspect,
            field: parts.field,
            comparison: parts.comparison,
            principal_aspect: parts.principal_aspect,
            principal_field: parts.principal_field,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationAuthorizationFieldConstraintParts {
        WorthQueryPortableApplicationAuthorizationFieldConstraintParts {
            traversal_ordinal: self.traversal_ordinal,
            entity: self.entity.clone(),
            aspect: self.aspect.clone(),
            field: self.field.clone(),
            comparison: self.comparison,
            principal_aspect: self.principal_aspect.clone(),
            principal_field: self.principal_field.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationAuthorizationPathParts {
    pub effect: ApplicationAuthorizationPathEffect,
    pub principal_entity: String,
    pub scope_entity: String,
    pub traversals: Vec<ApplicationAuthorizationTraversal>,
    pub predicates: Vec<ApplicationAuthorizationPredicate>,
    pub field_constraints: Vec<ApplicationAuthorizationFieldConstraint>,
}

impl ApplicationAuthorizationPath {
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableApplicationAuthorizationPathParts,
    ) -> Self {
        Self {
            effect: parts.effect,
            principal_entity: parts.principal_entity,
            scope_entity: parts.scope_entity,
            traversals: parts.traversals,
            predicates: parts.predicates,
            field_constraints: parts.field_constraints,
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationAuthorizationPathParts {
        WorthQueryPortableApplicationAuthorizationPathParts {
            effect: self.effect,
            principal_entity: self.principal_entity.clone(),
            scope_entity: self.scope_entity.clone(),
            traversals: self.traversals.clone(),
            predicates: self.predicates.clone(),
            field_constraints: self.field_constraints.clone(),
        }
    }
}
