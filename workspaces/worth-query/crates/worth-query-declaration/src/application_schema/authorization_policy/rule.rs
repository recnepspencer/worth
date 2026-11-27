//! A ReBAC rule over one principal and scope: allow and deny paths.
//!
//! `actor_is_not_target_of`/`actor_is_not_source_of` deny the principal that
//! a scope's relation names. A deny path only fires on a recorded edge, so
//! each requires, at compile time, that every live scope has at least one
//! such edge. A relation without that minimum does not compile:
//!
//! ```compile_fail
//! # use worth_query_declaration::facade::application_schema::{fragment::{self, Relation}, *};
//! struct Case;
//! struct User;
//! impl fragment::Entity for User { const NAME: &'static str = "User"; type Aspects = (); type Relations = (); }
//! struct Author; // declares no `source_min`
//! impl Relation for Author { const NAME: &'static str = "Author"; type To = User; }
//! let author = ApplicationRelationRef::<(), Author, Case, User>::from_schema_identifiers(
//!     "Author", "Case", "User", fragment::relation_integrity::<Author>(),
//! );
//! let _ = ApplicationAuthorizationRule::<(), User, Case>::new().actor_is_not_target_of(author);
//! ```
//!
//! The same rule compiles once every case must name its author:
//!
//! ```
//! # use worth_query_declaration::facade::application_schema::{fragment::{self, Relation}, *};
//! # struct Case;
//! # struct User;
//! # impl fragment::Entity for User { const NAME: &'static str = "User"; type Aspects = (); type Relations = (); }
//! struct Author;
//! impl Relation for Author {
//!     const NAME: &'static str = "Author";
//!     type To = User;
//!     const CARDINALITY: ApplicationRelationCardinality =
//!         ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, None);
//! }
//! let author = ApplicationRelationRef::<(), Author, Case, User>::from_schema_identifiers(
//!     "Author", "Case", "User", fragment::relation_integrity::<Author>(),
//! );
//! let _ = ApplicationAuthorizationRule::<(), User, Case>::new().actor_is_not_target_of(author);
//! ```
use std::marker::PhantomData;

use super::{
    ApplicationAuthorizationPath, ApplicationAuthorizationPathEffect,
    ApplicationAuthorizationTraversal, ApplicationAuthorizationTraversalDirection,
};
use crate::application_schema::references::ApplicationRelationRef;
use crate::application_schema::{fragment, ApplicationAuthorizationRuleDenial};

/// The kernel paths one authored path lowered to, or why it could not lower.
pub struct ApplicationAuthorizationPathSet<Schema, Principal, Scope> {
    paths: Result<Vec<ApplicationAuthorizationPath>, ApplicationAuthorizationRuleDenial>,
    _marker: PhantomData<fn() -> (Schema, Principal, Scope)>,
}

impl<Schema, Principal, Scope> ApplicationAuthorizationPathSet<Schema, Principal, Scope> {
    pub(super) fn lowered(
        paths: Result<Vec<ApplicationAuthorizationPath>, ApplicationAuthorizationRuleDenial>,
    ) -> Self {
        Self {
            paths,
            _marker: PhantomData,
        }
    }

    /// The lowered kernel paths, or the typed refusal naming the authored path.
    pub fn into_paths(
        self,
    ) -> Result<Vec<ApplicationAuthorizationPath>, ApplicationAuthorizationRuleDenial> {
        self.paths
    }
}

/// An exactly built path joins a rule unchanged; installation still checks
/// that its principal and scope match.
impl<Schema, Principal, Scope> From<ApplicationAuthorizationPath>
    for ApplicationAuthorizationPathSet<Schema, Principal, Scope>
{
    fn from(path: ApplicationAuthorizationPath) -> Self {
        Self::lowered(Ok(vec![path]))
    }
}

pub struct ApplicationAuthorizationRule<Schema, Principal, Scope> {
    paths: Result<Vec<ApplicationAuthorizationPath>, ApplicationAuthorizationRuleDenial>,
    _marker: PhantomData<fn() -> (Schema, Principal, Scope)>,
}

impl<Schema, Principal, Scope> Default for ApplicationAuthorizationRule<Schema, Principal, Scope> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Schema, Principal, Scope> From<ApplicationAuthorizationPathSet<Schema, Principal, Scope>>
    for ApplicationAuthorizationRule<Schema, Principal, Scope>
{
    fn from(set: ApplicationAuthorizationPathSet<Schema, Principal, Scope>) -> Self {
        Self::new().path(set)
    }
}

impl<Schema, Principal, Scope> ApplicationAuthorizationRule<Schema, Principal, Scope> {
    pub fn new() -> Self {
        Self {
            paths: Ok(Vec::new()),
            _marker: PhantomData,
        }
    }

    /// Adds an allow or deny path. Deny wins over every allow path.
    pub fn path(
        mut self,
        set: impl Into<ApplicationAuthorizationPathSet<Schema, Principal, Scope>>,
    ) -> Self {
        let lowered = set.into().paths;
        if let Ok(paths) = &mut self.paths {
            match lowered {
                Ok(mut more) => paths.append(&mut more),
                Err(denial) => self.paths = Err(denial),
            }
        }
        self
    }

    /// Denies the principal named by the scope's `relation` (for example, a
    /// case's author). Every live scope must have at least one such edge
    /// (`source_min >= 1`), so the deny cannot fail open on a missing edge.
    pub fn actor_is_not_target_of<R: fragment::Relation>(
        self,
        relation: ApplicationRelationRef<Schema, R, Scope, Principal>,
    ) -> Self {
        const {
            assert!(
                at_least_one(R::CARDINALITY.source_min),
                "actor_is_not_target_of needs `source_min >= 1`: every scope must name its target"
            )
        };
        self.deny_one_step(relation.name(), relation.from(), relation.to(), false)
    }

    /// Denies a principal with a `relation` edge into the scope (for example,
    /// a payment's initiator). Every live scope must have at least one such
    /// edge (`target_min >= 1`), so the deny cannot fail open on a missing edge.
    pub fn actor_is_not_source_of<R: fragment::Relation>(
        self,
        relation: ApplicationRelationRef<Schema, R, Principal, Scope>,
    ) -> Self {
        const {
            assert!(
                at_least_one(R::CARDINALITY.target_min),
                "actor_is_not_source_of needs `target_min >= 1`: every scope must have its source"
            )
        };
        self.deny_one_step(relation.name(), relation.from(), relation.to(), true)
    }

    /// The installed paths, or the first typed refusal; a rule without an
    /// allow path admits nobody and is refused.
    pub fn into_paths(
        self,
    ) -> Result<Vec<ApplicationAuthorizationPath>, ApplicationAuthorizationRuleDenial> {
        let mut paths = self.paths?;
        if !paths
            .iter()
            .any(|path| path.effect == ApplicationAuthorizationPathEffect::Allow)
        {
            return Err(ApplicationAuthorizationRuleDenial::NoAllowPath);
        }
        paths.sort();
        paths.dedup();
        Ok(paths)
    }

    fn deny_one_step(mut self, relation: &str, from: &str, to: &str, forward: bool) -> Self {
        let (principal, scope, direction) = match forward {
            true => (
                from,
                to,
                ApplicationAuthorizationTraversalDirection::Forward,
            ),
            false => (
                to,
                from,
                ApplicationAuthorizationTraversalDirection::Reverse,
            ),
        };
        if let Ok(paths) = &mut self.paths {
            paths.push(ApplicationAuthorizationPath {
                effect: ApplicationAuthorizationPathEffect::Deny,
                principal_entity: principal.to_owned(),
                scope_entity: scope.to_owned(),
                traversals: vec![ApplicationAuthorizationTraversal {
                    relation: relation.to_owned(),
                    from: from.to_owned(),
                    to: to.to_owned(),
                    direction,
                }],
                predicates: Vec::new(),
                field_constraints: Vec::new(),
            });
        }
        self
    }
}

const fn at_least_one(minimum: Option<u64>) -> bool {
    matches!(minimum, Some(minimum) if minimum >= 1)
}
