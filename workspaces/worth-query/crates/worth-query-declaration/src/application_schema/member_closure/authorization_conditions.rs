//! Closure of a declared path's predicates and principal-field constraints.
//!
//! A deny path fires only on observed values, so every value it reads (each
//! predicate field, constrained field and principal field) must be a required
//! field of an aspect every entity of the kind always has. An optional field,
//! an `Option<A>` or extension aspect, or an `Exclusive<..>` case aspect can be
//! absent, and absence would silently disable the deny.
//!
//! Archives are untrusted: an `AtMost`/`AtLeast` comparison is closed only on
//! an order-faithful scalar family. `StrictlyLess`/`StrictlyGreater` remain
//! admissible on every family because `ne` lowers to that pair, which is sound
//! for any total order consistent with equality.
use worth_foundational::facade::ScalarAspectType;

use super::super::{
    ApplicationAuthorizationComparison as Comparison, ApplicationAuthorizationPath,
    ApplicationAuthorizationPathEffect, ApplicationAuthorizationRuleDenial,
    ApplicationAuthorizationTraversalDirection, ApplicationFieldPresence, ApplicationSchemaMember,
};
use super::ClosureIndex;

impl ClosureIndex<'_> {
    /// The typed refusal of a deny path that reads a value which can be absent.
    pub(super) fn deny_reads_absent_value(
        &self,
        path: &ApplicationAuthorizationPath,
    ) -> Option<ApplicationAuthorizationRuleDenial> {
        if path.effect() != ApplicationAuthorizationPathEffect::Deny {
            return None;
        }
        let principal = path.principal_entity();
        let read = path
            .predicates()
            .iter()
            .map(|predicate| (predicate.entity(), predicate.aspect(), predicate.field()))
            .chain(path.field_constraints().iter().flat_map(|constraint| {
                [
                    (constraint.entity(), constraint.aspect(), constraint.field()),
                    (
                        principal,
                        constraint.principal_aspect(),
                        constraint.principal_field(),
                    ),
                ]
            }));
        for (entity, aspect, field) in read {
            let why = match (
                self.field_presence(entity, aspect, field),
                self.aspect_required(entity, aspect),
            ) {
                (Some(ApplicationFieldPresence::Required), true) => continue,
                (_, false) => "its aspect can be absent",
                _ => "the field can be absent",
            };
            return Some(
                ApplicationAuthorizationRuleDenial::DenyConditionOnOptionalField {
                    path: describe(path),
                    condition: format!("{entity}.{aspect}.{field} ({why})"),
                },
            );
        }
        None
    }

    pub(super) fn path_conditions_are_closed(
        &self,
        path: &ApplicationAuthorizationPath,
        entity_by_ordinal: &[&str],
    ) -> bool {
        if self.deny_reads_absent_value(path).is_some() {
            return false;
        }
        let reached = |ordinal: usize, entity: &str| {
            entity_by_ordinal
                .get(ordinal)
                .is_some_and(|reached| *reached == entity)
        };
        path.predicates().iter().all(|predicate| {
            reached(predicate.traversal_ordinal(), predicate.entity())
                && self.fields.contains(&(
                    predicate.entity(),
                    predicate.aspect(),
                    predicate.field(),
                ))
                && self.field_is_equality_queryable(
                    predicate.entity(),
                    predicate.aspect(),
                    predicate.field(),
                )
                && self.comparison_is_faithful(
                    predicate.comparison(),
                    predicate.entity(),
                    predicate.aspect(),
                    predicate.field(),
                )
        }) && path.field_constraints().iter().all(|constraint| {
            let principal = path.principal_entity();
            reached(constraint.traversal_ordinal(), constraint.entity())
                && self.field_is_equality_queryable(
                    constraint.entity(),
                    constraint.aspect(),
                    constraint.field(),
                )
                && self.field_is_equality_queryable(
                    principal,
                    constraint.principal_aspect(),
                    constraint.principal_field(),
                )
                && self.comparison_is_faithful(
                    constraint.comparison(),
                    constraint.entity(),
                    constraint.aspect(),
                    constraint.field(),
                )
                && self.field_family(constraint.entity(), constraint.aspect(), constraint.field())
                    == self.field_family(
                        principal,
                        constraint.principal_aspect(),
                        constraint.principal_field(),
                    )
        })
    }

    fn comparison_is_faithful(
        &self,
        comparison: Comparison,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> bool {
        match comparison {
            Comparison::AtMost | Comparison::AtLeast => self
                .field_family(entity, aspect, field)
                .is_some_and(order_faithful),
            Comparison::Equal | Comparison::StrictlyLess | Comparison::StrictlyGreater => true,
        }
    }

    fn field_presence(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<ApplicationFieldPresence> {
        self.members.iter().find_map(|member| match member {
            ApplicationSchemaMember::Field {
                entity: candidate_entity,
                aspect: candidate_aspect,
                field: candidate_field,
                presence,
                ..
            } if candidate_entity == entity
                && candidate_aspect == aspect
                && candidate_field == field =>
            {
                Some(*presence)
            }
            _ => None,
        })
    }

    /// Only a `Required` aspect is present on every entity of its kind.
    fn aspect_required(&self, entity: &str, aspect: &str) -> bool {
        self.members.iter().any(|member| {
            matches!(member, ApplicationSchemaMember::Aspect {
                entity: candidate_entity,
                aspect: candidate_aspect,
                absence: worth_foundational::facade::AbsenceLaw::Required,
                ..
            } if candidate_entity == entity && candidate_aspect == aspect)
        })
    }

    fn field_family(&self, entity: &str, aspect: &str, field: &str) -> Option<ScalarAspectType> {
        self.members.iter().find_map(|member| match member {
            ApplicationSchemaMember::Field {
                entity: candidate_entity,
                aspect: candidate_aspect,
                field: candidate_field,
                scalar_family,
                ..
            } if candidate_entity == entity
                && candidate_aspect == aspect
                && candidate_field == field =>
            {
                Some(*scalar_family)
            }
            _ => None,
        })
    }
}

/// The installed path as `Principal -Relation-> Entity <-Relation- Scope`.
fn describe(path: &ApplicationAuthorizationPath) -> String {
    path.traversals()
        .iter()
        .fold(path.principal_entity().to_owned(), |mut text, traversal| {
            let relation = traversal.relation();
            text.push_str(&match traversal.direction() {
                ApplicationAuthorizationTraversalDirection::Forward => {
                    format!(" -{relation}-> {}", traversal.to())
                }
                ApplicationAuthorizationTraversalDirection::Reverse => {
                    format!(" <-{relation}- {}", traversal.from())
                }
            });
            text
        })
}

/// Families whose `AspectValue` order is their value order.
const fn order_faithful(family: ScalarAspectType) -> bool {
    matches!(
        family,
        ScalarAspectType::Int8
            | ScalarAspectType::Int16
            | ScalarAspectType::Int32
            | ScalarAspectType::Int64
            | ScalarAspectType::UInt8
            | ScalarAspectType::UInt16
            | ScalarAspectType::UInt32
            | ScalarAspectType::UInt64
            | ScalarAspectType::Date
            | ScalarAspectType::Time
            | ScalarAspectType::Timestamp
    )
}

#[cfg(test)]
#[path = "authorization_conditions_tests.rs"]
mod tests;
