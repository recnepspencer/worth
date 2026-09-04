#[path = "appearance_validation.rs"]
mod appearance_validation;
#[path = "sealed_semantic_package/authored_route_validation.rs"]
mod authored_route_validation;
#[path = "sealed_semantic_accessors.rs"]
mod sealed_semantic_accessors;
#[path = "sealed_semantic_appearance.rs"]
mod sealed_semantic_appearance;
mod sealing;
pub use sealed_semantic_appearance::{
    WorthUiSemanticAppearanceRoleDeclaration, WorthUiSemanticBackdropDeclaration,
};
use std::collections::{BTreeMap, BTreeSet};

use super::{
    WorthUiAuthoredMode, WorthUiDslProtocolIdentity, WorthUiSealedSemanticArtifact,
    WorthUiSemanticPackageIdentity,
};
use crate::source::{
    WorthUiArtifactInput, WorthUiArtifactInputModule, WorthUiArtifactInputNode,
    WorthUiArtifactInputProvenance, WorthUiArtifactInputReference, WorthUiAuthoredStructuralBody,
    WorthUiDslCompileDiagnostic, WorthUiDslCompileReport, WorthUiProjectionRequirement,
    WorthUiSealedOverlayDeclarationBindings, WorthUiSourceModuleId,
};
use crate::UiDslLoweringReceipt;
#[derive(Debug)]
pub struct WorthUiSealedSemanticPackage {
    modules: BTreeMap<WorthUiSourceModuleId, WorthUiSemanticModule>,
    canonical_module_order: Vec<WorthUiSourceModuleId>,
    provenance_table: Box<[WorthUiArtifactInputProvenance]>,
    identity: WorthUiSemanticPackageIdentity,
    protocol: WorthUiDslProtocolIdentity,
    authored_mode: WorthUiAuthoredMode,
    overlay_declaration_bindings: WorthUiSealedOverlayDeclarationBindings,
    overlay_relation_graph: Option<crate::UiOverlayRelationGraph>,
    _seal: WorthUiSemanticPackageSeal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticModule {
    module_id: WorthUiSourceModuleId,
    declarations: Vec<WorthUiSemanticDeclaration>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiSemanticDeclaration {
    Import(WorthUiSemanticImport),
    Component(WorthUiSemanticBlock),
    Surface(WorthUiSemanticBlock),
    Binding(WorthUiSemanticBlock),
    Projection(WorthUiSemanticProjectionDeclaration),
    Token(WorthUiSemanticToken),
    SemanticArtifact(WorthUiSealedSemanticArtifact),
    AppearanceRole(WorthUiSemanticAppearanceRoleDeclaration),
    Backdrop(WorthUiSemanticBackdropDeclaration),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticImport {
    target: WorthUiArtifactInputReference,
    provenance_ref: WorthUiSemanticProvenanceRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticBlock {
    name_text: String,
    authored_identity: Option<String>,
    structure: WorthUiAuthoredStructuralBody,
    appearance_role_attachment: Option<crate::UiAppearanceRoleAttachmentDeclaration>,
    provenance_ref: WorthUiSemanticProvenanceRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticToken {
    name_text: String,
    authored_identity: Option<String>,
    value_text: String,
    provenance_ref: WorthUiSemanticProvenanceRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticProjectionDeclaration {
    requirement: WorthUiProjectionRequirement,
    provenance_ref: WorthUiSemanticProvenanceRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorthUiSemanticPackageSeal;

/// Compact, package-local reference to diagnostic provenance.
///
/// Callers cannot construct or resolve this reference independently from the
/// package that minted it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticProvenanceRef(usize);

pub struct WorthUiSemanticDeclarationView<'package> {
    declaration: &'package WorthUiSemanticDeclaration,
    provenance_ref: WorthUiSemanticProvenanceRef,
    provenance: &'package WorthUiArtifactInputProvenance,
}

struct WorthUiSemanticPackageSealingState {
    modules: BTreeMap<WorthUiSourceModuleId, WorthUiSemanticModule>,
    provenance_table: Vec<WorthUiArtifactInputProvenance>,
    diagnostics: Vec<WorthUiDslCompileDiagnostic>,
    projection_identities: BTreeSet<String>,
    projection_content_references: Vec<(String, WorthUiArtifactInputProvenance)>,
    appearance_roles: BTreeMap<
        String,
        (
            crate::UiAppearanceRoleDeclaration,
            WorthUiArtifactInputProvenance,
        ),
    >,
    backdrops: Vec<(crate::UiBackdropDeclaration, WorthUiArtifactInputProvenance)>,
    overlay_declaration_bindings: WorthUiSealedOverlayDeclarationBindings,
    overlay_relation_graph: Option<crate::UiOverlayRelationGraph>,
}

impl WorthUiSealedSemanticPackage {
    pub(super) fn seal(
        semantic_input: WorthUiArtifactInput,
        authored_mode: WorthUiAuthoredMode,
    ) -> Result<Self, WorthUiDslCompileReport> {
        let canonical_module_order = semantic_input.module_ids().to_vec();
        let overlay_declaration_bindings = semantic_input.overlay_declaration_bindings().clone();
        let mut state = WorthUiSemanticPackageSealingState::new(overlay_declaration_bindings);
        for module_id in &canonical_module_order {
            let input_module = semantic_input
                .module(module_id)
                .expect("normalized semantic input should contain every canonical module");
            state.seal_module(module_id, input_module);
        }
        state.validate_projection_content_references();
        authored_route_validation::validate(&mut state);
        state.validate_appearance_declarations();
        if !state.diagnostics.is_empty() {
            return Err(WorthUiDslCompileReport::new(state.diagnostics));
        }
        let identity = WorthUiSemanticPackageIdentity::from_modules(
            canonical_module_order.iter().map(|module_id| {
                (
                    module_id,
                    state
                        .modules
                        .get(module_id)
                        .expect("sealed package contains every canonical module"),
                )
            }),
        );
        Ok(Self {
            modules: state.modules,
            canonical_module_order,
            provenance_table: state.provenance_table.into_boxed_slice(),
            identity,
            protocol: WorthUiDslProtocolIdentity::current(),
            authored_mode,
            overlay_declaration_bindings: state.overlay_declaration_bindings,
            overlay_relation_graph: state.overlay_relation_graph,
            _seal: WorthUiSemanticPackageSeal,
        })
    }

    pub fn module_ids(&self) -> &[WorthUiSourceModuleId] {
        &self.canonical_module_order
    }

    pub fn module(&self, module_id: &WorthUiSourceModuleId) -> Option<&WorthUiSemanticModule> {
        self.modules.get(module_id)
    }

    pub fn declaration_views(
        &self,
        module_id: &WorthUiSourceModuleId,
    ) -> Option<impl ExactSizeIterator<Item = WorthUiSemanticDeclarationView<'_>>> {
        self.modules.get(module_id).map(|module| {
            module.declarations.iter().map(|declaration| {
                let provenance_ref = declaration.provenance_ref();
                WorthUiSemanticDeclarationView {
                    declaration,
                    provenance_ref,
                    provenance: &self.provenance_table[provenance_ref.0],
                }
            })
        })
    }

    pub fn identity(&self) -> &WorthUiSemanticPackageIdentity {
        &self.identity
    }

    pub fn protocol(&self) -> WorthUiDslProtocolIdentity {
        self.protocol
    }

    pub fn authored_mode(&self) -> WorthUiAuthoredMode {
        self.authored_mode
    }

    /// Affine compiler evidence for runtime declaration admission.
    ///
    /// The receipts contain only declarations sealed into this package.
    /// Runtime-owned bootstrap declarations are deliberately excluded.
    pub fn declaration_lowering_receipts(&self) -> Vec<UiDslLoweringReceipt> {
        super::semantic_package_lowering_receipts::lower(self)
    }

    #[cfg(feature = "certification-support")]
    pub(super) fn with_protocol_for_certification(
        mut self,
        protocol: WorthUiDslProtocolIdentity,
    ) -> Self {
        self.protocol = protocol;
        self
    }
}

impl WorthUiSemanticPackageSealingState {
    fn new(overlay_declaration_bindings: WorthUiSealedOverlayDeclarationBindings) -> Self {
        Self {
            modules: BTreeMap::new(),
            provenance_table: Vec::new(),
            diagnostics: Vec::new(),
            projection_identities: BTreeSet::new(),
            projection_content_references: Vec::new(),
            appearance_roles: BTreeMap::new(),
            backdrops: Vec::new(),
            overlay_declaration_bindings,
            overlay_relation_graph: None,
        }
    }

    fn seal_module(
        &mut self,
        module_id: &WorthUiSourceModuleId,
        input_module: &WorthUiArtifactInputModule,
    ) {
        let mut declarations = Vec::new();
        for input_declaration in input_module.nodes() {
            self.seal_input_declaration(input_declaration, &mut declarations);
        }
        self.modules.insert(
            module_id.clone(),
            WorthUiSemanticModule {
                module_id: module_id.clone(),
                declarations,
            },
        );
    }

    fn seal_input_declaration(
        &mut self,
        input: &WorthUiArtifactInputNode,
        declarations: &mut Vec<WorthUiSemanticDeclaration>,
    ) {
        let provenance_ref = WorthUiSemanticProvenanceRef(self.provenance_table.len());
        self.provenance_table
            .push(sealing::input_node_provenance(input).clone());
        match sealing::seal_declaration(input, provenance_ref) {
            Ok(WorthUiSemanticDeclaration::Projection(projection))
                if !self
                    .projection_identities
                    .insert(projection.requirement().declaration_identity().to_owned()) =>
            {
                self.diagnostics
                    .push(sealing::duplicate_projection_diagnostic(
                        projection.requirement().declaration_identity(),
                        sealing::input_node_provenance(input),
                    ));
            }
            Ok(declaration) => {
                self.collect_appearance_declaration(&declaration, input);
                if let WorthUiSemanticDeclaration::Component(component) = &declaration {
                    self.projection_content_references.extend(
                        component
                            .structure()
                            .projection_contents()
                            .iter()
                            .map(|content| {
                                (
                                    content.projection_identity_text().to_owned(),
                                    sealing::input_node_provenance(input).clone(),
                                )
                            }),
                    );
                }
                declarations.push(declaration);
            }
            Err(diagnostic) => self.diagnostics.push(diagnostic),
        }
    }

    fn validate_projection_content_references(&mut self) {
        for (identity, provenance) in &self.projection_content_references {
            if !self.projection_identities.contains(identity) {
                self.diagnostics
                    .push(sealing::unknown_projection_content_diagnostic(
                        identity, provenance,
                    ));
            }
        }
    }
}

impl WorthUiSemanticModule {
    pub fn module_id(&self) -> &WorthUiSourceModuleId {
        &self.module_id
    }

    pub fn declarations(&self) -> &[WorthUiSemanticDeclaration] {
        &self.declarations
    }
}

impl WorthUiSemanticImport {
    pub fn target(&self) -> &WorthUiArtifactInputReference {
        &self.target
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSemanticBlock {
    pub fn name_text(&self) -> &str {
        &self.name_text
    }

    pub fn authored_identity(&self) -> Option<&str> {
        self.authored_identity.as_deref()
    }

    pub fn structure(&self) -> &WorthUiAuthoredStructuralBody {
        &self.structure
    }

    pub fn appearance_role_attachment(
        &self,
    ) -> Option<&crate::UiAppearanceRoleAttachmentDeclaration> {
        self.appearance_role_attachment.as_ref()
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSemanticToken {
    pub fn name_text(&self) -> &str {
        &self.name_text
    }

    pub fn authored_identity(&self) -> Option<&str> {
        self.authored_identity.as_deref()
    }

    pub fn value_text(&self) -> &str {
        &self.value_text
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSemanticProjectionDeclaration {
    pub fn requirement(&self) -> &WorthUiProjectionRequirement {
        &self.requirement
    }

    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        self.provenance_ref
    }
}

impl WorthUiSemanticDeclaration {
    pub fn provenance_ref(&self) -> WorthUiSemanticProvenanceRef {
        match self {
            Self::Import(declaration) => declaration.provenance_ref(),
            Self::Component(declaration)
            | Self::Surface(declaration)
            | Self::Binding(declaration) => declaration.provenance_ref(),
            Self::Projection(declaration) => declaration.provenance_ref(),
            Self::Token(declaration) => declaration.provenance_ref(),
            Self::SemanticArtifact(declaration) => declaration.provenance_ref(),
            Self::AppearanceRole(declaration) => declaration.provenance_ref(),
            Self::Backdrop(declaration) => declaration.provenance_ref(),
        }
    }
}
