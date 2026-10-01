mod interning;
mod interning_batch;
mod reserved_fork;
pub(crate) use interning_batch::PreparedPartitionInternerExpansion;
mod fork_growth;
mod retained_charge;
mod subscription_lookup;

use serde::{Deserialize, Serialize};

use super::{OutputChange, ScopeCoverage, ScopePath, ScopePathError};

/// One opaque segment in a Signal scope path.
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChangedRegion {
    path: ScopePath,
    coverage: ScopeCoverage,
}

impl ChangedRegion {
    pub fn try_new(partition: impl Into<PartitionToken>) -> Result<Self, ScopePathError> {
        ScopePath::one(partition.into().0).map(Self::subtree)
    }
    pub fn new(partition: impl Into<PartitionToken>) -> Self {
        Self::try_new(partition).expect("valid scope segment")
    }

    pub fn subtree(path: ScopePath) -> Self {
        Self {
            path,
            coverage: ScopeCoverage::Subtree,
        }
    }
    pub fn exact(path: ScopePath) -> Self {
        Self {
            path,
            coverage: ScopeCoverage::Exact,
        }
    }

    pub fn with_detail(self, detail: impl Into<String>) -> Self {
        self.try_with_segment(detail).expect("valid scope segment")
    }

    pub fn try_with_segment(self, segment: impl Into<String>) -> Result<Self, ScopePathError> {
        self.path.with_segment(segment).map(Self::exact)
    }

    pub fn path(&self) -> &ScopePath {
        &self.path
    }
    pub fn coverage(&self) -> ScopeCoverage {
        self.coverage
    }
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
        debug_assert!(is_strict_region_order(&regions));
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

fn is_strict_region_order(regions: &[ChangedRegion]) -> bool {
    regions.windows(2).all(|pair| pair[0] < pair[1])
}

/// A dependency's declared observation of one path and coverage.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartitionSubscription {
    path: ScopePath,
    coverage: ScopeCoverage,
}

impl PartitionSubscription {
    pub fn subtree(path: ScopePath) -> Self {
        Self {
            path,
            coverage: ScopeCoverage::Subtree,
        }
    }
    pub fn exact(path: ScopePath) -> Self {
        Self {
            path,
            coverage: ScopeCoverage::Exact,
        }
    }
    pub fn try_whole_partition(
        partition: impl Into<PartitionToken>,
    ) -> Result<Self, ScopePathError> {
        ScopePath::one(partition.into().0).map(Self::subtree)
    }
    pub fn whole_partition(partition: impl Into<PartitionToken>) -> Self {
        Self::try_whole_partition(partition).expect("valid scope segment")
    }
    pub fn partition_and_detail(
        partition: impl Into<PartitionToken>,
        detail: impl Into<String>,
    ) -> Self {
        let path =
            ScopePath::new([partition.into().0, detail.into()]).expect("valid two-segment scope");
        Self::exact(path)
    }
    pub fn path(&self) -> &ScopePath {
        &self.path
    }
    pub fn coverage(&self) -> ScopeCoverage {
        self.coverage
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct PartitionTokenId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "InternedScopePathRepr", into = "InternedScopePathRepr")]
pub struct InternedScopePath {
    segments: [PartitionTokenId; ScopePath::MAX_DEPTH],
    depth: u8,
}

#[derive(Serialize, Deserialize)]
struct InternedScopePathRepr {
    segments: [PartitionTokenId; ScopePath::MAX_DEPTH],
    depth: u8,
}

impl TryFrom<InternedScopePathRepr> for InternedScopePath {
    type Error = ScopePathError;

    fn try_from(value: InternedScopePathRepr) -> Result<Self, Self::Error> {
        let depth = usize::from(value.depth);
        if depth == 0 {
            return Err(ScopePathError::Empty);
        }
        if depth > ScopePath::MAX_DEPTH {
            return Err(ScopePathError::TooDeep);
        }
        if value.segments[depth..]
            .iter()
            .any(|token| *token != PartitionTokenId(0))
        {
            return Err(ScopePathError::InvalidPadding);
        }
        Self::new(&value.segments[..depth])
    }
}

impl From<InternedScopePath> for InternedScopePathRepr {
    fn from(value: InternedScopePath) -> Self {
        Self {
            segments: value.segments,
            depth: value.depth,
        }
    }
}

impl InternedScopePath {
    pub fn new(segments: &[PartitionTokenId]) -> Result<Self, ScopePathError> {
        if segments.is_empty() {
            return Err(ScopePathError::Empty);
        }
        if segments.len() > ScopePath::MAX_DEPTH {
            return Err(ScopePathError::TooDeep);
        }
        let mut result = Self {
            segments: [PartitionTokenId(0); ScopePath::MAX_DEPTH],
            depth: segments.len() as u8,
        };
        result.segments[..segments.len()].copy_from_slice(segments);
        Ok(result)
    }
    pub fn segments(&self) -> &[PartitionTokenId] {
        &self.segments[..usize::from(self.depth)]
    }
    pub fn depth(&self) -> usize {
        usize::from(self.depth)
    }
    pub fn covers(&self, descendant: &Self) -> bool {
        descendant.segments().starts_with(self.segments())
    }
    pub fn prefix(&self, depth: usize) -> Option<Self> {
        (1..=self.depth())
            .contains(&depth)
            .then(|| Self::new(&self.segments()[..depth]).expect("valid prefix"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InternedPartitionSubscription {
    path: InternedScopePath,
    coverage: ScopeCoverage,
}

impl InternedPartitionSubscription {
    pub fn new(path: InternedScopePath, coverage: ScopeCoverage) -> Self {
        Self { path, coverage }
    }
    pub fn path(&self) -> InternedScopePath {
        self.path
    }
    pub fn coverage(&self) -> ScopeCoverage {
        self.coverage
    }
}

pub(crate) enum ScopedPathRef<'a> {
    Public(&'a ScopePath),
    Interned(InternedScopePath),
}

impl ScopedPathRef<'_> {
    fn covers(&self, descendant: &Self) -> bool {
        match (self, descendant) {
            (Self::Public(left), Self::Public(right)) => left.covers(right),
            (Self::Interned(left), Self::Interned(right)) => left.covers(right),
            _ => false,
        }
    }
}

pub(crate) trait PartitionScoped {
    fn scoped_path(&self) -> ScopedPathRef<'_>;
    fn coverage(&self) -> ScopeCoverage;
}

impl PartitionScoped for PartitionSubscription {
    fn scoped_path(&self) -> ScopedPathRef<'_> {
        ScopedPathRef::Public(&self.path)
    }
    fn coverage(&self) -> ScopeCoverage {
        self.coverage
    }
}
impl PartitionScoped for ChangedRegion {
    fn scoped_path(&self) -> ScopedPathRef<'_> {
        ScopedPathRef::Public(&self.path)
    }
    fn coverage(&self) -> ScopeCoverage {
        self.coverage
    }
}
impl PartitionScoped for InternedPartitionSubscription {
    fn scoped_path(&self) -> ScopedPathRef<'_> {
        ScopedPathRef::Interned(self.path)
    }
    fn coverage(&self) -> ScopeCoverage {
        self.coverage
    }
}

pub(crate) fn scopes_overlap<L: PartitionScoped, R: PartitionScoped>(left: &L, right: &R) -> bool {
    let left_path = left.scoped_path();
    let right_path = right.scoped_path();
    match (left.coverage(), right.coverage()) {
        (ScopeCoverage::Exact, ScopeCoverage::Exact) => {
            left_path.covers(&right_path) && right_path.covers(&left_path)
        }
        (ScopeCoverage::Subtree, ScopeCoverage::Exact) => left_path.covers(&right_path),
        (ScopeCoverage::Exact, ScopeCoverage::Subtree) => right_path.covers(&left_path),
        (ScopeCoverage::Subtree, ScopeCoverage::Subtree) => {
            left_path.covers(&right_path) || right_path.covers(&left_path)
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
        .any(|changed_scope| {
            super::scope_path::scopes_overlap(
                scope.path(),
                scope.coverage(),
                changed_scope.path(),
                changed_scope.coverage(),
            )
        })
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartitionInterner {
    segments: crate::data::persistent_vector::PersistentVector<String>,
    #[serde(default)]
    segment_lookup: crate::data::persistent_ord_map::PersistentOrdMap<String, PartitionTokenId>,
}
