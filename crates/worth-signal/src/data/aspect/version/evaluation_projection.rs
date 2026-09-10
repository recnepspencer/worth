//! Borrowed projection of the existing scoped evaluation write algorithm.
use super::{
    Aspect, AspectVersion, ChangedRegion, PartitionSubscription, PartitionVersionOverrides,
};
use crate::data::error::SignalError;
use crate::data::output::PartitionMatchMode;
use crate::logic::evaluation::EvaluationWork;
impl PartitionVersionOverrides {
    pub(crate) fn version_after_evaluation(
        &self,
        aspect: Aspect,
        scope: Option<&PartitionSubscription>,
        version: AspectVersion,
        regions: &[ChangedRegion],
        work: &mut EvaluationWork<'_>,
    ) -> Result<u64, SignalError> {
        let Some(scope) = scope else {
            work.reserve(Some(1))?;
            return Ok(version.get(aspect));
        };
        work.reserve(self.lookup_work_bound(Some(scope)))?;
        let bytes = scope
            .partition
            .0
            .len()
            .checked_add(scope.detail.as_ref().map_or(0, String::len));
        work.reserve(
            bytes
                .and_then(|n| n.checked_mul(2))
                .and_then(|n| n.checked_add(16))
                .and_then(|n| n.checked_mul(regions.len())),
        )?;
        let mut partition_written = false;
        let mut whole_written = false;
        let mut detail_written = false;
        for region in regions {
            if region.partition != scope.partition {
                continue;
            }
            partition_written = true;
            whole_written |= region.detail.is_none();
            detail_written |= scope.detail.is_some()
                && region.detail == scope.detail
                && scope.match_mode == PartitionMatchMode::PartitionAndDetail;
        }
        if scope.detail.is_some() {
            if detail_written {
                return Ok(version.get(aspect));
            }
            if let Some(previous) = self.details.get(scope) {
                return Ok(if whole_written { version } else { *previous }.get(aspect));
            }
        }
        Ok(if partition_written {
            version
        } else {
            self.partitions
                .get(&scope.partition)
                .copied()
                .unwrap_or(version)
        }
        .get(aspect))
    }
}
#[cfg(test)]
mod tests;
