mod binding;
mod binding_compaction;
mod dirty_basis;
mod footer;
mod stream_header;

pub use binding::CHECKPOINT_BINDING_INTEGRITY_DECLARATION;
pub use binding_compaction::CHECKPOINT_BINDING_COMPACTION_INTEGRITY_DECLARATION;
pub use dirty_basis::CHECKPOINT_DIRTY_BASIS_INTEGRITY_DECLARATION;
pub use footer::CHECKPOINT_FOOTER_INTEGRITY_DECLARATION;
pub use stream_header::CHECKPOINT_STREAM_HEADER_INTEGRITY_DECLARATION;

use crate::integrity_declarations::{
    coverage::CHECKPOINT_RECORD_V1_CHECKSUMS, PhysicalIntegrityArtifactFamily,
    PhysicalIntegrityFormatDeclaration, PhysicalIntegrityFormatVersion,
};

const fn checkpoint_record_declaration(
    family: PhysicalIntegrityArtifactFamily,
) -> PhysicalIntegrityFormatDeclaration {
    PhysicalIntegrityFormatDeclaration::new(
        family,
        PhysicalIntegrityFormatVersion::new(1, None),
        CHECKPOINT_RECORD_V1_CHECKSUMS,
    )
}
