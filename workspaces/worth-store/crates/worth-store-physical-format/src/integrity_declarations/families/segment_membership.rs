use super::durable_frame_declaration;
use crate::integrity_declarations::{
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration,
};

pub const SEGMENT_MEMBERSHIP_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    durable_frame_declaration(PhysicalIntegrityArtifactFamily::SegmentMembership);
