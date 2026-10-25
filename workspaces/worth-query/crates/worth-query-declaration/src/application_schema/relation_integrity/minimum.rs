//! Portable minimum reachability and completed-schema kind membership.
use super::super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

/// A positive distinct-target minimum over a set of authored entity names.
/// Completed schema validation checks membership and nonempty, positive meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationRelationMinimumReachability {
    pub minimum_reachable_targets: u32,
    pub target_kinds: &'static [&'static str],
}

pub(super) fn validate_minimum_kinds(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let minima = || {
        members.iter().filter_map(|member| match member {
            ApplicationSchemaMember::Relation { integrity, .. } => {
                integrity.reaches_at_least.as_ref()
            }
            _ => None,
        })
    };
    if minima().next().is_none() {
        return Ok(());
    }
    let entities: std::collections::BTreeSet<&str> = members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Entity { entity } => Some(entity.as_str()),
            _ => None,
        })
        .collect();
    for minimum in minima() {
        if minimum.minimum_reachable_targets == 0
            || minimum.target_kinds.is_empty()
            || minimum
                .target_kinds
                .iter()
                .any(|kind| !entities.contains(kind.as_str()))
        {
            return Err(ApplicationSchemaDeclarationDenial::InvalidRelationIntegrity);
        }
    }
    Ok(())
}
