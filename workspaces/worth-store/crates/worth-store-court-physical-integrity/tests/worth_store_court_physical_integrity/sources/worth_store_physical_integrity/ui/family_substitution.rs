use worth_store_physical_integrity::{
    IntegrityValidatedBootstrapCatalog,
    IntegrityValidatedCheckpointBinding, IntegrityValidatedCheckpointDirtyBasis,
    IntegrityValidatedCurrentRootSelector, IntegrityValidatedPhysicalWorkObligation,
    IntegrityValidatedExtentChunkFrame, IntegrityValidatedExtentManifest,
    IntegrityValidatedPageFrame, IntegrityValidatedPreviousRootSelector,
    IntegrityValidatedRootRoutingBlock,
    IntegrityValidatedSegmentMembershipBlock, IntegrityValidatedWalFrame,
};

fn requires_dirty(_: IntegrityValidatedCheckpointDirtyBasis<'_>) {}

fn substitute_checkpoint(binding: IntegrityValidatedCheckpointBinding<'_>) {
    requires_dirty(binding);
}

fn requires_current(_: IntegrityValidatedCurrentRootSelector<'_>) {}
fn requires_physical_work(_: IntegrityValidatedPhysicalWorkObligation<'_>) {}
fn requires_page(_: IntegrityValidatedPageFrame<'_>) {}
fn requires_extent_manifest(_: IntegrityValidatedExtentManifest<'_>) {}
fn requires_root_routing(_: IntegrityValidatedRootRoutingBlock<'_>) {}

fn substitute(previous: IntegrityValidatedPreviousRootSelector<'_>) {
    requires_current(previous);
}

fn substitute_physical_work(physical_work: IntegrityValidatedPhysicalWorkObligation<'_>) {
    requires_current(physical_work);
}

fn substitute_current(current: IntegrityValidatedCurrentRootSelector<'_>) {
    requires_physical_work(current);
}

fn substitute_page(current: IntegrityValidatedCurrentRootSelector<'_>) {
    requires_page(current);
}

fn substitute_wal(wal: IntegrityValidatedWalFrame<'_>) {
    requires_current(wal);
}

fn substitute_extent(chunk: IntegrityValidatedExtentChunkFrame<'_>) {
    requires_extent_manifest(chunk);
}

fn substitute_segment(segment: IntegrityValidatedSegmentMembershipBlock<'_>) {
    requires_root_routing(segment);
}

fn substitute_bootstrap(bootstrap: IntegrityValidatedBootstrapCatalog<'_>) {
    requires_root_routing(bootstrap);
}

fn main() {}
