//! Sealed test doors over this module's private members.
use super::*;

impl ComputationChanges {
    #[doc(hidden)]
    pub fn carry_for_test(
        &self,
        original: &FactPayload,
        prefix: usize,
        fresh: &OrdSet<usize>,
        snapshot: &SnapshotHandle,
        control: StorageControl<'_, '_>,
    ) -> Result<Option<PostconditionCarry>, StoreDenial> {
        Self::carry(self, original, prefix, fresh, snapshot, control)
    }
}

impl PostconditionCarry {
    #[doc(hidden)]
    pub fn matches_publication_source_for_test(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, CompanionPreflightStop> {
        Self::matches_publication_source(self, runtime, snapshot)
    }
}

impl PostconditionCarry {
    #[doc(hidden)]
    pub fn select_for_test(
        &self,
        touches: &RelationalDescriptiveTouchGraph,
        count: usize,
        moved: &[usize],
        control: StorageControl<'_, '_>,
    ) -> Result<Option<SelectedPostconditionRebase>, StoreDenial> {
        Self::select(self, touches, count, moved, control)
    }
}

impl SelectedPostconditionRebase {
    #[doc(hidden)]
    pub fn basis_for_test(&self) -> &super::super::super::settlement::PostingRebaseBasis {
        &self.basis
    }
}

impl SelectedPostconditionRebase {
    #[doc(hidden)]
    pub fn ordinals_for_test(&self) -> &OrdSet<usize> {
        &self.ordinals
    }
}
