use std::collections::BTreeSet;

use worth_foundational::facade::AspectKey;

use crate::identity::data::KindId;
use crate::schema::data::{AspectExclusionGroupDeclaration, SchemaRegistryError};

use super::declarations::RegistrationDomain;

pub(super) fn canonicalize(
    kind_id: KindId,
    mut groups: Vec<AspectExclusionGroupDeclaration>,
    aspects: &BTreeSet<AspectKey>,
    domain: &RegistrationDomain,
) -> Result<Vec<AspectExclusionGroupDeclaration>, SchemaRegistryError> {
    groups.sort_by(|left, right| left.group_id.cmp(&right.group_id));
    let mut previous_id = None;
    for group in &groups {
        let invalid = |reason| {
            SchemaRegistryError::invalid_aspect_declaration(
                kind_id,
                format!("aspect exclusion group {:?}: {reason}", group.group_id),
            )
        };
        if !matches!(domain, RegistrationDomain::Entity) {
            return Err(invalid("only entity kinds may declare an exclusion group"));
        }
        if !worth_foundational::is_valid_simple_identifier(&group.group_id) {
            return Err(invalid("group ID must be a simple identifier"));
        }
        if previous_id == Some(group.group_id.as_str()) {
            return Err(invalid("duplicate group ID"));
        }
        previous_id = Some(group.group_id.as_str());
        // A singleton required group requires its sole aspect. A singleton
        // optional group accepts either state, as does Query's singleton OneOf.
        if group.aspects.is_empty() {
            return Err(invalid("at least one member aspect is required"));
        }
        if !group.aspects.is_subset(aspects) {
            return Err(invalid("references an undeclared aspect"));
        }
    }
    Ok(groups)
}
