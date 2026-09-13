//! Owned cache derived from a prepared cause replacement, before publication.
use crate::data::aspect::{Aspect, AspectMask};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;
use crate::logic::evaluation::EvaluationWork;
use smallvec::SmallVec;
type Scopes =
    SmallVec<[(Aspect, PartitionSubscription); crate::data::core_profile::HOT_VEC_INLINE_CAPACITY]>;

#[derive(Debug)]
pub(crate) struct PreparedInvalidationCache {
    dirty: AspectMask,
    scoped: AspectMask,
    scopes: Scopes,
}
impl PreparedInvalidationCache {
    pub(in crate::data::graph) fn install_payload(
        self,
        hot: &mut crate::data::node::NodeHotData,
        warm: &mut crate::data::node::NodeWarmData,
    ) {
        warm.dirty_partition_scope_payload = self.scopes;
        hot.dirty_aspects = self.dirty;
        hot.dirty_partition_scope_aspects = self.scoped;
    }

    pub(crate) fn from_causes(
        causes: &[ResolvedDependencyCause],
        work: &mut EvaluationWork<'_>,
    ) -> Result<Self, SignalError> {
        work.reserve(Some(causes.len()))?;
        let count = causes.iter().try_fold(0usize, |n, c| {
            n.checked_add(c.changed_scopes.as_slice().len())
        });
        work.reserve(
            count
                .and_then(|n| {
                    n.checked_mul(std::mem::size_of::<(Aspect, PartitionSubscription)>() + 1)
                })
                .filter(|n| *n <= isize::MAX as usize),
        )?;
        let count = count.expect("admitted scope count");
        let mut largest = 0usize;
        for cause in causes {
            for scope in cause.changed_scopes.as_slice() {
                let bytes = scope
                    .partition
                    .0
                    .len()
                    .checked_add(scope.detail.as_ref().map_or(0, String::len));
                work.reserve(bytes.and_then(|n| n.checked_add(1)))?;
                largest = largest.max(bytes.expect("admitted scope bytes"));
            }
        }
        let levels = usize::BITS as usize - count.leading_zeros() as usize + 1;
        work.reserve(
            largest
                .checked_mul(2)
                .and_then(|n| n.checked_add(32))
                .and_then(|n| n.checked_mul(count))
                .and_then(|n| n.checked_mul(levels))
                .and_then(|n| n.checked_mul(8)),
        )?;
        let mut scopes = Scopes::with_capacity(count);
        let mut dirty = AspectMask::EMPTY;
        for cause in causes {
            dirty.insert(cause.key.aspect);
            scopes.extend(
                cause
                    .changed_scopes
                    .as_slice()
                    .iter()
                    .cloned()
                    .map(|s| (cause.key.aspect, s)),
            );
        }
        // In-place heapsort gives the admitted comparison bound independently
        // of library sort implementation and allocates no scratch payload.
        for root in (0..scopes.len() / 2).rev() {
            sift(&mut scopes, root, count);
        }
        for end in (1..scopes.len()).rev() {
            scopes.swap(0, end);
            sift(&mut scopes, 0, end);
        }
        scopes.dedup();
        let mut scoped = AspectMask::EMPTY;
        for (aspect, _) in &scopes {
            scoped.insert(*aspect);
        }
        Ok(Self {
            dirty,
            scoped,
            scopes,
        })
    }
}
fn sift(scopes: &mut [(Aspect, PartitionSubscription)], mut root: usize, end: usize) {
    while let Some(left) = root
        .checked_mul(2)
        .and_then(|n| n.checked_add(1))
        .filter(|n| *n < end)
    {
        let child = if left + 1 < end && scopes[left] < scopes[left + 1] {
            left + 1
        } else {
            left
        };
        if scopes[root] >= scopes[child] {
            break;
        }
        scopes.swap(root, child);
        root = child;
    }
}
impl SignalGraph {
    pub(crate) fn install_prepared_invalidation_cache(
        &mut self,
        node: NodeId,
        cache: PreparedInvalidationCache,
    ) -> Result<(), SignalError> {
        self.validate_handle(node)?;
        let index = node.index() as usize;
        let hot = self.arena.hot[index]
            .as_mut()
            .expect("validated live node retains hot storage");
        cache.install_payload(hot, &mut self.arena.warm[index]);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
