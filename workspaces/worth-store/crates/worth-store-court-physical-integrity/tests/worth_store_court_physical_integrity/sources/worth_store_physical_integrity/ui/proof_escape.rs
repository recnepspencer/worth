use worth_store_physical_integrity::{
    IntegrityValidatedCheckpointBinding, IntegrityValidatedCheckpointBindingCompaction,
    IntegrityValidatedCheckpointDirtyBasis, IntegrityValidatedCheckpointFooter,
    IntegrityValidatedCheckpointStreamHeader,
    IntegrityValidatedCurrentRootSelector, IntegrityValidatedPreviousRootSelector,
    IntegrityValidatedExtentChunkFrame, IntegrityValidatedExtentManifest,
    IntegrityValidatedPageFrame, IntegrityValidatedPhysicalWorkObligation,
    IntegrityValidatedRootManifest, IntegrityValidatedWalFrame,
};

fn escape_checkpoint_header<'media>(
    validation: IntegrityValidatedCheckpointStreamHeader<'media>,
) -> IntegrityValidatedCheckpointStreamHeader<'static> {
    validation
}

fn escape_checkpoint_dirty<'media>(
    validation: IntegrityValidatedCheckpointDirtyBasis<'media>,
) -> IntegrityValidatedCheckpointDirtyBasis<'static> {
    validation
}

fn escape_checkpoint_compaction<'media>(
    validation: IntegrityValidatedCheckpointBindingCompaction<'media>,
) -> IntegrityValidatedCheckpointBindingCompaction<'static> {
    validation
}

fn escape_checkpoint_binding<'media>(
    validation: IntegrityValidatedCheckpointBinding<'media>,
) -> IntegrityValidatedCheckpointBinding<'static> {
    validation
}

fn escape_checkpoint_footer<'media>(
    validation: IntegrityValidatedCheckpointFooter<'media>,
) -> IntegrityValidatedCheckpointFooter<'static> {
    validation
}

fn escape_current<'media>(
    validation: IntegrityValidatedCurrentRootSelector<'media>,
) -> IntegrityValidatedCurrentRootSelector<'static> {
    validation
}

fn escape_previous<'media>(
    validation: IntegrityValidatedPreviousRootSelector<'media>,
) -> IntegrityValidatedPreviousRootSelector<'static> {
    validation
}

fn escape_manifest<'media>(
    validation: IntegrityValidatedRootManifest<'media>,
) -> IntegrityValidatedRootManifest<'static> {
    validation
}

fn escape_physical_work<'media>(
    validation: IntegrityValidatedPhysicalWorkObligation<'media>,
) -> IntegrityValidatedPhysicalWorkObligation<'static> {
    validation
}

fn escape_page<'media>(
    validation: IntegrityValidatedPageFrame<'media>,
) -> IntegrityValidatedPageFrame<'static> {
    validation
}

fn escape_wal<'media>(
    validation: IntegrityValidatedWalFrame<'media>,
) -> IntegrityValidatedWalFrame<'static> {
    validation
}

fn escape_extent_manifest<'media>(
    validation: IntegrityValidatedExtentManifest<'media>,
) -> IntegrityValidatedExtentManifest<'static> {
    validation
}

fn escape_extent_chunk<'media>(
    validation: IntegrityValidatedExtentChunkFrame<'media>,
) -> IntegrityValidatedExtentChunkFrame<'static> {
    validation
}

fn main() {}
