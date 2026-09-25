//! Exact member-scope tokens retained alongside Query-issued entry identities.

use std::any::Any;
use std::collections::BTreeMap;

use worth_relational::facade::identity::EntityId;

/// A certified membership row's exact token for admitting an entry query.
/// Token equality is required in addition to entity identity for reuse.
pub trait WorthQueryManagedDerivedMemberToken: Clone + Eq + Send + Sync + 'static {
    fn retained_bytes(&self) -> usize;
}

impl WorthQueryManagedDerivedMemberToken for String {
    fn retained_bytes(&self) -> usize {
        self.capacity()
    }
}

pub(in crate::domain_computation::primary_graph::application_query::derived_view) struct RetainedMemberToken
{
    value: Box<dyn Any + Send + Sync>,
    charged_bytes: usize,
}

impl RetainedMemberToken {
    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn new<
        Member: WorthQueryManagedDerivedMemberToken,
    >(
        value: Member,
    ) -> Self {
        let charged_bytes = std::mem::size_of::<Member>()
            .saturating_add(value.retained_bytes())
            .saturating_add(std::mem::size_of::<EntityId>() + 4 * std::mem::size_of::<usize>());
        Self {
            value: Box::new(value),
            charged_bytes,
        }
    }

    pub(super) fn matches<Member: WorthQueryManagedDerivedMemberToken>(
        &self,
        other: &Member,
    ) -> bool {
        self.value.downcast_ref::<Member>() == Some(other)
    }
}

pub(super) fn token_charge(tokens: &BTreeMap<EntityId, RetainedMemberToken>) -> usize {
    tokens.values().fold(0usize, |bytes, token| {
        bytes.saturating_add(token.charged_bytes)
    })
}
