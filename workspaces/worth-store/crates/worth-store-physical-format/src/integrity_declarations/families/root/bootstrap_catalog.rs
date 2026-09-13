use super::super::durable_frame_declaration;
use crate::integrity_declarations::{
    PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration,
};

pub const BOOTSTRAP_CATALOG_INTEGRITY_DECLARATION: PhysicalIntegrityFormatDeclaration =
    durable_frame_declaration(PhysicalIntegrityArtifactFamily::BootstrapCatalog);
