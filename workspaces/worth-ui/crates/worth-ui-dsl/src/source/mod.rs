mod artifact_input;
mod canonical;
mod compile;
mod import_graph;
mod legality;
mod lexical;
mod lower;
mod module;
mod package;
mod parse;
mod projection;
#[cfg(test)]
mod tests;

#[cfg(feature = "certification-support")]
pub use compile::certification;

#[cfg(test)]
pub(crate) use artifact_input::WorthUiArtifactInputEquivalentShape;
pub(crate) use artifact_input::WorthUiArtifactInputNormalizer;
pub use artifact_input::{
    WorthUiArtifactInput, WorthUiArtifactInputAppearanceRoleNode, WorthUiArtifactInputBackdropNode,
    WorthUiArtifactInputBlockNode, WorthUiArtifactInputBodyAtom, WorthUiArtifactInputImportNode,
    WorthUiArtifactInputModule, WorthUiArtifactInputNode, WorthUiArtifactInputNodeKind,
    WorthUiArtifactInputProvenance, WorthUiArtifactInputReference,
    WorthUiArtifactInputSemanticArtifactNode, WorthUiArtifactInputTokenNode,
    WorthUiSemanticArtifactDeclaration,
};
pub(crate) use canonical::WorthUiCanonicalModuleOrder;
pub(crate) use compile::{
    resolve_file_authored_overlay_declarations, resolve_rust_authored_overlay_declaration_bindings,
};
pub use compile::{
    WorthUiAuthoredMode, WorthUiAuthoredSourceInput, WorthUiDslCompileDiagnostic,
    WorthUiDslCompileDiagnosticCode, WorthUiDslCompileDiagnosticDetail, WorthUiDslCompileReport,
    WorthUiDslCompileStopClass, WorthUiDslCompiler, WorthUiDslDiagnosticIdentity,
    WorthUiDslProtocolIdentity, WorthUiDslSourceSpan, WorthUiSealedOverlayDeclarationBindings,
    WorthUiSealedSemanticArtifact, WorthUiSealedSemanticPackage,
    WorthUiSemanticAppearanceRoleDeclaration, WorthUiSemanticBackdropDeclaration,
    WorthUiSemanticBlock, WorthUiSemanticDeclaration, WorthUiSemanticDeclarationView,
    WorthUiSemanticImport, WorthUiSemanticModule, WorthUiSemanticPackageIdentity,
    WorthUiSemanticProjectionDeclaration, WorthUiSemanticProvenanceRef, WorthUiSemanticToken,
};
pub(crate) use import_graph::{WorthUiSourceImport, WorthUiSourceImportGraph};
pub use legality::{
    WorthUiAuthoredMount, WorthUiAuthoredProjectionContent, WorthUiAuthoredRegion,
    WorthUiAuthoredStructuralBody,
};
pub(crate) use legality::{
    WorthUiStructuralBodyParser, WorthUiStructuralLanguageDiagnosticCode,
    WorthUiStructuralParseFailure,
};
pub(crate) use lexical::{tokenize_module_source, WorthUiSourceToken, WorthUiSourceTokenKind};
pub use lower::rust_authored::{
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
};
pub(crate) use lower::{
    WorthUiFileAuthoredLoweredDeclaration, WorthUiParsedSourceToArtifactInputLowerer,
    WorthUiRustAuthoredInputLoweringDenial, WorthUiRustAuthoredToArtifactInputLowerer,
};
pub use module::WorthUiSourceModuleId;
pub(crate) use module::WorthUiSourceModuleRecord;
pub(crate) use package::{
    WorthUiSourcePackage, WorthUiSourcePackageDiagnostic, WorthUiSourcePackageDiagnosticCode,
    WorthUiSourcePackageDigest, WorthUiSourcePackageLoader, WorthUiSourcePackageReport,
};
pub use parse::WorthUiSourceSpan;
pub(crate) use parse::{
    WorthUiParseDiagnostic, WorthUiParseDiagnosticCode, WorthUiParseReport,
    WorthUiParsedAppearanceRoleDeclaration, WorthUiParsedBlockBody, WorthUiParsedBlockDeclaration,
    WorthUiParsedImportDeclaration, WorthUiParsedSourceDeclaration, WorthUiParsedSourceModule,
    WorthUiParsedSourcePackage, WorthUiParsedTokenDeclaration, WorthUiSourceParser,
};
pub(crate) use projection::parse_projection_requirement;
pub use projection::{
    WorthUiProjectionCollectionPolicy, WorthUiProjectionCollectionSelection,
    WorthUiProjectionDeclarationError, WorthUiProjectionDeclarationErrorKind,
    WorthUiProjectionLifecycle, WorthUiProjectionNativeFamily, WorthUiProjectionRequirement,
    WorthUiProjectionRequirementIdentity, WorthUiProjectionShape,
};
