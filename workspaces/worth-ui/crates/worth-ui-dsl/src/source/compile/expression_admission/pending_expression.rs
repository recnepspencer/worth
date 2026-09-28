use super::super::WorthUiSemanticProvenanceRef;
use crate::semantic::WorthUiExpressionDeclaration;
use crate::source::{WorthUiArtifactInputProvenance, WorthUiSourceModuleId};

/// A parsed expression declaration waiting for package-level admission.
///
/// Admission needs every projection and every other expression, so a
/// declaration is held until the whole package has been read. `insert_at` is
/// its position among the module declarations, so the sealed expression lands
/// where the author wrote it.
pub(in super::super) struct PendingExpression {
    pub(super) declaration: WorthUiExpressionDeclaration,
    pub(super) provenance: WorthUiArtifactInputProvenance,
    pub(super) provenance_ref: WorthUiSemanticProvenanceRef,
    pub(super) module_id: WorthUiSourceModuleId,
    pub(super) insert_at: usize,
}
