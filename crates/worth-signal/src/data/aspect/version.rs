mod clone_work;
mod evaluation;
mod evaluation_projection;
mod evaluation_work;
use std::collections::BTreeMap;

mod lookup;
mod path_versions_wire;
mod retained_charge;

use serde::{Deserialize, Serialize};

use super::aspect::{Aspect, MAX_ASPECTS};
use super::mask::AspectMask;
use crate::data::output::{ChangedRegion, PartitionSubscription, ScopeCoverage, ScopePath};

/// Per-aspect version counters carried by each signal node.
///
/// Embedding runtimes assign meaning to aspect slots. `worth-signal` only
/// provides deterministic storage and comparison mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectVersion {
    slots: [u64; MAX_ASPECTS],
}

impl AspectVersion {
    /// Create a new aspect version with all counters at zero.
    pub const fn zero() -> Self {
        Self {
            slots: [0; MAX_ASPECTS],
        }
    }

    /// Create a new aspect version from a full slot array.
    pub const fn from_slots(slots: [u64; MAX_ASPECTS]) -> Self {
        Self { slots }
    }

    /// Create a new aspect version from explicit slot/value pairs.
    pub fn from_updates<const N: usize>(updates: [(Aspect, u64); N]) -> Self {
        let mut version = Self::zero();
        let mut i = 0;
        while i < N {
            let (aspect, value) = updates[i];
            version.slots[aspect.index()] = value;
            i += 1;
        }
        version
    }

    /// Read the version for a specific aspect.
    pub const fn get(self, aspect: Aspect) -> u64 {
        self.slots[aspect.index()]
    }

    /// Return a copy with one aspect set to an explicit value.
    pub fn with(mut self, aspect: Aspect, value: u64) -> Self {
        self.slots[aspect.index()] = value;
        self
    }

    /// Bump one aspect version by one.
    pub fn bump(mut self, aspect: Aspect) -> Self {
        self.slots[aspect.index()] += 1;
        self
    }

    /// Bump all aspects included in the provided mask.
    pub fn bump_mask(mut self, mask: AspectMask) -> Self {
        let mut bits = mask.bits();
        while bits != 0 {
            let index = bits.trailing_zeros() as usize;
            self.slots[index] += 1;
            bits &= bits - 1;
        }
        self
    }

    /// Borrow all aspect slots.
    pub const fn slots(&self) -> &[u64; MAX_ASPECTS] {
        &self.slots
    }

    fn max_slots(mut self, other: Self) -> Self {
        for (slot, candidate) in self.slots.iter_mut().zip(other.slots) {
            *slot = (*slot).max(candidate);
        }
        self
    }
}

impl Default for AspectVersion {
    fn default() -> Self {
        Self::zero()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartitionVersionMap {
    global: AspectVersion,
    overrides: PartitionVersionOverrides,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AspectVersionHeader {
    global: AspectVersion,
    has_partition_overrides: bool,
}

impl Default for AspectVersionHeader {
    fn default() -> Self {
        Self::zero()
    }
}

#[allow(dead_code)]
impl AspectVersionHeader {
    pub const fn zero() -> Self {
        Self {
            global: AspectVersion::zero(),
            has_partition_overrides: false,
        }
    }

    pub const fn global(&self) -> AspectVersion {
        self.global
    }

    pub const fn has_partition_overrides(&self) -> bool {
        self.has_partition_overrides
    }

    pub fn set_global(&mut self, version: AspectVersion) {
        self.global = version;
    }

    pub fn set_has_partition_overrides(&mut self, has_partition_overrides: bool) {
        self.has_partition_overrides = has_partition_overrides;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PartitionVersionOverrides {
    baseline: AspectVersion,
    #[serde(with = "path_versions_wire")]
    paths: BTreeMap<ScopePath, PathVersions>,
}

/// Separate write reach from the aggregate observed by a subtree reader.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct PathVersions {
    exact_write: Option<AspectVersion>,
    subtree_write: Option<AspectVersion>,
    descendant_aggregate: Option<AspectVersion>,
}

impl PartitionVersionOverrides {
    pub fn scoped_or_global(
        &self,
        scope: &PartitionSubscription,
        _global: AspectVersion,
    ) -> AspectVersion {
        let mut result = self.baseline;
        for depth in 1..=scope.path().depth() {
            let prefix = scope.path().prefix(depth).expect("validated scope prefix");
            if let Some(record) = self.paths.get(&prefix) {
                if let Some(write) = record.subtree_write {
                    result = result.max_slots(write);
                }
                if depth == scope.path().depth() {
                    let direct = match scope.coverage() {
                        ScopeCoverage::Exact => record.exact_write,
                        ScopeCoverage::Subtree => record.descendant_aggregate,
                    };
                    if let Some(write) = direct {
                        result = result.max_slots(write);
                    }
                }
            }
        }
        result
    }

    pub fn version_for_scope(
        &self,
        aspect: Aspect,
        scope: Option<&PartitionSubscription>,
        global: AspectVersion,
    ) -> u64 {
        match scope {
            Some(scope) => self.scoped_or_global(scope, global).get(aspect),
            None => global.get(aspect),
        }
    }

    pub fn set_global(&mut self, version: AspectVersion) {
        self.baseline = version;
        self.paths.clear();
    }

    pub fn apply_evaluation(&mut self, version: AspectVersion, changed_regions: &[ChangedRegion]) {
        evaluation::apply(self, version, changed_regions);
    }

    pub fn has_overrides(&self) -> bool {
        !self.paths.is_empty()
    }
}

impl Default for PartitionVersionMap {
    fn default() -> Self {
        Self::zero()
    }
}

#[allow(dead_code)]
impl PartitionVersionMap {
    pub const fn zero() -> Self {
        Self {
            global: AspectVersion::zero(),
            overrides: PartitionVersionOverrides {
                baseline: AspectVersion::zero(),
                paths: BTreeMap::new(),
            },
        }
    }

    pub const fn global(&self) -> AspectVersion {
        self.global
    }

    pub fn scoped(&self, scope: &PartitionSubscription) -> AspectVersion {
        self.overrides.scoped_or_global(scope, self.global)
    }

    pub fn version_for_scope(&self, aspect: Aspect, scope: Option<&PartitionSubscription>) -> u64 {
        match scope {
            Some(scope) => self.scoped(scope).get(aspect),
            None => self.global.get(aspect),
        }
    }

    pub fn set_global(&mut self, version: AspectVersion) {
        self.global = version;
        self.overrides.set_global(version);
    }

    pub fn apply_evaluation(&mut self, version: AspectVersion, changed_regions: &[ChangedRegion]) {
        self.global = version;
        self.overrides.apply_evaluation(version, changed_regions);
    }

    /// Apply one producer-local aspect change without projecting that aspect's
    /// locality onto sibling aspects. Empty regions mean the aspect changed
    /// globally; otherwise only the named partition/detail scopes advance.
    pub(crate) fn apply_scoped_aspect_bump(
        &mut self,
        aspect: Aspect,
        changed_regions: &[ChangedRegion],
        _baseline: &Self,
    ) {
        let previous_global = self.global;
        let next_value = previous_global.get(aspect) + 1;
        self.global = previous_global.with(aspect, next_value);

        if changed_regions.is_empty() {
            self.overrides.baseline = self.overrides.baseline.with(aspect, next_value);
        } else {
            for region in changed_regions {
                self.overrides.write_aspect(region, aspect, next_value);
            }
        }
    }

    pub fn into_storage_parts(self) -> (AspectVersionHeader, PartitionVersionOverrides) {
        let overrides = self.overrides;
        (
            AspectVersionHeader {
                global: self.global,
                has_partition_overrides: overrides.has_overrides(),
            },
            overrides,
        )
    }

    pub fn from_storage_parts(
        header: AspectVersionHeader,
        overrides: PartitionVersionOverrides,
    ) -> Self {
        Self {
            global: header.global(),
            overrides,
        }
    }
}
