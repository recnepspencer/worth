use std::collections::BTreeMap;

#[cfg(test)]
use worth_ui_dsl::{UiDslLoweringReceipt, WorthUiDslCompiler};
use worth_ui_dsl::{
    UiDslSemanticArtifactSpec, WorthUiRustAuthoredArtifactInput,
    WorthUiRustAuthoredArtifactInputModule, WorthUiSemanticArtifactDeclaration,
};

#[derive(Clone)]
pub(crate) struct WorthUiRustAuthoredDeclarationFixture {
    appearance_roles: Vec<(String, worth_ui_dsl::UiAppearanceRoleDeclaration)>,
    components: Vec<(
        String,
        String,
        Option<worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration>,
    )>,
    specs: Vec<UiDslSemanticArtifactSpec>,
}

impl WorthUiRustAuthoredDeclarationFixture {
    pub(crate) fn empty() -> Self {
        Self {
            appearance_roles: Vec::new(),
            components: Vec::new(),
            specs: Vec::new(),
        }
    }

    pub(crate) fn named(_diagnostic_name: impl Into<String>) -> Self {
        Self::empty()
    }

    pub(crate) fn with_semantic_artifact_spec(mut self, spec: UiDslSemanticArtifactSpec) -> Self {
        self.specs.push(spec);
        self
    }

    pub(crate) fn with_appearance_role(
        mut self,
        module_path: impl Into<String>,
        role: worth_ui_dsl::UiAppearanceRoleDeclaration,
    ) -> Self {
        self.appearance_roles.push((module_path.into(), role));
        self
    }

    pub(crate) fn with_component_appearance_role(
        mut self,
        module_path: impl Into<String>,
        name_text: impl Into<String>,
        attachment: worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration,
    ) -> Self {
        self.components
            .push((module_path.into(), name_text.into(), Some(attachment)));
        self
    }

    #[cfg(test)]
    pub(crate) fn admit_semantic_artifact(
        &self,
        spec: UiDslSemanticArtifactSpec,
    ) -> UiDslLoweringReceipt {
        let package = WorthUiDslCompiler::compile_rust_authored(
            &self.clone().with_semantic_artifact_spec(spec).into_input(),
        )
        .expect("semantic declaration fixture should compile");
        package
            .declaration_lowering_receipts()
            .into_iter()
            .last()
            .expect("incoming declaration should mint one lowering receipt")
    }

    pub(crate) fn into_input(self) -> WorthUiRustAuthoredArtifactInput {
        rust_authored_input_from_declarations(self.appearance_roles, self.components, self.specs)
    }
}

fn rust_authored_input_from_declarations(
    appearance_roles: impl IntoIterator<Item = (String, worth_ui_dsl::UiAppearanceRoleDeclaration)>,
    components: impl IntoIterator<
        Item = (
            String,
            String,
            Option<worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration>,
        ),
    >,
    specs: impl IntoIterator<Item = UiDslSemanticArtifactSpec>,
) -> WorthUiRustAuthoredArtifactInput {
    let mut modules = BTreeMap::<String, WorthUiRustAuthoredArtifactInputModule>::new();
    for (module_path, role) in appearance_roles {
        let module = modules
            .remove(&module_path)
            .unwrap_or_else(|| WorthUiRustAuthoredArtifactInputModule::new(&module_path))
            .with_appearance_role(role);
        modules.insert(module_path, module);
    }
    for (module_path, name_text, attachment) in components {
        let module = modules
            .remove(&module_path)
            .unwrap_or_else(|| WorthUiRustAuthoredArtifactInputModule::new(&module_path));
        let module = match attachment {
            Some(attachment) => module
                .with_component_appearance_role(name_text, attachment)
                .expect("one fixture component carries at most one appearance attachment"),
            None => module.with_component(name_text),
        };
        modules.insert(module_path, module);
    }
    for spec in specs {
        let artifact = spec.into_semantic_artifact();
        push_artifact(&mut modules, &artifact);
    }
    WorthUiRustAuthoredArtifactInput::from_modules(modules.into_values())
}

fn push_artifact(
    modules: &mut BTreeMap<String, WorthUiRustAuthoredArtifactInputModule>,
    artifact: &worth_ui_dsl::UiDslSemanticArtifact,
) {
    let module_path = artifact.provenance().module_path().to_owned();
    let declaration = semantic_declaration(artifact);
    let module = modules
        .remove(&module_path)
        .unwrap_or_else(|| WorthUiRustAuthoredArtifactInputModule::new(&module_path))
        .with_semantic_declaration(declaration);
    modules.insert(module_path, module);
}

fn semantic_declaration(
    artifact: &worth_ui_dsl::UiDslSemanticArtifact,
) -> WorthUiSemanticArtifactDeclaration {
    let declaration =
        WorthUiSemanticArtifactDeclaration::new(artifact.key().clone(), artifact.family());
    let declaration = artifact.published_aspects().iter().cloned().fold(
        declaration,
        WorthUiSemanticArtifactDeclaration::with_published_aspect,
    );
    let declaration = artifact.consumed_aspects().iter().cloned().fold(
        declaration,
        WorthUiSemanticArtifactDeclaration::with_consumed_aspect,
    );
    let declaration = artifact.structural_tokens().iter().cloned().fold(
        declaration,
        WorthUiSemanticArtifactDeclaration::with_structural_token,
    );
    let declaration = artifact.posture_tokens().iter().cloned().fold(
        declaration,
        WorthUiSemanticArtifactDeclaration::with_posture_token,
    );
    let mut declaration = artifact.support_tokens().iter().cloned().fold(
        declaration,
        WorthUiSemanticArtifactDeclaration::with_support_token,
    );
    if let Some(component) = artifact.component_reference() {
        declaration = declaration
            .with_component_reference(component.clone())
            .expect("one semantic artifact carries at most one component reference");
    }
    match artifact.appearance_role_attachment() {
        Some(attachment) => declaration
            .with_appearance_role_attachment(attachment.clone())
            .expect("one semantic artifact carries at most one appearance attachment"),
        None => declaration,
    }
}
