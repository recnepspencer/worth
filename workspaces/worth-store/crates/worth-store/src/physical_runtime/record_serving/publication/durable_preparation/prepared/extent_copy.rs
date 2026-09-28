use super::{PreparedPhysicalMutation, PreparedPhysicalMutationData, PreparedRewriteAnchor};
use crate::physical_runtime::record_serving::CompletedExtentCopy;
use worth_store_physical_format::DurableExtentRecordPlacement;

impl PreparedPhysicalMutation {
    pub(in crate::physical_runtime::record_serving) fn mark_extent_copy(
        mut self,
        source_root: u64,
        source: DurableExtentRecordPlacement,
    ) -> Self {
        self.selected_segment_rewrite = true;
        self.source_root_generation = source_root;
        self.rewrite_anchor = Some(PreparedRewriteAnchor::ExtentCopy(source));
        self
    }

    pub(in crate::physical_runtime::record_serving) fn extent_copy_source(
        &self,
    ) -> Option<DurableExtentRecordPlacement> {
        match self.rewrite_anchor {
            Some(PreparedRewriteAnchor::ExtentCopy(source)) => Some(source),
            _ => match &self.data {
                PreparedPhysicalMutationData::Planned { data, .. } => {
                    data.source_copy().map(|copy| copy.intent().source())
                }
                _ => None,
            },
        }
    }

    pub(in crate::physical_runtime::record_serving) fn attach_completed_extent_copy(
        mut self,
        copy: CompletedExtentCopy,
    ) -> Self {
        assert_eq!(self.extent_copy_source(), Some(copy.intent().source()));
        assert_eq!(
            self.idempotency_identity().bytes(),
            copy.intent().operation()
        );
        let PreparedPhysicalMutationData::Unplanned { copy: slot, .. } = &mut self.data else {
            unreachable!("completed copy attaches before final publication planning")
        };
        assert!(slot.is_none());
        *slot = Some(copy);
        self
    }

    pub(in crate::physical_runtime::record_serving) fn take_completed_extent_copy(
        &mut self,
    ) -> Option<CompletedExtentCopy> {
        match &mut self.data {
            PreparedPhysicalMutationData::Unplanned { copy, .. } => copy.take(),
            PreparedPhysicalMutationData::Planned { .. } => None,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn rebase_extent_copy_root(
        &mut self,
        generation: u64,
    ) {
        assert!(self.extent_copy_source().is_some());
        self.source_root_generation = generation;
    }

    /// Prepared authority means no final publication WAL escaped. Its root
    /// projection can be refreshed without rebasing the original copy intent.
    pub(in crate::physical_runtime::record_serving) fn refresh_planned_extent_copy(
        &mut self,
        current_root: worth_store_physical_format::DurablePhysicalRootManifest,
    ) -> Result<bool, ()> {
        let PreparedPhysicalMutationData::Planned { data, root, .. } = &mut self.data else {
            return Ok(false);
        };
        let Some(copy) = data.source_copy() else {
            return Ok(false);
        };
        copy.require_live_source()?;
        self.source_root_generation = current_root.generation();
        root.source_root = current_root;
        Ok(true)
    }
}
