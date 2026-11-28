use super::*;

impl<PatchState, SnapshotState, SignalState, BranchHeadState, MappingState>
    RuntimeBridgeBuilder<PatchState, SnapshotState, SignalState, BranchHeadState, MappingState>
{
    pub fn register_aspect_mapping(mut self, registration: BridgeAspectRegistration) -> Self {
        self.aspect_registrations.push(registration);
        self
    }

    pub fn register_source(mut self, declaration: SourceDeclaration) -> Self {
        self.source_declarations.push(declaration);
        self
    }

    pub fn register_structural(mut self, declaration: StructuralIdentityDeclaration) -> Self {
        self.structural_declarations.push(declaration);
        self
    }

    pub fn register_semantic_correspondence(
        mut self,
        registration: BridgeSemanticCorrespondenceRegistration,
    ) -> Self {
        self.semantic_dependency_registrations.push(registration);
        self
    }
}

impl<PatchState, SnapshotState, SignalState, BranchHeadState>
    RuntimeBridgeBuilder<
        PatchState,
        SnapshotState,
        SignalState,
        BranchHeadState,
        MissingMappingRegistrations,
    >
{
    pub fn register_mapping(
        self,
        registration: BridgeMappingRegistration,
    ) -> RuntimeBridgeBuilder<
        PatchState,
        SnapshotState,
        SignalState,
        BranchHeadState,
        PresentMappingRegistrations,
    > {
        RuntimeBridgeBuilder {
            policy: self.policy,
            committed_patch_source: self.committed_patch_source,
            snapshot_read_source: self.snapshot_read_source,
            signal_sink: self.signal_sink,
            truth_branch_head_source: self.truth_branch_head_source,
            continuity_lineage_source: self.continuity_lineage_source,
            writeback_authority: self.writeback_authority,
            snapshot_reader_pool: self.snapshot_reader_pool,
            source_adapter_registrations: self.source_adapter_registrations,
            source_declarations: self.source_declarations,
            structural_declarations: self.structural_declarations,
            mapping_registrations: PresentMappingRegistrations(vec![registration]),
            aspect_registrations: self.aspect_registrations,
            semantic_dependency_registrations: self.semantic_dependency_registrations,
        }
    }
}

impl<PatchState, SnapshotState, SignalState, BranchHeadState>
    RuntimeBridgeBuilder<
        PatchState,
        SnapshotState,
        SignalState,
        BranchHeadState,
        PresentMappingRegistrations,
    >
{
    pub fn register_mapping(mut self, registration: BridgeMappingRegistration) -> Self {
        self.mapping_registrations.0.push(registration);
        self
    }
}
