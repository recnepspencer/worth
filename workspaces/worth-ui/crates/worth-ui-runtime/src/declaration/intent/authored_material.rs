use worth_ui_dsl::{
    UiDslSemanticFamily, UiPortalDeclarationId, WorthUiArtifactInputProvenance,
    WorthUiIntentDeclarationMeaning, WorthUiIntentInteractionFamily, WorthUiIntentInteractionRoute,
    WorthUiIntentPayloadSourceSpec, WorthUiSealedSemanticPackage, WorthUiSemanticDeclaration,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorthUiAuthoredIntentMaterial {
    declarations: Box<[WorthUiAuthoredIntentDeclaration]>,
    routes: Box<[WorthUiAuthoredIntentRoute]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiAuthoredIntentDeclaration {
    module_identity: Box<str>,
    identity: Box<str>,
    meaning: WorthUiIntentDeclarationMeaning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiAuthoredIntentRoute {
    target_provenance_digest: u64,
    route: WorthUiIntentInteractionRoute,
    portal_declaration: Option<UiPortalDeclarationId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiAuthoredIntentMaterialDenial {
    InvalidDeclaration {
        identity: Box<str>,
        reason: WorthUiAuthoredIntentDeclarationDenial,
    },
    UnknownPortalRoute {
        portal: Box<str>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiAuthoredIntentDeclarationDenial {
    MissingStructuredMeaning,
    UnexpectedSemanticSurface,
}

pub(crate) fn prepare_authored_intent_material(
    package: &WorthUiSealedSemanticPackage,
) -> Result<WorthUiAuthoredIntentMaterial, WorthUiAuthoredIntentMaterialDenial> {
    let mut declarations = Vec::new();
    let mut routes = Vec::new();
    for module_id in package.module_ids() {
        let views = package
            .declaration_views(module_id)
            .expect("sealed package contains every canonical module");
        for view in views {
            match view.declaration() {
                WorthUiSemanticDeclaration::SemanticArtifact(artifact)
                    if artifact.declaration().family() == UiDslSemanticFamily::Intent =>
                {
                    declarations.push(admit_declaration(
                        module_id.as_str(),
                        artifact.declaration(),
                    )?);
                }
                WorthUiSemanticDeclaration::Component(component) => {
                    let target_provenance_digest = provenance_digest(view.provenance());
                    routes.extend(
                        component
                            .structure()
                            .interaction_routes()
                        .iter()
                        .cloned()
                            .map(|route| {
                                let portal_declaration = route
                                    .opened_portal_identity()
                                    .map(|portal| {
                                        package
                                            .overlay_declaration_bindings()
                                            .portal_named(portal)
                                            .ok_or_else(|| {
                                                WorthUiAuthoredIntentMaterialDenial::UnknownPortalRoute {
                                                    portal: portal.into(),
                                                }
                                            })
                                    })
                                    .transpose();
                                portal_declaration.map(|portal_declaration| {
                                    WorthUiAuthoredIntentRoute {
                                        target_provenance_digest,
                                        route,
                                        portal_declaration,
                                    }
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                _ => {}
            }
        }
    }
    Ok(WorthUiAuthoredIntentMaterial {
        declarations: declarations.into_boxed_slice(),
        routes: routes.into_boxed_slice(),
    })
}

fn admit_declaration(
    module_identity: &str,
    declaration: &worth_ui_dsl::WorthUiSemanticArtifactDeclaration,
) -> Result<WorthUiAuthoredIntentDeclaration, WorthUiAuthoredIntentMaterialDenial> {
    let identity: Box<str> = declaration.key().as_str().into();
    if !declaration.published_aspects().is_empty()
        || !declaration.consumed_aspects().is_empty()
        || !declaration.structural_tokens().is_empty()
        || !declaration.support_tokens().is_empty()
    {
        return Err(invalid(
            identity,
            WorthUiAuthoredIntentDeclarationDenial::UnexpectedSemanticSurface,
        ));
    }
    let meaning = declaration.intent_declaration().cloned().ok_or_else(|| {
        invalid(
            identity.clone(),
            WorthUiAuthoredIntentDeclarationDenial::MissingStructuredMeaning,
        )
    })?;
    Ok(WorthUiAuthoredIntentDeclaration {
        module_identity: module_identity.into(),
        identity,
        meaning,
    })
}

fn invalid(
    identity: Box<str>,
    reason: WorthUiAuthoredIntentDeclarationDenial,
) -> WorthUiAuthoredIntentMaterialDenial {
    WorthUiAuthoredIntentMaterialDenial::InvalidDeclaration { identity, reason }
}

fn provenance_digest(provenance: &WorthUiArtifactInputProvenance) -> u64 {
    match provenance {
        WorthUiArtifactInputProvenance::ParsedSourceDeclaration {
            declaration_span,
            declaration_index,
            ..
        } => crate::declaration::authored_source_provenance_digest(
            declaration_span.module_id().as_str(),
            *declaration_index,
        ),
        WorthUiArtifactInputProvenance::RustAuthoredDeclaration {
            authored_module_path,
            declaration_index,
        } => crate::declaration::authored_source_provenance_digest(
            authored_module_path,
            *declaration_index,
        ),
    }
}

impl WorthUiAuthoredIntentMaterial {
    pub(crate) fn declarations(&self) -> &[WorthUiAuthoredIntentDeclaration] {
        &self.declarations
    }

    pub(crate) fn routes(&self) -> &[WorthUiAuthoredIntentRoute] {
        &self.routes
    }

    pub(crate) fn runtime_service_support(&self) -> crate::capability::UiRuntimeServiceSupport {
        let selection_declared = self.declarations.iter().any(|declaration| {
            declaration.interaction()
                == worth_ui_dsl::WorthUiIntentInteractionFamily::SelectionCommit
        });
        if selection_declared {
            crate::capability::UiRuntimeServiceSupport::none_installed()
                .with_installed(crate::capability::UiRuntimeServiceFamily::Selection)
        } else {
            crate::capability::UiRuntimeServiceSupport::none_installed()
        }
    }
}

impl WorthUiAuthoredIntentDeclaration {
    pub(crate) fn module_identity(&self) -> &str {
        &self.module_identity
    }

    pub(crate) fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) const fn meaning(&self) -> &WorthUiIntentDeclarationMeaning {
        &self.meaning
    }

    pub(crate) fn definition_reference(&self) -> &str {
        self.meaning.definition_reference()
    }

    pub(crate) const fn interaction(&self) -> WorthUiIntentInteractionFamily {
        self.meaning.interaction()
    }

    pub(crate) fn expected_payload_schema(
        &self,
    ) -> Option<&worth_ui_dsl::WorthUiIntentSchemaExpectation> {
        self.meaning.expected_payload_schema()
    }

    pub(crate) fn expected_outcome_schema(
        &self,
    ) -> Option<&worth_ui_dsl::WorthUiIntentSchemaExpectation> {
        self.meaning.expected_outcome_schema()
    }

    pub(crate) fn payload_sources(&self) -> &[WorthUiIntentPayloadSourceSpec] {
        self.meaning.payload_sources()
    }

    pub(crate) fn operability(&self) -> &worth_ui_dsl::WorthUiIntentOperabilityContractSpec {
        self.meaning.operability()
    }

    pub(crate) fn confirmation(&self) -> &worth_ui_dsl::WorthUiIntentConfirmationContractSpec {
        self.meaning.confirmation()
    }

    pub(crate) const fn concurrency(&self) -> worth_ui_dsl::WorthUiIntentConcurrencyScope {
        self.meaning.concurrency()
    }

    pub(crate) const fn consequences(&self) -> &worth_ui_dsl::WorthUiIntentConsequenceContractSpec {
        self.meaning.consequences()
    }
}

impl WorthUiAuthoredIntentRoute {
    pub(crate) const fn target_provenance_digest(&self) -> u64 {
        self.target_provenance_digest
    }

    pub(crate) const fn route(&self) -> &WorthUiIntentInteractionRoute {
        &self.route
    }

    pub(crate) const fn portal_declaration(&self) -> Option<UiPortalDeclarationId> {
        self.portal_declaration
    }
}
