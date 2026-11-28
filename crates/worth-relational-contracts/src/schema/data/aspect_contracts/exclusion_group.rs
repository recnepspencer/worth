use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use worth_foundational::facade::AspectKey;

/// Mutually exclusive active aspects of an entity kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectExclusionGroupDeclaration {
    pub group_id: String,
    pub aspects: BTreeSet<AspectKey>,
    /// Require one active member. Optional groups also accept zero.
    pub required: bool,
}

impl AspectExclusionGroupDeclaration {
    pub fn member_tree_bytes(members: usize) -> Option<usize> {
        if members == 0 {
            return Some(0);
        }
        usize::try_from(
            crate::branch::root::schema::standard_tree_quote::retained_tree_bytes::<AspectKey, ()>(
                members,
            )
            .ok()?,
        )
        .ok()
    }

    pub fn allocation_bytes(groups: &Vec<Self>) -> Option<usize> {
        let mut bytes = groups.capacity().checked_mul(size_of::<Self>())?;
        for group in groups {
            bytes = bytes
                .checked_add(group.group_id.capacity())?
                .checked_add(Self::member_tree_bytes(group.aspects.len())?)?;
            for key in &group.aspects {
                bytes = bytes.checked_add(key.owned_allocation_capacity_bytes())?;
            }
        }
        Some(bytes)
    }
}
