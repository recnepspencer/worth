//! Closure of a declared path's predicates and principal-field constraints.
//!
//! Archives are untrusted: an `AtMost`/`AtLeast` comparison is closed only on
//! an order-faithful scalar family. `StrictlyLess`/`StrictlyGreater` remain
//! admissible on every family because `ne` lowers to that pair, which is sound
//! for any total order consistent with equality.
use worth_foundational::facade::ScalarAspectType;

use super::super::{
    ApplicationAuthorizationComparison as Comparison, ApplicationAuthorizationPath,
    ApplicationSchemaMember,
};
use super::ClosureIndex;

impl ClosureIndex<'_> {
    pub(super) fn path_conditions_are_closed(
        &self,
        path: &ApplicationAuthorizationPath,
        entity_by_ordinal: &[&str],
    ) -> bool {
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
