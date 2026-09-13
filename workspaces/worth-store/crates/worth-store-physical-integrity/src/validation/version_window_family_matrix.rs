use worth_store_physical_format::integrity_declarations::families::{
    checkpoint::{
        CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION,
        CHECKPOINT_BINDING_INTEGRITY_DECLARATION, CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
        CHECKPOINT_FOOTER_INTEGRITY_DECLARATION, CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
    },
    free_space::{
        FREE_SPACE_HEADER_INTEGRITY_DECLARATION, FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
    },
    root::{
        BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION, CURRENT_SELECTOR_INTEGRITY_DECLARATION,
        PREVIOUS_SELECTOR_INTEGRITY_DECLARATION, ROOT_MANIFEST_INTEGRITY_DECLARATION,
        ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION,
    },
    EXTENT_CHUNK_INTEGRITY_DECLARATION, EXTENT_MANIFEST_INTEGRITY_DECLARATION,
    PAGE_FRAME_INTEGRITY_DECLARATION, PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION,
    SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION, WAL_FRAME_INTEGRITY_DECLARATION,
};
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityFormatDeclaration;

use super::{artifact_version_axis, PhysicalIntegrityVersionAxis};

pub(super) fn assert_current_family_version_matrix() {
    use PhysicalIntegrityVersionAxis as Axis;

    let declarations: [(PhysicalIntegrityFormatDeclaration, Axis, u32, Option<u32>); 18] = [
        (
            PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION,
            Axis::PhysicalWorkObligation,
            6,
            None,
        ),
        (
            PAGE_FRAME_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
        (
            EXTENT_CHUNK_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
        (WAL_FRAME_INTEGRITY_DECLARATION, Axis::WalFrame, 1, None),
        (
            CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION,
            Axis::CheckpointRecordSchema,
            1,
            None,
        ),
        (
            CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION,
            Axis::CheckpointRecordSchema,
            1,
            None,
        ),
        (
            CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION,
            Axis::CheckpointRecordSchema,
            1,
            None,
        ),
        (
            CHECKPOINT_BINDING_INTEGRITY_DECLARATION,
            Axis::CheckpointRecordSchema,
            1,
            None,
        ),
        (
            CHECKPOINT_FOOTER_INTEGRITY_DECLARATION,
            Axis::CheckpointRecordSchema,
            1,
            None,
        ),
        (
            BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
        (
            CURRENT_SELECTOR_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
        (
            PREVIOUS_SELECTOR_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
        (
            ROOT_MANIFEST_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
        (
            ROOT_ROUTING_BLOCK_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
        (
            SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
        (
            EXTENT_MANIFEST_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
        (
            FREE_SPACE_HEADER_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
        (
            FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION,
            Axis::PhysicalFormat,
            1,
            Some(2),
        ),
    ];

    for (declaration, expected_axis, expected_version, expected_envelope) in declarations {
        assert_eq!(artifact_version_axis(declaration.family()), expected_axis);
        assert_eq!(
            u32::from(declaration.version().format_version()),
            expected_version
        );
        assert_eq!(
            declaration.version().envelope_schema().map(u32::from),
            expected_envelope
        );
    }
}
