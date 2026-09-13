use super::checkpoint_record_declaration;
use crate::integrity_declarations::{
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration,
};

pub const CHECKPOINT_FOOTER_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    checkpoint_record_declaration(PhysicalIntegrityArtifactFamily::CheckpointFooter);
