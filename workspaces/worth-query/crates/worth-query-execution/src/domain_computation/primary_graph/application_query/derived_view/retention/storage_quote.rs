//! Checked logical storage for the actual retained projection image.
use super::*;

/// Query's conservative structural storage quote for a selected managed image.
/// Values supply logical retained charges and may overcount shared backing.
/// This is not pool funding, allocator capacity, RSS, or currentness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedDerivedStorageQuote {
    entries: usize,
    retained_bytes: usize,
}
impl WorthQueryManagedDerivedStorageQuote {
    pub const fn entries(self) -> usize {
        self.entries
    }
    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
}

pub(in crate::domain_computation::primary_graph::application_query::derived_view) enum ManagedStoragePolicy
{
    Bounded(ApplicationDerivedViewLimits),
    OwnerSized,
}
impl ManagedStoragePolicy {
    pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn rejects_entries(
        &self,
        count: usize,
    ) -> bool {
        matches!(self, Self::Bounded(limits) if count > limits.maximum_entries())
    }
    pub(super) fn rejects_bytes(&self, bytes: usize) -> bool {
        matches!(self, Self::Bounded(limits) if bytes > limits.maximum_retained_bytes())
    }
}

pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn image_quote<
    'a,
    Key: Clone + Ord,
    Value: WorthQueryManagedDerivedValue,
>(
    entries: impl Iterator<Item = (&'a Value, &'a BTreeSet<ViewDependency>)>,
    count: usize,
    membership: &BTreeSet<ViewDependency>,
    tokens: Option<&BTreeMap<EntityId, RetainedMemberToken>>,
) -> Result<usize, WorthQueryManagedDerivedViewDenial> {
    let mut bytes = std::mem::size_of::<Key>();
    let mut actual = 0usize;
    bytes = bytes
        .checked_add(dependency_quote::<Key>(membership)?)
        .ok_or(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)?;
    for (value, dependencies) in entries {
        actual = actual
            .checked_add(1)
            .ok_or(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)?;
        bytes = bytes
            .checked_add(entry_quote::<Key, Value>(value, dependencies)?)
            .ok_or(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)?;
    }
    if actual != count {
        return Err(WorthQueryManagedDerivedViewDenial::IncompleteDependencies);
    }
    let dirty = count
        .checked_mul(std::mem::size_of::<Key>() + 4 * std::mem::size_of::<usize>())
        .ok_or(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)?;
    let tokens = tokens.map_or(Ok(0), |tokens| {
        tokens.values().try_fold(0usize, |bytes, token| {
            bytes
                .checked_add(token.charged_bytes)
                .ok_or(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)
        })
    })?;
    bytes
        .checked_add(dirty)
        .and_then(|n| n.checked_add(tokens))
        .ok_or(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)
}

fn dependency_quote<Key: Clone + Ord>(
    dependencies: &BTreeSet<ViewDependency>,
) -> Result<usize, WorthQueryManagedDerivedViewDenial> {
    dependencies
        .iter()
        .try_fold(0usize, |bytes, dependency| {
            bytes
                .checked_add(dependency.retained_bytes())?
                .checked_add(DependencyIndex::<Key>::entry_insertion_bound(dependency))
        })
        .ok_or(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)
}

pub(in crate::domain_computation::primary_graph::application_query::derived_view) fn entry_quote<
    Key: Clone + Ord,
    Value: WorthQueryManagedDerivedValue,
>(
    value: &Value,
    dependencies: &BTreeSet<ViewDependency>,
) -> Result<usize, WorthQueryManagedDerivedViewDenial> {
    value
        .retained_bytes()
        .checked_add(std::mem::size_of::<(Key, RetainedEntry<Value>)>())
        .and_then(|n| n.checked_add(4 * std::mem::size_of::<usize>()))
        .and_then(|n| n.checked_add(dependency_quote::<Key>(dependencies).ok()?))
        .ok_or(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)
}

impl<Key: Clone + Ord, Value: WorthQueryManagedDerivedValue>
    WorthQueryManagedDerivedViewSnapshot<Key, Value>
{
    /// Describes the owner's actual selected image; it grants no future access.
    pub fn storage_quote(
        &self,
    ) -> Result<WorthQueryManagedDerivedStorageQuote, WorthQueryManagedDerivedViewDenial> {
        let retained = self
            .state
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if retained.disposed {
            return Err(WorthQueryManagedDerivedViewDenial::Disposed);
        }
        if retained.revision_exhausted {
            return Err(WorthQueryManagedDerivedViewDenial::ViewRevisionExhausted);
        }
        if retained.current_commit != self.commit {
            return Err(WorthQueryManagedDerivedViewDenial::StaleSource);
        }
        Ok(WorthQueryManagedDerivedStorageQuote {
            entries: retained.entries.len(),
            retained_bytes: retained.charged_bytes,
        })
    }
}
