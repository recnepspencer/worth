mod projection;
mod revision;

use crate::source::{WorthUiArtifactInputBodyAtom, WorthUiSemanticArtifactDeclaration};
use crate::{WorthUiIntentDeclarationSpec, WorthUiIntentInteractionRoute};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorthUiRustAuthoredArtifactInputModule {
    relative_module_path: String,
    declarations: Vec<WorthUiRustAuthoredDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorthUiRustAuthoredDeclaration {
    Import {
        target_module_path: String,
    },
    Component {
        name_text: String,
        authored_identity: Option<String>,
        body_atoms: Vec<WorthUiArtifactInputBodyAtom>,
        appearance_role_attachment: Option<crate::UiAppearanceRoleAttachmentDeclaration>,
    },
    Surface {
        name_text: String,
        authored_identity: Option<String>,
        body_atoms: Vec<WorthUiArtifactInputBodyAtom>,
    },
    Binding {
        name_text: String,
        authored_identity: Option<String>,
        body_atoms: Vec<WorthUiArtifactInputBodyAtom>,
    },
    QueryScalar {
        name_text: String,
        body_atoms: Vec<WorthUiArtifactInputBodyAtom>,
    },
    QueryCollection {
        name_text: String,
        body_atoms: Vec<WorthUiArtifactInputBodyAtom>,
    },
    Token {
        name_text: String,
        authored_identity: Option<String>,
        value_text: String,
    },
    SemanticArtifact(WorthUiSemanticArtifactDeclaration),
    AppearanceRole(crate::UiAppearanceRoleDeclaration),
    Backdrop(crate::UiStaticBackdropDeclaration),
}

impl WorthUiRustAuthoredArtifactInputModule {
    pub fn new(relative_module_path: impl Into<String>) -> Self {
        Self {
            relative_module_path: relative_module_path.into(),
            declarations: Vec::new(),
        }
    }

    pub fn with_import(mut self, target_module_path: impl Into<String>) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Import {
                target_module_path: target_module_path.into(),
            });
        self
    }

    pub fn with_component(mut self, name_text: impl Into<String>) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Component {
                name_text: name_text.into(),
                authored_identity: None,
                body_atoms: Vec::new(),
                appearance_role_attachment: None,
            });
        self
    }

    pub fn with_component_authored_identity(
        mut self,
        name_text: impl Into<String>,
        authored_identity: impl Into<String>,
    ) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Component {
                name_text: name_text.into(),
                authored_identity: Some(authored_identity.into()),
                body_atoms: Vec::new(),
                appearance_role_attachment: None,
            });
        self
    }

    pub fn with_component_body_atoms(
        mut self,
        name_text: impl Into<String>,
        body_atoms: impl IntoIterator<Item = WorthUiArtifactInputBodyAtom>,
    ) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Component {
                name_text: name_text.into(),
                authored_identity: None,
                body_atoms: body_atoms.into_iter().collect(),
                appearance_role_attachment: None,
            });
        self
    }

    pub fn with_component_body_atoms_and_authored_identity(
        mut self,
        name_text: impl Into<String>,
        authored_identity: impl Into<String>,
        body_atoms: impl IntoIterator<Item = WorthUiArtifactInputBodyAtom>,
    ) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Component {
                name_text: name_text.into(),
                authored_identity: Some(authored_identity.into()),
                body_atoms: body_atoms.into_iter().collect(),
                appearance_role_attachment: None,
            });
        self
    }

    pub fn with_control_routes(
        mut self,
        name_text: impl Into<String>,
        routes: impl IntoIterator<Item = WorthUiIntentInteractionRoute>,
    ) -> Self {
        let body_atoms = routes
            .into_iter()
            .flat_map(|route| route.body_atoms())
            .collect();
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Component {
                name_text: name_text.into(),
                authored_identity: None,
                body_atoms,
                appearance_role_attachment: None,
            });
        self
    }

    pub fn with_control_routes_and_authored_identity(
        mut self,
        name_text: impl Into<String>,
        authored_identity: impl Into<String>,
        routes: impl IntoIterator<Item = WorthUiIntentInteractionRoute>,
    ) -> Self {
        let body_atoms = routes
            .into_iter()
            .flat_map(|route| route.body_atoms())
            .collect();
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Component {
                name_text: name_text.into(),
                authored_identity: Some(authored_identity.into()),
                body_atoms,
                appearance_role_attachment: None,
            });
        self
    }

    pub fn with_intent_declaration(self, declaration: WorthUiIntentDeclarationSpec) -> Self {
        self.with_semantic_declaration(declaration.into_semantic_declaration())
    }

    pub fn with_surface(mut self, name_text: impl Into<String>) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Surface {
                name_text: name_text.into(),
                authored_identity: None,
                body_atoms: Vec::new(),
            });
        self
    }

    pub fn with_surface_body_atoms(
        mut self,
        name_text: impl Into<String>,
        body_atoms: impl IntoIterator<Item = WorthUiArtifactInputBodyAtom>,
    ) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Surface {
                name_text: name_text.into(),
                authored_identity: None,
                body_atoms: body_atoms.into_iter().collect(),
            });
        self
    }

    pub fn with_binding(mut self, name_text: impl Into<String>) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Binding {
                name_text: name_text.into(),
                authored_identity: None,
                body_atoms: Vec::new(),
            });
        self
    }

    pub fn with_binding_authored_identity(
        mut self,
        name_text: impl Into<String>,
        authored_identity: impl Into<String>,
    ) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Binding {
                name_text: name_text.into(),
                authored_identity: Some(authored_identity.into()),
                body_atoms: Vec::new(),
            });
        self
    }

    pub fn with_binding_body_atoms(
        mut self,
        name_text: impl Into<String>,
        body_atoms: impl IntoIterator<Item = WorthUiArtifactInputBodyAtom>,
    ) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Binding {
                name_text: name_text.into(),
                authored_identity: None,
                body_atoms: body_atoms.into_iter().collect(),
            });
        self
    }

    pub fn with_token(
        mut self,
        name_text: impl Into<String>,
        value_text: impl Into<String>,
    ) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Token {
                name_text: name_text.into(),
                authored_identity: None,
                value_text: value_text.into(),
            });
        self
    }

    pub fn with_surface_authored_identity(
        mut self,
        name_text: impl Into<String>,
        authored_identity: impl Into<String>,
    ) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Surface {
                name_text: name_text.into(),
                authored_identity: Some(authored_identity.into()),
                body_atoms: Vec::new(),
            });
        self
    }

    pub fn with_token_authored_identity(
        mut self,
        name_text: impl Into<String>,
        authored_identity: impl Into<String>,
        value_text: impl Into<String>,
    ) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Token {
                name_text: name_text.into(),
                authored_identity: Some(authored_identity.into()),
                value_text: value_text.into(),
            });
        self
    }

    pub fn with_semantic_declaration(
        mut self,
        declaration: WorthUiSemanticArtifactDeclaration,
    ) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::SemanticArtifact(
                declaration,
            ));
        self
    }

    pub fn with_appearance_role(mut self, role: crate::UiAppearanceRoleDeclaration) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::AppearanceRole(role));
        self
    }

    pub fn with_backdrop(mut self, declaration: crate::UiStaticBackdropDeclaration) -> Self {
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Backdrop(declaration));
        self
    }

    pub fn with_component_appearance_role(
        mut self,
        component_name: impl Into<String>,
        attachment: crate::UiAppearanceRoleAttachmentDeclaration,
    ) -> Result<Self, crate::UiAppearanceRoleAttachmentDeclarationDenial> {
        let component_name = component_name.into();
        let matching_components = self
            .declarations
            .iter()
            .filter(|declaration| {
                matches!(
                    declaration,
                    WorthUiRustAuthoredDeclaration::Component { name_text, .. }
                        if name_text == &component_name
                )
            })
            .count();
        if matching_components > 1 {
            return Err(crate::UiAppearanceRoleAttachmentDeclarationDenial::DuplicateAttachment);
        }
        if matching_components == 1 {
            let declaration = self
                .declarations
                .iter_mut()
                .find(|declaration| {
                    matches!(
                        declaration,
                        WorthUiRustAuthoredDeclaration::Component { name_text, .. }
                            if name_text == &component_name
                    )
                })
                .expect("the single matching component must be present");
            if let WorthUiRustAuthoredDeclaration::Component {
                appearance_role_attachment,
                ..
            } = declaration
            {
                if appearance_role_attachment.is_some() {
                    return Err(
                        crate::UiAppearanceRoleAttachmentDeclarationDenial::DuplicateAttachment,
                    );
                }
                *appearance_role_attachment = Some(attachment);
            }
            return Ok(self);
        }
        self.declarations
            .push(WorthUiRustAuthoredDeclaration::Component {
                name_text: component_name,
                authored_identity: None,
                body_atoms: Vec::new(),
                appearance_role_attachment: Some(attachment),
            });
        Ok(self)
    }

    pub(crate) fn relative_module_path(&self) -> &str {
        &self.relative_module_path
    }

    pub(crate) fn declarations(&self) -> &[WorthUiRustAuthoredDeclaration] {
        &self.declarations
    }

    pub(crate) fn source_revision_digest(&self) -> u64 {
        revision::module_digest(&self.relative_module_path, &self.declarations)
    }
}
