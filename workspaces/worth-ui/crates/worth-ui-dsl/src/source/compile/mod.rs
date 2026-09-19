mod authored_mode;
mod authored_source_input;
#[cfg(feature = "certification-support")]
pub mod certification;
mod compile_diagnostic;
mod compiler;
mod overlay_identity_resolution;
mod protocol_identity;
mod rust_overlay_identity_resolution;
mod sealed_overlay_declaration_bindings;
mod sealed_semantic_artifact;
mod sealed_semantic_package;
mod semantic_package_exact_basis;
mod semantic_package_identity;
mod semantic_package_lowering_receipts;

pub use authored_mode::WorthUiAuthoredMode;
pub use authored_source_input::WorthUiAuthoredSourceInput;
pub use compile_diagnostic::{
    WorthUiDslCompileDiagnostic, WorthUiDslCompileDiagnosticCode,
    WorthUiDslCompileDiagnosticDetail, WorthUiDslCompileReport, WorthUiDslCompileStopClass,
    WorthUiDslDiagnosticIdentity, WorthUiDslSourceSpan,
};
pub use compiler::WorthUiDslCompiler;
pub(crate) use overlay_identity_resolution::resolve_file_authored_overlay_declarations;
pub use protocol_identity::WorthUiDslProtocolIdentity;
pub(crate) use rust_overlay_identity_resolution::resolve_rust_authored_overlay_declaration_bindings;
pub use sealed_overlay_declaration_bindings::WorthUiSealedOverlayDeclarationBindings;
pub use sealed_semantic_artifact::WorthUiSealedSemanticArtifact;
pub use sealed_semantic_package::{
    WorthUiSealedSemanticPackage, WorthUiSemanticAppearanceRoleDeclaration,
    WorthUiSemanticBackdropDeclaration, WorthUiSemanticBlock, WorthUiSemanticDeclaration,
    WorthUiSemanticDeclarationView, WorthUiSemanticImport, WorthUiSemanticModule,
    WorthUiSemanticProjectionDeclaration, WorthUiSemanticProvenanceRef, WorthUiSemanticToken,
};
pub use semantic_package_identity::WorthUiSemanticPackageIdentity;
