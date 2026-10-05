//! Borrowed projection of the existing scoped evaluation write algorithm.
use super::{
    Aspect, AspectVersion, ChangedRegion, PartitionSubscription, PartitionVersionOverrides,
};
use crate::data::error::SignalError;
use crate::data::output::scopes_overlap;
use crate::logic::evaluation::EvaluationWork;
impl PartitionVersionOverrides {
    pub(crate) fn version_after_evaluation(
        &self,
        aspect: Aspect,
        scope: Option<&PartitionSubscription>,
        version: AspectVersion,
        regions: &[ChangedRegion],
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<u64, SignalError> {
        let Some(scope) = scope else {
            work.reserve(Some(1))?;
            return Ok(version.get(aspect));
        };
        if regions.is_empty() {
            work.reserve(Some(1))?;
            return Ok(version.get(aspect));
        }
        work.reserve(self.lookup_work_bound(Some(scope)))?;
        let bytes = Some(scope.path().total_segment_bytes());
        work.reserve(
            bytes
                .and_then(|n| n.checked_mul(2))
                .and_then(|n| n.checked_add(16))
                .and_then(|n| n.checked_mul(regions.len())),
        )?;
        let previous = self.scoped_or_global(scope, version).get(aspect);
        let written = regions.iter().any(|region| scopes_overlap(scope, region));
        Ok(if written {
            previous.max(version.get(aspect))
        } else {
            previous
        })
    }
}
#[cfg(test)]
mod tests;
