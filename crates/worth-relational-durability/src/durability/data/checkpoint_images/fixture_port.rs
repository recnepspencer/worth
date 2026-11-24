//! Access for the existing cross-package corruption and custody fixtures.
use super::*;

#[doc(hidden)]
pub trait DurableCheckpointFieldsFixturePort {
    fn branch_cells_mut_for_test(&mut self) -> &mut Vec<RelationalBranchCellCheckpoint>;
    fn branch_cells_for_test(&self) -> &[RelationalBranchCellCheckpoint];
    fn branch_root_schema_images_mut_for_test(
        &mut self,
    ) -> &mut Vec<super::super::DurableBranchRootSchemaImage>;
    fn branch_root_schema_images_for_test(&self) -> &[super::super::DurableBranchRootSchemaImage];
    fn branch_roots_mut_for_test(&mut self) -> &mut Vec<DurableBranchRootImage>;
    fn branch_roots_for_test(&self) -> &[DurableBranchRootImage];
    fn canonical_directory_mut_for_test(
        &mut self,
    ) -> &mut Option<super::super::NativeCheckpointDirectoryEdge>;
    fn commit_descriptors_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<
        CommitId,
        worth_relational_contracts::internal_port::branch::target::RelationalBranchRootDescriptor,
    >;
    fn derived_index_checkpoint_mut_for_test(
        &mut self,
    ) -> &mut Option<crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts>;
    fn derived_index_checkpoint_format_mut_for_test(&mut self) -> &mut u16;
    fn derived_index_checkpoint_format_for_test(&self) -> u16;
    fn into_selected_directory_for_test(
        self,
    ) -> Option<super::super::NativeCheckpointDirectoryEdge>;
    fn envelopes_mut_for_test(&mut self) -> &mut Vec<PositionedCanonicalCommit>;
    fn record_identity_mut_for_test(&mut self) -> &mut DurableRecordIdentityState;
    fn record_identity_for_test(&self) -> &DurableRecordIdentityState;
    fn retired_branch_names_mut_for_test(
        &mut self,
    ) -> &mut Vec<worth_relational_contracts::internal_port::history::data::coordinates::BranchId>;
    fn symbol_identity_frontier_mut_for_test(&mut self) -> &mut u32;
    fn symbol_identity_frontier_for_test(&self) -> u32;
}

impl DurableCheckpointFieldsFixturePort for DurableCheckpoint {
    fn branch_cells_mut_for_test(&mut self) -> &mut Vec<RelationalBranchCellCheckpoint> {
        &mut self.branch_cells
    }

    fn branch_cells_for_test(&self) -> &[RelationalBranchCellCheckpoint] {
        &self.branch_cells
    }

    fn branch_root_schema_images_mut_for_test(
        &mut self,
    ) -> &mut Vec<super::super::DurableBranchRootSchemaImage> {
        &mut self.branch_root_schema_images
    }

    fn branch_root_schema_images_for_test(&self) -> &[super::super::DurableBranchRootSchemaImage] {
        &self.branch_root_schema_images
    }

    fn branch_roots_mut_for_test(&mut self) -> &mut Vec<DurableBranchRootImage> {
        &mut self.branch_roots
    }

    fn branch_roots_for_test(&self) -> &[DurableBranchRootImage] {
        &self.branch_roots
    }

    fn canonical_directory_mut_for_test(
        &mut self,
    ) -> &mut Option<super::super::NativeCheckpointDirectoryEdge> {
        &mut self.canonical_directory
    }

    fn commit_descriptors_mut_for_test(
        &mut self,
    ) -> &mut BTreeMap<
        CommitId,
        worth_relational_contracts::internal_port::branch::target::RelationalBranchRootDescriptor,
    > {
        &mut self.commit_descriptors
    }

    fn derived_index_checkpoint_mut_for_test(
        &mut self,
    ) -> &mut Option<crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts>
    {
        &mut self.derived_index_checkpoint
    }

    fn derived_index_checkpoint_format_mut_for_test(&mut self) -> &mut u16 {
        &mut self.derived_index_checkpoint_format
    }

    fn derived_index_checkpoint_format_for_test(&self) -> u16 {
        self.derived_index_checkpoint_format
    }

    fn into_selected_directory_for_test(
        self,
    ) -> Option<super::super::NativeCheckpointDirectoryEdge> {
        self.directory
    }

    fn envelopes_mut_for_test(&mut self) -> &mut Vec<PositionedCanonicalCommit> {
        &mut self.envelopes
    }

    fn record_identity_mut_for_test(&mut self) -> &mut DurableRecordIdentityState {
        &mut self.record_identity
    }

    fn record_identity_for_test(&self) -> &DurableRecordIdentityState {
        &self.record_identity
    }

    fn retired_branch_names_mut_for_test(
        &mut self,
    ) -> &mut Vec<worth_relational_contracts::internal_port::history::data::coordinates::BranchId>
    {
        &mut self.retired_branch_names
    }

    fn symbol_identity_frontier_mut_for_test(&mut self) -> &mut u32 {
        &mut self.symbol_identity_frontier
    }

    fn symbol_identity_frontier_for_test(&self) -> u32 {
        self.symbol_identity_frontier
    }
}
