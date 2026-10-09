use super::durable_frame_declaration;
use crate::integrity_declarations::{
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration,
};

/// Arena slots retain the canonical manifest/chunk checksum domains; zero padding
/// is validated separately and cannot alter the admitted inner frame length.
pub const EXTENT_ARENA_FRAME_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    durable_frame_declaration(PhysicalIntegrityArtifactFamily::ExtentArenaFrame);
