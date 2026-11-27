//! The one path builder: typed traversals from a principal to a scope.
//!
//! `ExactPath` keeps the original single-path surface (`where_equal`,
//! `allow`/`deny` returning one [`ApplicationAuthorizationPath`]). The first
//! `when` moves to `AuthoredPath`, whose conditions may lower to several
//! paths, so its `allow`/`deny` return an [`ApplicationAuthorizationPathSet`].
//! A principal-field condition only joins a path from that principal:
//!
//! ```compile_fail
//! # use worth_query_declaration::facade::application_schema::*;
//! # fn wrong<S, User, Clerk, Case, F, P>(user: ApplicationEntityRef<S, User>, case: ApplicationEntityRef<S, Case>,
//! #     to_case: ApplicationRelationRef<S, (), User, Case>, field: F, clerk_key: P)
//! # where F: fragment::RecordSelection<Entity = Case>, P: fragment::RecordSelection<Entity = Clerk, Binding = F::Binding> {
//! ApplicationAuthorizationPathBuilder::from_principal(user)
//!     .forward(to_case)
//!     .when(field.eq_actor(clerk_key)) // Clerk is not this path's principal
//!     .allow(case);
//! # }
//! ```
//!
//! The same condition joins a path from its own principal:
//!
//! ```
//! # use worth_query_declaration::facade::application_schema::*;
//! # fn right<S, User, Case, F, P>(user: ApplicationEntityRef<S, User>, case: ApplicationEntityRef<S, Case>,
//! #     to_case: ApplicationRelationRef<S, (), User, Case>, field: F, user_key: P)
//! # where F: fragment::RecordSelection<Entity = Case>, P: fragment::RecordSelection<Entity = User, Binding = F::Binding> {
//! ApplicationAuthorizationPathBuilder::from_principal(user)
//!     .forward(to_case)
//!     .when(field.ne_actor(user_key))
//!     .allow(case);
//! # }
//! ```
use std::marker::PhantomData;

use super::condition::{NoActor, PrincipalActor, ResolvedCondition};
use super::{
    expansion, ApplicationAuthorizationCondition, ApplicationAuthorizationPath,
    ApplicationAuthorizationPathEffect as Effect, ApplicationAuthorizationPathSet,
    ApplicationAuthorizationPredicate, ApplicationAuthorizationTraversal,
    ApplicationAuthorizationTraversalDirection as Direction,
};
use crate::application_schema::references::{ApplicationEntityRef, ApplicationRelationRef};
use crate::application_schema::{
    fragment, ApplicationAuthorizationRuleDenial, ApplicationEncodedScalarValue,
    ApplicationSchemaDeclarationDenial,
};

/// Builder mode before any authored condition.
pub struct ExactPath;
/// Builder mode once a condition has been authored with `when`.
pub struct AuthoredPath;

pub struct ApplicationAuthorizationPathBuilder<
    Schema,
    Current,
    Principal = Current,
    Mode = ExactPath,
> {
    principal_entity: &'static str,
    traversals: Vec<ApplicationAuthorizationTraversal>,
    predicates: Vec<ApplicationAuthorizationPredicate>,
    conditions: Vec<(
        usize,
        Result<ResolvedCondition, ApplicationAuthorizationRuleDenial>,
    )>,
    _marker: PhantomData<fn() -> (Schema, Current, Principal, Mode)>,
}

/// Which principal a condition may compare against.
pub trait ApplicationAuthorizationActor<Principal> {}
impl<Principal> ApplicationAuthorizationActor<Principal> for NoActor {}
impl<Principal> ApplicationAuthorizationActor<Principal> for PrincipalActor<Principal> {}

impl<Schema, Principal> ApplicationAuthorizationPathBuilder<Schema, Principal, Principal> {
    pub fn from_principal(principal: ApplicationEntityRef<Schema, Principal>) -> Self {
        Self {
            principal_entity: principal.name(),
            traversals: Vec::new(),
            predicates: Vec::new(),
            conditions: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<Schema, Principal, Current, Mode>
    ApplicationAuthorizationPathBuilder<Schema, Current, Principal, Mode>
{
    pub fn forward<Relation, Next>(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Current, Next>,
    ) -> ApplicationAuthorizationPathBuilder<Schema, Next, Principal, Mode> {
        self.step(
            relation.name(),
            relation.from(),
            relation.to(),
            Direction::Forward,
        )
    }

    pub fn reverse<Relation, Previous>(
        self,
        relation: ApplicationRelationRef<Schema, Relation, Previous, Current>,
    ) -> ApplicationAuthorizationPathBuilder<Schema, Previous, Principal, Mode> {
        self.step(
            relation.name(),
            relation.from(),
            relation.to(),
            Direction::Reverse,
        )
    }

    /// Requires `condition` on the entity reached so far.
    pub fn when<Actor: ApplicationAuthorizationActor<Principal>>(
        mut self,
        condition: ApplicationAuthorizationCondition<Current, Actor>,
    ) -> ApplicationAuthorizationPathBuilder<Schema, Current, Principal, AuthoredPath> {
        self.conditions
            .push((self.traversals.len(), condition.resolved));
        self.into_mode()
    }

    fn step<Next>(
        mut self,
        relation: &str,
        from: &str,
        to: &str,
        direction: Direction,
    ) -> ApplicationAuthorizationPathBuilder<Schema, Next, Principal, Mode> {
        self.traversals.push(ApplicationAuthorizationTraversal {
            relation: relation.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            direction,
        });
        self.into_mode()
    }

    fn into_mode<Next, NextMode>(
        self,
    ) -> ApplicationAuthorizationPathBuilder<Schema, Next, Principal, NextMode> {
        ApplicationAuthorizationPathBuilder {
            principal_entity: self.principal_entity,
            traversals: self.traversals,
            predicates: self.predicates,
            conditions: self.conditions,
            _marker: PhantomData,
        }
    }

    fn path(self, scope: &str, effect: Effect) -> ApplicationAuthorizationPath {
        ApplicationAuthorizationPath {
            effect,
            principal_entity: self.principal_entity.to_string(),
            scope_entity: scope.to_string(),
            traversals: self.traversals,
            predicates: self.predicates,
            field_constraints: Vec::new(),
        }
    }
}

impl<Schema, Principal, Current> ApplicationAuthorizationPathBuilder<Schema, Current, Principal> {
    /// Selects the predicate's binding from the same record used by storage.
    pub fn where_record_equal<A, T>(
        mut self,
        field: fragment::RecordField<Current, A, T>,
        value: T,
    ) -> Result<Self, fragment::FieldEncodeRefusal> {
        if !field.equality_queryable() {
            return Err(fragment::FieldEncodeRefusal::new(
                format!(
                    "{}.{}.{}",
                    field.locus().entity(),
                    field.locus().aspect(),
                    field.locus().field()
                ),
                crate::application_schema::ApplicationValueEncodeDenial::CodecRejected {
                    binding_identity: field.recipe().binding_identity().clone(),
                    reason: "field does not support equality",
                },
            ));
        }
        let value = field.encode(&value)?;
        self.predicates.push(ApplicationAuthorizationPredicate {
            traversal_ordinal: self.traversals.len(),
            entity: field.locus().entity().to_owned(),
            aspect: field.locus().aspect().to_owned(),
            field: field.locus().field().to_owned(),
            value,
            comparison: Default::default(),
        });
        Ok(self)
    }

    pub fn where_equal<Field: fragment::RecordSelection<Entity = Current>>(
        mut self,
        field: Field,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<Self, ApplicationSchemaDeclarationDenial> {
        let checked = field.checked()?;
        if !checked.equality_queryable() {
            return Err(ApplicationSchemaDeclarationDenial::InvalidRecordField {
                field: checked.locus().clone(),
            });
        }
        self.predicates.push(ApplicationAuthorizationPredicate {
            traversal_ordinal: self.traversals.len(),
            entity: field.entity().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
            value: value.into_foundational_value(),
            comparison: Default::default(),
        });
        Ok(self)
    }

    pub fn allow(
        self,
        scope: ApplicationEntityRef<Schema, Current>,
    ) -> ApplicationAuthorizationPath {
        self.path(scope.name(), Effect::Allow)
    }

    pub fn deny(
        self,
        scope: ApplicationEntityRef<Schema, Current>,
    ) -> ApplicationAuthorizationPath {
        self.path(scope.name(), Effect::Deny)
    }
}

impl<Schema, Principal, Current>
    ApplicationAuthorizationPathBuilder<Schema, Current, Principal, AuthoredPath>
{
    /// Allows the principal when every authored condition holds on the path.
    pub fn allow(
        self,
        scope: ApplicationEntityRef<Schema, Current>,
    ) -> ApplicationAuthorizationPathSet<Schema, Principal, Current> {
        self.lower(scope.name(), Effect::Allow)
    }

    /// Denies the principal when the path and every authored condition hold.
    /// A deny condition must read required fields: an absent value never
    /// matches, so an optional field would let the deny fail open.
    pub fn deny(
        self,
        scope: ApplicationEntityRef<Schema, Current>,
    ) -> ApplicationAuthorizationPathSet<Schema, Principal, Current> {
        self.lower(scope.name(), Effect::Deny)
    }

    fn lower(
        mut self,
        scope: &str,
        effect: Effect,
    ) -> ApplicationAuthorizationPathSet<Schema, Principal, Current> {
        let conditions = std::mem::take(&mut self.conditions);
        let base = self.path(scope, effect);
        ApplicationAuthorizationPathSet::lowered(expansion::expand(base, conditions))
    }
}
