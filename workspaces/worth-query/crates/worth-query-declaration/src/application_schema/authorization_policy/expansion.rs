//! Lowers one authored path to the kernel paths its conditions mean.
//!
//! A condition with several alternatives (`one_of`, `ne`, `ne_actor`)
//! multiplies the path; the product is bounded so one authored line cannot
//! silently install an unreviewable policy.
use super::condition::{Alternative, ResolvedCondition};
use super::{
    ApplicationAuthorizationFieldConstraint, ApplicationAuthorizationPath,
    ApplicationAuthorizationPathEffect, ApplicationAuthorizationPredicate,
    ApplicationAuthorizationTraversalDirection,
};
use crate::application_schema::ApplicationAuthorizationRuleDenial as RuleDenial;

/// The most kernel paths one authored path may lower to.
pub const MAXIMUM_LOWERED_PATHS: usize = 16;

type AuthoredCondition = (usize, Result<ResolvedCondition, RuleDenial>);

pub(super) fn expand(
    base: ApplicationAuthorizationPath,
    conditions: Vec<AuthoredCondition>,
) -> Result<Vec<ApplicationAuthorizationPath>, RuleDenial> {
    let mut resolved = Vec::with_capacity(conditions.len());
    for (ordinal, condition) in conditions {
        resolved.push((ordinal, condition?));
    }
    let described = || describe(&base, &resolved);
    if base.effect == ApplicationAuthorizationPathEffect::Deny {
        if let Some((_, optional)) = resolved.iter().find(|(_, condition)| condition.optional) {
            return Err(RuleDenial::DenyConditionOnOptionalField {
                path: described(),
                condition: optional.description.clone(),
            });
        }
    }
    let lowered = resolved.iter().fold(1_usize, |product, (_, condition)| {
        product.saturating_mul(condition.alternatives.len())
    });
    if lowered > MAXIMUM_LOWERED_PATHS {
        return Err(RuleDenial::ExpansionLimit {
            path: described(),
            expanding: resolved
                .iter()
                .filter(|(_, condition)| condition.alternatives.len() > 1)
                .map(|(_, condition)| condition.description.clone())
                .collect(),
            lowered,
            limit: MAXIMUM_LOWERED_PATHS,
        });
    }
    let mut paths = vec![base];
    for (ordinal, condition) in &resolved {
        paths = paths
            .into_iter()
            .flat_map(|path| {
                condition
                    .alternatives
                    .iter()
                    .map(move |alternative| with(path.clone(), *ordinal, condition, alternative))
            })
            .collect();
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn with(
    mut path: ApplicationAuthorizationPath,
    ordinal: usize,
    condition: &ResolvedCondition,
    alternative: &Alternative,
) -> ApplicationAuthorizationPath {
    match alternative {
        Alternative::Value(comparison, value) => {
            path.predicates.push(ApplicationAuthorizationPredicate {
                traversal_ordinal: ordinal,
                entity: condition.entity.to_owned(),
                aspect: condition.aspect.clone(),
                field: condition.field.clone(),
                value: value.clone(),
                comparison: *comparison,
            });
        }
        Alternative::Principal(comparison, principal_aspect, principal_field) => {
            path.field_constraints
                .push(ApplicationAuthorizationFieldConstraint {
                    traversal_ordinal: ordinal,
                    entity: condition.entity.to_owned(),
                    aspect: condition.aspect.clone(),
                    field: condition.field.clone(),
                    comparison: *comparison,
                    principal_aspect: principal_aspect.clone(),
                    principal_field: principal_field.clone(),
                });
        }
    }
    path
}

/// The authored path as its author wrote it, for diagnostics.
fn describe(
    path: &ApplicationAuthorizationPath,
    conditions: &[(usize, ResolvedCondition)],
) -> String {
    let at = |ordinal: usize| {
        conditions
            .iter()
            .filter(|(at, _)| *at == ordinal)
            .map(|(_, condition)| condition.description.as_str())
            .collect::<Vec<_>>()
    };
    let mut text = path.principal_entity.clone();
    let push_conditions = |text: &mut String, ordinal| {
        let here = at(ordinal);
        if !here.is_empty() {
            text.push_str(&format!(" [{}]", here.join(", ")));
        }
    };
    push_conditions(&mut text, 0);
    for (index, traversal) in path.traversals.iter().enumerate() {
        let relation = &traversal.relation;
        text.push_str(&match traversal.direction {
            ApplicationAuthorizationTraversalDirection::Forward => {
                format!(" -{relation}-> {}", traversal.to)
            }
            ApplicationAuthorizationTraversalDirection::Reverse => {
                format!(" <-{relation}- {}", traversal.from)
            }
        });
        push_conditions(&mut text, index + 1);
    }
    text
}
