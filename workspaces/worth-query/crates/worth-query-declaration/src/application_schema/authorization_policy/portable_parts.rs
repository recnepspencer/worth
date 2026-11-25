use crate::archive_decode_authority::ArchiveDecodeAuthority;
use worth_foundational::facade::AspectValue;

use super::{
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
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationAuthorizationPredicateParts {
        WorthQueryPortableApplicationAuthorizationPredicateParts {
            traversal_ordinal: self.traversal_ordinal,
            entity: self.entity.clone(),
            aspect: self.aspect.clone(),
            field: self.field.clone(),
            value: self.value.clone(),
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
        }
    }

    pub fn parts(&self) -> WorthQueryPortableApplicationAuthorizationPathParts {
        WorthQueryPortableApplicationAuthorizationPathParts {
            effect: self.effect,
            principal_entity: self.principal_entity.clone(),
            scope_entity: self.scope_entity.clone(),
            traversals: self.traversals.clone(),
            predicates: self.predicates.clone(),
        }
    }
}
