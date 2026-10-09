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
    pub(super) charged_bytes: usize,
}

impl RetainedMemberToken {
    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn clone_admitted<
        Member: WorthQueryManagedDerivedMemberToken,
    >(
        value: &Member,
    ) -> Result<Self, super::WorthQueryManagedDerivedViewDenial> {
        let charged_bytes = Self::clone_quote(value)?;
        Ok(Self {
            value: Box::new(value.clone()),
            charged_bytes,
        })
    }

    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn clone_quote<
        Member: WorthQueryManagedDerivedMemberToken,
    >(
        value: &Member,
    ) -> Result<usize, super::WorthQueryManagedDerivedViewDenial> {
        std::mem::size_of::<Member>()
            .checked_add(value.retained_bytes())
            .and_then(|n| {
                n.checked_add(std::mem::size_of::<EntityId>() + 4 * std::mem::size_of::<usize>())
            })
            .ok_or(super::WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)
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
