use super::durable_frame_declaration;
use crate::integrity_declarations::{
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration,
};

pub const EXTENT_CHUNK_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    durable_frame_declaration(PhysicalIntegrityArtifactFamily::ExtentChunk);
