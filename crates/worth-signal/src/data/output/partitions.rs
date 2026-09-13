mod fork_growth;
mod reserved_fork;
mod retained_charge;

use serde::{Deserialize, Serialize};

use super::OutputChange;

/// Generic opaque partition token for partitioned outputs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct PartitionToken(pub String);

impl PartitionToken {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<String> for PartitionToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for PartitionToken {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Generic changed-region descriptor for partition-aware outputs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct ChangedRegion {
    pub partition: PartitionToken,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CanonicalChangedRegions {
    regions: Vec<ChangedRegion>,
}

impl CanonicalChangedRegions {
    pub(crate) fn from_canonical_regions(regions: Vec<ChangedRegion>) -> Option<Self> {
        is_strict_region_order(&regions).then_some(Self { regions })
    }

    pub fn new(regions: impl IntoIterator<Item = ChangedRegion>) -> Self {
        Self::canonicalize_unordered(regions)
    }

    pub fn canonicalize_unordered(regions: impl IntoIterator<Item = ChangedRegion>) -> Self {
        let mut regions = regions.into_iter().collect::<Vec<_>>();
        if regions.len() > 1 {
            regions.sort_unstable();
            regions.dedup();
        }
        Self { regions }
    }

    pub fn from_ordered_unique(regions: impl IntoIterator<Item = ChangedRegion>) -> Self {
        let regions = regions.into_iter().collect::<Vec<_>>();
        debug_assert!(is_strict_region_order(regions.as_slice()));
        Self { regions }
    }

    pub fn from_slice(regions: &[ChangedRegion]) -> Self {
        Self::new(regions.iter().cloned())
    }

    pub fn as_slice(&self) -> &[ChangedRegion] {
        &self.regions
    }

    pub fn into_vec(self) -> Vec<ChangedRegion> {
        self.regions
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }
}

impl From<Vec<ChangedRegion>> for CanonicalChangedRegions {
    fn from(regions: Vec<ChangedRegion>) -> Self {
        Self::new(regions)
    }
}

impl From<&[ChangedRegion]> for CanonicalChangedRegions {
    fn from(regions: &[ChangedRegion]) -> Self {
        Self::from_slice(regions)
    }
}

/// How one partition subscription should match changed-region data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PartitionMatchMode {
    WholePartition,
    PartitionAndDetail,
}

/// Public partition subscription descriptor for dependency edges.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartitionSubscription {
    pub partition: PartitionToken,
    #[serde(default)]
    pub detail: Option<String>,
    pub match_mode: PartitionMatchMode,
}

impl PartitionSubscription {
    pub fn whole_partition(partition: impl Into<PartitionToken>) -> Self {
        Self {
            partition: partition.into(),
            detail: None,
            match_mode: PartitionMatchMode::WholePartition,
        }
    }

    pub fn partition_and_detail(
        partition: impl Into<PartitionToken>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            partition: partition.into(),
            detail: Some(detail.into()),
            match_mode: PartitionMatchMode::PartitionAndDetail,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct PartitionTokenId(pub u32);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct DetailTokenId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InternedPartitionSubscription {
    pub partition: PartitionTokenId,
    pub detail: Option<DetailTokenId>,
    pub match_mode: PartitionMatchMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PartitionTokenRef<'a> {
    Public(&'a PartitionToken),
    Interned(PartitionTokenId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DetailTokenRef<'a> {
    Public(&'a str),
    Interned(DetailTokenId),
}

fn is_strict_region_order(regions: &[ChangedRegion]) -> bool {
    regions.windows(2).all(|pair| pair[0] < pair[1])
}

pub(crate) trait PartitionScoped {
    fn partition_token_ref(&self) -> PartitionTokenRef<'_>;
    fn detail_token_ref(&self) -> Option<DetailTokenRef<'_>>;
    fn match_mode(&self) -> PartitionMatchMode;
}

impl PartitionScoped for PartitionSubscription {
    fn partition_token_ref(&self) -> PartitionTokenRef<'_> {
        PartitionTokenRef::Public(&self.partition)
    }

    fn detail_token_ref(&self) -> Option<DetailTokenRef<'_>> {
        self.detail.as_deref().map(DetailTokenRef::Public)
    }

    fn match_mode(&self) -> PartitionMatchMode {
        self.match_mode
    }
}

impl PartitionScoped for InternedPartitionSubscription {
    fn partition_token_ref(&self) -> PartitionTokenRef<'_> {
        PartitionTokenRef::Interned(self.partition)
    }

    fn detail_token_ref(&self) -> Option<DetailTokenRef<'_>> {
        self.detail.map(DetailTokenRef::Interned)
    }

    fn match_mode(&self) -> PartitionMatchMode {
        self.match_mode
    }
}

impl PartitionScoped for ChangedRegion {
    fn partition_token_ref(&self) -> PartitionTokenRef<'_> {
        PartitionTokenRef::Public(&self.partition)
    }

    fn detail_token_ref(&self) -> Option<DetailTokenRef<'_>> {
        self.detail.as_deref().map(DetailTokenRef::Public)
    }

    fn match_mode(&self) -> PartitionMatchMode {
        if self.detail.is_some() {
            PartitionMatchMode::PartitionAndDetail
        } else {
            PartitionMatchMode::WholePartition
        }
    }
}

pub(crate) fn scopes_overlap(left: &impl PartitionScoped, right: &impl PartitionScoped) -> bool {
    if left.partition_token_ref() != right.partition_token_ref() {
        return false;
    }
    match (left.match_mode(), right.match_mode()) {
        (PartitionMatchMode::WholePartition, _) | (_, PartitionMatchMode::WholePartition) => true,
        (PartitionMatchMode::PartitionAndDetail, PartitionMatchMode::PartitionAndDetail) => {
            left.detail_token_ref() == right.detail_token_ref()
        }
    }
}

pub(crate) fn scope_touched_by_artifact_state(
    artifact_state: Option<&crate::data::trace::RuntimeArtifactState>,
    scope: &PartitionSubscription,
) -> bool {
    let Some(artifact_state) = artifact_state else {
        return false;
    };
    if artifact_state.output_change() == OutputChange::Unchanged {
        return false;
    }
    if artifact_state.changed_scopes().is_empty() {
        return true;
    }
    artifact_state
        .changed_scopes()
        .as_slice()
        .iter()
        .any(|changed_scope| scopes_overlap(scope, changed_scope))
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartitionInterner {
    partitions: crate::data::persistent_vector::PersistentVector<String>,
    details: crate::data::persistent_vector::PersistentVector<String>,
    #[serde(default)]
    partition_lookup: crate::data::persistent_ord_map::PersistentOrdMap<String, PartitionTokenId>,
    #[serde(default)]
    detail_lookup: crate::data::persistent_ord_map::PersistentOrdMap<String, DetailTokenId>,
}

impl PartitionInterner {
    pub fn intern_subscription(
        &mut self,
        subscription: &PartitionSubscription,
    ) -> InternedPartitionSubscription {
        InternedPartitionSubscription {
            partition: self.intern_partition(&subscription.partition.0),
            detail: subscription
                .detail
                .as_deref()
                .map(|detail| self.intern_detail(detail)),
            match_mode: subscription.match_mode,
        }
    }

    pub(crate) fn resolve_subscription(
        &self,
        subscription: &PartitionSubscription,
    ) -> Option<InternedPartitionSubscription> {
        Some(InternedPartitionSubscription {
            partition: *self.partition_lookup.get(&subscription.partition.0)?,
            detail: subscription
                .detail
                .as_deref()
                .and_then(|detail| self.detail_lookup.get(detail).copied()),
            match_mode: subscription.match_mode,
        })
    }

    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    pub fn detail_count(&self) -> usize {
        self.details.len()
    }

    pub fn token_count(&self) -> usize {
        self.partition_count() + self.detail_count()
    }

    fn intern_partition(&mut self, partition: &str) -> PartitionTokenId {
        if let Some(id) = self.partition_lookup.get(partition).copied() {
            return id;
        }
        let id = PartitionTokenId(self.partitions.len() as u32);
        self.partitions.push_back(partition.to_owned());
        self.partition_lookup.insert(partition.to_owned(), id);
        id
    }

    fn intern_detail(&mut self, detail: &str) -> DetailTokenId {
        if let Some(id) = self.detail_lookup.get(detail).copied() {
            return id;
        }
        let id = DetailTokenId(self.details.len() as u32);
        self.details.push_back(detail.to_owned());
        self.detail_lookup.insert(detail.to_owned(), id);
        id
    }

    pub(crate) fn operational_clone(&self) -> Self {
        Self {
            partitions: self.partitions.operational_clone(),
            details: self.details.operational_clone(),
            partition_lookup: self.partition_lookup.operational_clone(),
            detail_lookup: self.detail_lookup.operational_clone(),
        }
    }

    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            partitions: self.partitions.fork_persistent(),
            details: self.details.fork_persistent(),
            partition_lookup: self.partition_lookup.fork_persistent(),
            detail_lookup: self.detail_lookup.fork_persistent(),
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            partitions: self.partitions.clone(),
            details: self.details.clone(),
            partition_lookup: self.partition_lookup.fork_storage_identity(),
            detail_lookup: self.detail_lookup.fork_storage_identity(),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        self.partitions.shares_storage_with(&other.partitions)
            && self.details.shares_storage_with(&other.details)
            && self.partition_lookup.ptr_eq(&other.partition_lookup)
            && self.detail_lookup.ptr_eq(&other.detail_lookup)
    }
}

impl ChangedRegion {
    pub fn new(partition: impl Into<PartitionToken>) -> Self {
        Self {
            partition: partition.into(),
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

mod subscription_lookup;
