use worth_store_physical_integrity::{
    IntegrityValidatedCheckpointBinding, IntegrityValidatedCheckpointBindingCompaction,
    IntegrityValidatedCheckpointDirtyBasis, IntegrityValidatedCheckpointFooter,
    IntegrityValidatedCheckpointStreamHeader, IntegrityValidatedCurrentRootSelector,
    IntegrityValidatedExtentChunkFrame, IntegrityValidatedExtentManifest,
    IntegrityValidatedPageFrame, IntegrityValidatedPhysicalWorkObligation,
    IntegrityValidatedPreviousRootSelector, IntegrityValidatedRootManifest,
    IntegrityValidatedWalFrame, PhysicalArtifactScope,
};

fn forge_checkpoint_views(scope: PhysicalArtifactScope) {
    let _header = IntegrityValidatedCheckpointStreamHeader { scope };
    let _dirty = IntegrityValidatedCheckpointDirtyBasis { scope };
    let _compaction = IntegrityValidatedCheckpointBindingCompaction { scope };
    let _binding = IntegrityValidatedCheckpointBinding { scope };
    let _footer = IntegrityValidatedCheckpointFooter { scope };
}

fn forge_current(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedCurrentRootSelector {
        scope,
        inspected: supplied(),
    };
}

fn forge_previous(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedPreviousRootSelector {
        scope,
        inspected: supplied(),
    };
}

fn forge_manifest(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedRootManifest {
        scope,
        inspected: supplied(),
    };
}

fn forge_physical_work(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedPhysicalWorkObligation {
        scope,
        inspected: supplied(),
    };
}

fn forge_extent_manifest(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedExtentManifest {
        scope,
        inspected: supplied(),
    };
}

fn forge_page(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedPageFrame {
        scope,
        inspected: supplied(),
    };
}

fn forge_wal(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedWalFrame {
        scope,
        inspected: supplied(),
    };
}

fn forge_extent_chunk(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedExtentChunkFrame {
        scope,
        inspected: supplied(),
    };
}

fn main() {}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
