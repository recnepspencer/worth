use worth_ui_dsl::{
    WorthUiAuthoredMode, WorthUiDslProtocolIdentity, WorthUiSealedSemanticPackage,
    WorthUiSemanticPackageIdentity,
};

use super::WorthUiAuthoredOverlayMaterial;

/// Read-only evidence identifying the exact DSL package presented at the
/// authored-to-runtime ownership transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticHandoffEvidence {
    identity: WorthUiSemanticPackageIdentity,
    protocol: WorthUiDslProtocolIdentity,
    authored_mode: WorthUiAuthoredMode,
    projection_requirements: Box<[WorthUiAuthoredProjectionRequirement]>,
    projection_contents: Box<[WorthUiProjectionContentEdge]>,
    intent_material: crate::declaration::WorthUiAuthoredIntentMaterial,
    authored_overlay_material: WorthUiAuthoredOverlayMaterial,
    service_declarations: Box<[WorthUiAuthoredServiceDeclaration]>,
    authored_component_command_scopes: Box<[crate::capability::UiCommandRouteScopeIdentity]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiAuthoredServiceDeclaration {
    meaning: worth_ui_dsl::WorthUiServiceDeclarationMeaning,
    provenance: worth_ui_dsl::WorthUiArtifactInputProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiAuthoredProjectionRequirement {
    Scalar {
        declaration_identity: Box<str>,
        view_identity: worth_ui_query_binding::WorthUiQueryViewIdentity,
        requirement: worth_ui_query_binding::UiScalarSchemaRequirement,
    },
    Collection {
        declaration_identity: Box<str>,
        view_identity: worth_ui_query_binding::WorthUiQueryViewIdentity,
        requirement: worth_ui_query_binding::UiCollectionSchemaRequirement,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiProjectionContentEdge {
    component_identity: Box<str>,
    projection_identity: worth_ui_query_binding::WorthUiQueryViewIdentity,
}

impl WorthUiSemanticHandoffEvidence {
    pub(super) fn from_package(package: &WorthUiSealedSemanticPackage) -> Self {
        Self {
            identity: package.identity().clone(),
            protocol: package.protocol(),
            authored_mode: package.authored_mode(),
            projection_requirements: package
                .projection_requirements()
                .map(WorthUiAuthoredProjectionRequirement::from_sealed)
                .collect(),
            projection_contents: projection_contents(package),
            intent_material: Default::default(),
            authored_overlay_material: WorthUiAuthoredOverlayMaterial::from_package(package),
            service_declarations: package
                .service_declarations()
                .map(|(meaning, provenance)| WorthUiAuthoredServiceDeclaration {
                    meaning: meaning.clone(),
                    provenance: provenance.clone(),
                })
                .collect(),
            authored_component_command_scopes: authored_component_command_scopes(package),
        }
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

    pub fn projection_contents(&self) -> &[WorthUiProjectionContentEdge] {
        &self.projection_contents
    }

    pub fn projection_requirements(&self) -> &[WorthUiAuthoredProjectionRequirement] {
        &self.projection_requirements
    }

    pub(crate) fn admit_intent_material(
        &mut self,
        material: crate::declaration::WorthUiAuthoredIntentMaterial,
    ) {
        self.intent_material = material;
    }

    pub(crate) fn intent_material(&self) -> &crate::declaration::WorthUiAuthoredIntentMaterial {
        &self.intent_material
    }

    pub fn service_declarations(&self) -> &[WorthUiAuthoredServiceDeclaration] {
        &self.service_declarations
    }

    pub fn authored_overlay_material(&self) -> &WorthUiAuthoredOverlayMaterial {
        &self.authored_overlay_material
    }

    pub(crate) fn declares_command_scope(
        &self,
        scope: crate::capability::UiCommandRouteScopeIdentity,
    ) -> bool {
        self.authored_component_command_scopes
            .binary_search(&scope)
            .is_ok()
    }

    pub(crate) fn runtime_service_support(&self) -> crate::capability::UiRuntimeServiceSupport {
        self.service_declarations.iter().fold(
            self.intent_material.runtime_service_support(),
            |support, declaration| support.union(declaration.runtime_service_support()),
        )
    }

    pub fn projection_requirement(
        &self,
        identity: &worth_ui_query_binding::WorthUiQueryViewIdentity,
    ) -> Option<&WorthUiAuthoredProjectionRequirement> {
        self.projection_requirements
            .iter()
            .find(|requirement| requirement.view_identity() == identity)
    }
}

fn authored_component_command_scopes(
    package: &WorthUiSealedSemanticPackage,
) -> Box<[crate::capability::UiCommandRouteScopeIdentity]> {
    let mut scopes = package
        .module_ids()
        .iter()
        .flat_map(|module_id| {
            package
                .module(module_id)
                .expect("sealed package contains every canonical module")
                .declarations()
                .iter()
        })
        .filter_map(|declaration| match declaration {
            worth_ui_dsl::WorthUiSemanticDeclaration::Component(component) => Some(
                crate::capability::UiCommandRouteScopeIdentity::for_authored_component(
                    component.name_text(),
                ),
            ),
            _ => None,
        })
        .collect::<Vec<_>>();
    scopes.sort_unstable();
    scopes.dedup();
    scopes.into_boxed_slice()
}

impl WorthUiAuthoredServiceDeclaration {
    pub fn meaning(&self) -> &worth_ui_dsl::WorthUiServiceDeclarationMeaning {
        &self.meaning
    }

    pub fn provenance(&self) -> &worth_ui_dsl::WorthUiArtifactInputProvenance {
        &self.provenance
    }

    pub fn identity(&self) -> &str {
        service_identity(&self.meaning)
    }

    fn runtime_service_support(&self) -> crate::capability::UiRuntimeServiceSupport {
        use crate::capability::UiRuntimeServiceFamily as Family;
        let support = crate::capability::UiRuntimeServiceSupport::none_installed();
        match self.meaning {
            // A portal declaration demands its focus and motion requirements, plus
            // the Scroll owner that owns the focus owner's one reveal decision.
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Portal(_) => support
                .with_installed(Family::Portal)
                .with_installed(Family::Focus)
                .with_installed(Family::Motion)
                .with_installed(Family::Scroll),
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Focus(_) => {
                support.with_installed(Family::Focus)
            }
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Motion(_) => {
                support.with_installed(Family::Motion)
            }
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Command(_) => {
                support.with_installed(Family::CommandRouting)
            }
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Scroll(_) => {
                support.with_installed(Family::Scroll)
            }
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Selection(_) => {
                support.with_installed(Family::Selection)
            }
        }
    }
}

fn service_identity(meaning: &worth_ui_dsl::WorthUiServiceDeclarationMeaning) -> &str {
    match meaning {
        worth_ui_dsl::WorthUiServiceDeclarationMeaning::Portal(value) => value.identity(),
        worth_ui_dsl::WorthUiServiceDeclarationMeaning::Focus(value) => value.identity(),
        worth_ui_dsl::WorthUiServiceDeclarationMeaning::Motion(value) => value.identity(),
        worth_ui_dsl::WorthUiServiceDeclarationMeaning::Command(value) => value.identity(),
        worth_ui_dsl::WorthUiServiceDeclarationMeaning::Scroll(value) => value.identity(),
        worth_ui_dsl::WorthUiServiceDeclarationMeaning::Selection(value) => value.identity(),
    }
}

impl WorthUiAuthoredProjectionRequirement {
    fn from_sealed(requirement: &worth_ui_dsl::WorthUiProjectionRequirement) -> Self {
        let declaration_identity = requirement.declaration_identity().into();
        let view_identity =
            worth_ui_query_binding::WorthUiQueryViewIdentity::new(requirement.view_identity())
                .expect("sealed DSL Query view identity remains valid");
        let native_family = match requirement.native_family() {
            worth_ui_dsl::WorthUiProjectionNativeFamily::Text => {
                worth_ui_query_binding::UiProjectionNativeFamily::Text
            }
            worth_ui_dsl::WorthUiProjectionNativeFamily::Boolean => {
                worth_ui_query_binding::UiProjectionNativeFamily::Boolean
            }
        };
        let lifecycle = match requirement.lifecycle() {
            worth_ui_dsl::WorthUiProjectionLifecycle::Snapshot => {
                worth_ui_query_binding::UiProjectionLifecycleRequirement::Snapshot
            }
            worth_ui_dsl::WorthUiProjectionLifecycle::Live => {
                worth_ui_query_binding::UiProjectionLifecycleRequirement::Live
            }
        };
        match requirement.shape() {
            worth_ui_dsl::WorthUiProjectionShape::Scalar => {
                let selected_field = requirement
                    .selected_fields()
                    .next()
                    .expect("sealed scalar projection has one selected field");
                Self::Scalar {
                    declaration_identity,
                    view_identity,
                    requirement: worth_ui_query_binding::UiScalarSchemaRequirement::native(
                        declared_field(selected_field),
                        native_family,
                        lifecycle,
                    ),
                }
            }
            worth_ui_dsl::WorthUiProjectionShape::Collection => {
                let policy = requirement
                    .collection_policy()
                    .expect("sealed collection projection has a collection policy");
                Self::Collection {
                    declaration_identity,
                    view_identity,
                    requirement: worth_ui_query_binding::UiCollectionSchemaRequirement::native(
                        declared_field(
                            requirement
                                .row_identity_field()
                                .expect("sealed collection projection has row identity"),
                        ),
                        requirement.selected_fields().map(declared_field),
                        native_family,
                        lifecycle,
                        policy.requires_complete_result(),
                        policy.permits_continuation(),
                    )
                    .expect("sealed collection projection has valid selected fields"),
                }
            }
        }
    }

    pub fn declaration_identity(&self) -> &str {
        match self {
            Self::Scalar {
                declaration_identity,
                ..
            }
            | Self::Collection {
                declaration_identity,
                ..
            } => declaration_identity,
        }
    }

    pub fn view_identity(&self) -> &worth_ui_query_binding::WorthUiQueryViewIdentity {
        match self {
            Self::Scalar { view_identity, .. } | Self::Collection { view_identity, .. } => {
                view_identity
            }
        }
    }

    pub fn scalar_requirement(&self) -> Option<&worth_ui_query_binding::UiScalarSchemaRequirement> {
        match self {
            Self::Scalar { requirement, .. } => Some(requirement),
            Self::Collection { .. } => None,
        }
    }

    pub fn collection_requirement(
        &self,
    ) -> Option<&worth_ui_query_binding::UiCollectionSchemaRequirement> {
        match self {
            Self::Collection { requirement, .. } => Some(requirement),
            Self::Scalar { .. } => None,
        }
    }
}

fn declared_field(field: &str) -> worth_ui_query_binding::UiProjectionFieldRequirement {
    match field {
        "status" => worth_ui_query_binding::UiProjectionFieldRequirement::query_text_status(),
        "value" => worth_ui_query_binding::UiProjectionFieldRequirement::measurement_value(),
        "id" | "identity" | "identity.id" => {
            worth_ui_query_binding::UiProjectionFieldRequirement::identity_id()
        }
        "revision" => worth_ui_query_binding::UiProjectionFieldRequirement::query_revision(),
        other => panic!("sealed DSL projection field is not in WorthUiApplicationSchema: {other}"),
    }
}

impl WorthUiProjectionContentEdge {
    pub fn component_identity(&self) -> &str {
        &self.component_identity
    }

    pub fn projection_identity(&self) -> &worth_ui_query_binding::WorthUiQueryViewIdentity {
        &self.projection_identity
    }
}

fn projection_contents(
    package: &WorthUiSealedSemanticPackage,
) -> Box<[WorthUiProjectionContentEdge]> {
    let views = package
        .projection_requirements()
        .map(|requirement| {
            (
                requirement.declaration_identity(),
                requirement.view_identity(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    package
        .module_ids()
        .iter()
        .flat_map(|module_id| {
            package
                .module(module_id)
                .expect("sealed package contains every canonical module")
                .declarations()
                .iter()
                .filter_map(|declaration| match declaration {
                    worth_ui_dsl::WorthUiSemanticDeclaration::Component(component) => {
                        Some(component)
                    }
                    _ => None,
                })
                .flat_map(|component| {
                    component
                        .structure()
                        .projection_contents()
                        .iter()
                        .map(|content| {
                            let view = views[content.projection_identity_text()];
                            WorthUiProjectionContentEdge {
                                component_identity: format!("component:{}", component.name_text())
                                    .into(),
                                projection_identity:
                                    worth_ui_query_binding::WorthUiQueryViewIdentity::new(view)
                                        .expect("sealed DSL projection view identity is valid"),
                            }
                        })
                })
        })
        .collect()
}
