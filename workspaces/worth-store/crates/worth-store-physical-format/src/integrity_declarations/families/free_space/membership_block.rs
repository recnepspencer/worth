use super::super::durable_frame_declaration;
use crate::integrity_declarations::{
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration,
};

pub const FREE_SPACE_MEMBERSHIP_BLOCK_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    durable_frame_declaration(PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock);
