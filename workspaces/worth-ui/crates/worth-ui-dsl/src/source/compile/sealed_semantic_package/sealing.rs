use super::{
    WorthUiSemanticBlock, WorthUiSemanticDeclaration, WorthUiSemanticImport,
    WorthUiSemanticProjectionDeclaration, WorthUiSemanticProvenanceRef, WorthUiSemanticToken,
};
use crate::source::{
    WorthUiArtifactInputNode, WorthUiArtifactInputProvenance, WorthUiDslCompileDiagnostic,
    WorthUiDslCompileDiagnosticCode, WorthUiDslCompileStopClass, WorthUiDslSourceSpan,
    WorthUiProjectionShape, WorthUiStructuralBodyParser, WorthUiStructuralLanguageDiagnosticCode,
    WorthUiStructuralParseFailure,
};
use crate::WorthUiSealedSemanticArtifact;

pub(super) fn seal_declaration(
    declaration: &WorthUiArtifactInputNode,
    provenance_ref: WorthUiSemanticProvenanceRef,
) -> Result<WorthUiSemanticDeclaration, WorthUiDslCompileDiagnostic> {
    match declaration {
        WorthUiArtifactInputNode::Import(import) => {
            Ok(WorthUiSemanticDeclaration::Import(WorthUiSemanticImport {
                target: import.target().clone(),
                provenance_ref,
            }))
        }
        WorthUiArtifactInputNode::Component(block) => {
            seal_block(block, provenance_ref).map(WorthUiSemanticDeclaration::Component)
        }
        WorthUiArtifactInputNode::Surface(block) => {
            seal_block(block, provenance_ref).map(WorthUiSemanticDeclaration::Surface)
        }
        WorthUiArtifactInputNode::Binding(block) => {
            seal_block(block, provenance_ref).map(WorthUiSemanticDeclaration::Binding)
        }
        WorthUiArtifactInputNode::QueryScalar(block) => {
            seal_projection(block, WorthUiProjectionShape::Scalar, provenance_ref)
        }
        WorthUiArtifactInputNode::QueryCollection(block) => {
            seal_projection(block, WorthUiProjectionShape::Collection, provenance_ref)
        }
        WorthUiArtifactInputNode::Token(token) => {
            Ok(WorthUiSemanticDeclaration::Token(WorthUiSemanticToken {
                name_text: token.name_text().to_owned(),
                authored_identity: token.authored_identity().map(str::to_owned),
                value_text: token.value_text().to_owned(),
                provenance_ref,
            }))
        }
        WorthUiArtifactInputNode::SemanticArtifact(node) => {
            Ok(WorthUiSemanticDeclaration::SemanticArtifact(
                WorthUiSealedSemanticArtifact::new(node.declaration().clone(), provenance_ref),
            ))
        }
        WorthUiArtifactInputNode::AppearanceRole(node) => {
            Ok(WorthUiSemanticDeclaration::AppearanceRole(
                super::WorthUiSemanticAppearanceRoleDeclaration::new(
                    node.role().clone(),
                    provenance_ref,
                ),
            ))
        }
        WorthUiArtifactInputNode::Backdrop(node) => Ok(WorthUiSemanticDeclaration::Backdrop(
            super::WorthUiSemanticBackdropDeclaration::new(
                node.declaration().clone(),
                provenance_ref,
            ),
        )),
    }
}

pub(super) fn input_node_provenance(
    declaration: &WorthUiArtifactInputNode,
) -> &WorthUiArtifactInputProvenance {
    match declaration {
        WorthUiArtifactInputNode::Import(declaration) => declaration.provenance(),
        WorthUiArtifactInputNode::Component(declaration)
        | WorthUiArtifactInputNode::Surface(declaration)
        | WorthUiArtifactInputNode::Binding(declaration)
        | WorthUiArtifactInputNode::QueryScalar(declaration)
        | WorthUiArtifactInputNode::QueryCollection(declaration) => declaration.provenance(),
        WorthUiArtifactInputNode::Token(declaration) => declaration.provenance(),
        WorthUiArtifactInputNode::SemanticArtifact(declaration) => declaration.provenance(),
        WorthUiArtifactInputNode::AppearanceRole(declaration) => declaration.provenance(),
        WorthUiArtifactInputNode::Backdrop(declaration) => declaration.provenance(),
    }
}

pub(super) fn duplicate_projection_diagnostic(
    identity: &str,
    provenance: &WorthUiArtifactInputProvenance,
) -> WorthUiDslCompileDiagnostic {
    let (module_id, span) = diagnostic_location(provenance);
    WorthUiDslCompileDiagnostic::new(
        WorthUiDslCompileDiagnosticCode::InvalidProjectionDeclaration,
        WorthUiDslCompileStopClass::LanguageLegality,
        format!("projection declaration `{identity}` appears more than once"),
        Some(module_id),
        span,
    )
}

pub(super) fn unknown_projection_content_diagnostic(
    identity: &str,
    provenance: &WorthUiArtifactInputProvenance,
) -> WorthUiDslCompileDiagnostic {
    let (module_id, span) = diagnostic_location(provenance);
    WorthUiDslCompileDiagnostic::new(
        WorthUiDslCompileDiagnosticCode::UnknownProjectionContent,
        WorthUiDslCompileStopClass::LanguageLegality,
        format!("content references unknown projection declaration `{identity}`"),
        Some(module_id),
        span,
    )
}

fn seal_projection(
    block: &crate::source::WorthUiArtifactInputBlockNode,
    shape: WorthUiProjectionShape,
    provenance_ref: WorthUiSemanticProvenanceRef,
) -> Result<WorthUiSemanticDeclaration, WorthUiDslCompileDiagnostic> {
    crate::source::parse_projection_requirement(block.name_text(), shape, block.body_atoms())
        .map(|requirement| {
            WorthUiSemanticDeclaration::Projection(WorthUiSemanticProjectionDeclaration {
                requirement,
                provenance_ref,
            })
        })
        .map_err(|error| projection_diagnostic(error, block.provenance()))
}

fn seal_block(
    block: &crate::source::WorthUiArtifactInputBlockNode,
    provenance_ref: WorthUiSemanticProvenanceRef,
) -> Result<WorthUiSemanticBlock, WorthUiDslCompileDiagnostic> {
    let structure = WorthUiStructuralBodyParser::parse(block.body_atoms())
        .map_err(|failure| structural_diagnostic(failure, block.provenance()))?;
    Ok(WorthUiSemanticBlock {
        name_text: block.name_text().to_owned(),
        authored_identity: block.authored_identity().map(str::to_owned),
        structure,
        appearance_role_attachment: block.appearance_role_attachment().cloned(),
        provenance_ref,
    })
}

pub(super) fn appearance_diagnostic(
    code: WorthUiDslCompileDiagnosticCode,
    message: impl Into<String>,
    provenance: &WorthUiArtifactInputProvenance,
) -> WorthUiDslCompileDiagnostic {
    let (module_id, span) = diagnostic_location(provenance);
    WorthUiDslCompileDiagnostic::new(
        code,
        WorthUiDslCompileStopClass::SemanticNormalization,
        message,
        Some(module_id),
        span,
    )
}

pub(super) fn overlay_diagnostic(
    denial: crate::UiOverlayRelationAdmissionDenial,
    provenance: &WorthUiArtifactInputProvenance,
) -> WorthUiDslCompileDiagnostic {
    let code = match denial {
        crate::UiOverlayRelationAdmissionDenial::BackdropCapacityExceeded => {
            WorthUiDslCompileDiagnosticCode::OverlayCapacityDenied
        }
        crate::UiOverlayRelationAdmissionDenial::MissingAnchor => {
            WorthUiDslCompileDiagnosticCode::MissingOverlayAnchor
        }
        crate::UiOverlayRelationAdmissionDenial::ForeignSurfaceAnchor => {
            WorthUiDslCompileDiagnosticCode::ForeignOverlaySurface
        }
        crate::UiOverlayRelationAdmissionDenial::Cycle => {
            WorthUiDslCompileDiagnosticCode::CyclicOverlayRelation
        }
        crate::UiOverlayRelationAdmissionDenial::AmbiguousOrder => {
            WorthUiDslCompileDiagnosticCode::AmbiguousOverlayRelation
        }
        crate::UiOverlayRelationAdmissionDenial::DuplicateParticipant
        | crate::UiOverlayRelationAdmissionDenial::SelfRelation
        | crate::UiOverlayRelationAdmissionDenial::ConflictingImmediateAdjacency => {
            WorthUiDslCompileDiagnosticCode::InvalidBackdropDeclaration
        }
    };
    appearance_diagnostic(
        code,
        format!("overlay relation admission denied: {denial:?}"),
        provenance,
    )
}

fn projection_diagnostic(
    failure: crate::source::WorthUiProjectionDeclarationError,
    provenance: &WorthUiArtifactInputProvenance,
) -> WorthUiDslCompileDiagnostic {
    let (module_id, span) = diagnostic_location(provenance);
    WorthUiDslCompileDiagnostic::new(
        WorthUiDslCompileDiagnosticCode::InvalidProjectionDeclaration,
        WorthUiDslCompileStopClass::LanguageLegality,
        failure.detail(),
        Some(module_id),
        span,
    )
}

fn structural_diagnostic(
    failure: WorthUiStructuralParseFailure,
    provenance: &WorthUiArtifactInputProvenance,
) -> WorthUiDslCompileDiagnostic {
    let code = match failure.code {
        WorthUiStructuralLanguageDiagnosticCode::InvalidStructuralSyntax => {
            WorthUiDslCompileDiagnosticCode::InvalidStructuralSyntax
        }
        WorthUiStructuralLanguageDiagnosticCode::DuplicateRegionSizingDeclaration => {
            WorthUiDslCompileDiagnosticCode::DuplicateRegionSizingDeclaration
        }
        WorthUiStructuralLanguageDiagnosticCode::DuplicateRegionStateDeclaration => {
            WorthUiDslCompileDiagnosticCode::DuplicateRegionStateDeclaration
        }
        WorthUiStructuralLanguageDiagnosticCode::DuplicateMountPlacementDeclaration => {
            WorthUiDslCompileDiagnosticCode::DuplicateMountPlacementDeclaration
        }
        WorthUiStructuralLanguageDiagnosticCode::DuplicateMountStateDeclaration => {
            WorthUiDslCompileDiagnosticCode::DuplicateMountStateDeclaration
        }
        WorthUiStructuralLanguageDiagnosticCode::IllegalRootStructuralStatement => {
            WorthUiDslCompileDiagnosticCode::IllegalRootStructuralStatement
        }
    };
    let (module_id, span) = diagnostic_location(provenance);
    WorthUiDslCompileDiagnostic::new(
        code,
        WorthUiDslCompileStopClass::LanguageLegality,
        format!("{} at {}", failure.authored_text, failure.structural_locus),
        Some(module_id),
        span,
    )
}

fn diagnostic_location(
    provenance: &WorthUiArtifactInputProvenance,
) -> (String, Option<WorthUiDslSourceSpan>) {
    match provenance {
        WorthUiArtifactInputProvenance::ParsedSourceDeclaration {
            declaration_span, ..
        } => (
            declaration_span.module_id().as_str().to_owned(),
            Some(WorthUiDslSourceSpan::new(
                declaration_span.module_id().as_str(),
                declaration_span.start_byte(),
                declaration_span.end_byte(),
            )),
        ),
        WorthUiArtifactInputProvenance::RustAuthoredDeclaration {
            authored_module_path,
            ..
        } => (authored_module_path.clone(), None),
    }
}
