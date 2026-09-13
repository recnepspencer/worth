pub mod checkpoint;
mod extent_chunk;
mod extent_manifest;
pub mod free_space;
mod namespace_identity;
mod page_frame;
mod physical_work_obligation;
pub mod root;
mod segment_membership;
mod wal;

pub use extent_chunk::EXTENT_CHUNK_INTEGRITY_DECLARATION;
pub use extent_manifest::EXTENT_MANIFEST_INTEGRITY_DECLARATION;
pub use namespace_identity::NAMESPACE_IDENTITY_INTEGRITY_DECLARATION;
pub use page_frame::PAGE_FRAME_INTEGRITY_DECLARATION;
pub use physical_work_obligation::{
    PHYSICAL_WORK_OBLIGATION_INTEGRITY_DECLARATION, PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES,
    PHYSICAL_WORK_OBLIGATION_V6_VERSION,
};
pub use segment_membership::SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION;
pub use wal::{
    WAL_FRAME_INTEGRITY_DECLARATION, WAL_FRAME_V1_FOOTER_BYTES, WAL_FRAME_V1_HEADER_BYTES,
    WAL_FRAME_V1_VERSION,
};

use super::{
    coverage::DURABLE_FRAME_V2_CHECKSUMS, PhysicalIntegrityArtifactFamily,
    PhysicalIntegrityFormatDeclaration, PhysicalIntegrityFormatVersion,
};

const fn durable_frame_declaration(
    family: PhysicalIntegrityArtifactFamily,
) -> PhysicalIntegrityFormatDeclaration {
    PhysicalIntegrityFormatDeclaration::new(
        family,
        PhysicalIntegrityFormatVersion::new(1, Some(2)),
        DURABLE_FRAME_V2_CHECKSUMS,
    )
}
