use crate::source::WorthUiSemanticDeclaration;

use super::{Fingerprint, WorthUiSemanticBlockExactBasis};

#[derive(Debug, Eq, PartialEq)]
pub(super) enum WorthUiSemanticDeclarationExactBasis {
    Import {
        target: String,
    },
    Component(WorthUiSemanticBlockExactBasis),
    Surface(WorthUiSemanticBlockExactBasis),
    Binding(WorthUiSemanticBlockExactBasis),
    Projection {
        declaration: String,
        view: String,
        shape: String,
        selected_fields: Box<[String]>,
        row_identity: Option<String>,
        native_family: String,
        lifecycle: String,
        requires_complete_result: Option<bool>,
        permits_continuation: Option<bool>,
    },
    Token {
        name: String,
        authored_identity: Option<String>,
        value: String,
    },
    SemanticArtifact {
        key: String,
        family: String,
        published_aspects: Box<[String]>,
        consumed_aspects: Box<[String]>,
        structural_tokens: Box<[String]>,
        posture_tokens: Box<[String]>,
        support_tokens: Box<[String]>,
        intent: Option<WorthUiIntentDeclarationExactBasis>,
        service: Option<String>,
    },
    AppearanceRole(Box<[u8]>),
    Backdrop(Box<[u8]>),
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct WorthUiIntentDeclarationExactBasis {
    definition: String,
    interaction: String,
    payload_schema: Option<(String, u16)>,
    outcome_schema: Option<(String, u16)>,
    payload_sources: Box<[String]>,
    operability: String,
    confirmation: String,
    concurrency: String,
}

impl WorthUiSemanticDeclarationExactBasis {
    pub(super) fn from_declaration(declaration: &WorthUiSemanticDeclaration) -> Self {
        match declaration {
            WorthUiSemanticDeclaration::Import(import) => Self::Import {
                target: import.target().authored_text().to_owned(),
            },
            WorthUiSemanticDeclaration::Component(block) => {
                Self::Component(WorthUiSemanticBlockExactBasis::from_block(block))
            }
            WorthUiSemanticDeclaration::Surface(block) => {
                Self::Surface(WorthUiSemanticBlockExactBasis::from_block(block))
            }
            WorthUiSemanticDeclaration::Binding(block) => {
                Self::Binding(WorthUiSemanticBlockExactBasis::from_block(block))
            }
            WorthUiSemanticDeclaration::Projection(projection) => {
                let requirement = projection.requirement();
                Self::Projection {
                    declaration: requirement.declaration_identity().to_owned(),
                    view: requirement.view_identity().to_owned(),
                    shape: requirement.shape().canonical_token().to_owned(),
                    selected_fields: requirement.selected_fields().map(str::to_owned).collect(),
                    row_identity: requirement.row_identity_field().map(str::to_owned),
                    native_family: requirement.native_family().canonical_token().to_owned(),
                    lifecycle: requirement.lifecycle().canonical_token().to_owned(),
                    requires_complete_result: requirement
                        .collection_policy()
                        .map(|policy| policy.requires_complete_result()),
                    permits_continuation: requirement
                        .collection_policy()
                        .map(|policy| policy.permits_continuation()),
                }
            }
            WorthUiSemanticDeclaration::Token(token) => Self::Token {
                name: token.name_text().to_owned(),
                authored_identity: token.authored_identity().map(str::to_owned),
                value: token.value_text().to_owned(),
            },
            WorthUiSemanticDeclaration::SemanticArtifact(artifact) => {
                let declaration = artifact.declaration();
                Self::SemanticArtifact {
                    key: declaration.key().as_str().to_owned(),
                    family: declaration.family().as_str().to_owned(),
                    published_aspects: semantic_texts(declaration.published_aspects()),
                    consumed_aspects: semantic_texts(declaration.consumed_aspects()),
                    structural_tokens: semantic_texts(declaration.structural_tokens()),
                    posture_tokens: semantic_texts(declaration.posture_tokens()),
                    support_tokens: semantic_texts(declaration.support_tokens()),
                    intent: declaration
                        .intent_declaration()
                        .map(WorthUiIntentDeclarationExactBasis::from_meaning),
                    service: declaration
                        .service_declaration()
                        .map(crate::WorthUiServiceDeclarationMeaning::canonical_text),
                }
            }
            WorthUiSemanticDeclaration::AppearanceRole(role) => {
                Self::AppearanceRole(role.role().canonical_bytes().into_boxed_slice())
            }
            WorthUiSemanticDeclaration::Backdrop(backdrop) => {
                Self::Backdrop(backdrop.declaration().canonical_bytes().into_boxed_slice())
            }
        }
    }

    pub(super) fn fold_into(&self, fingerprint: &mut Fingerprint) {
        match self {
            Self::Import { target } => {
                fingerprint.fold_text("import");
                fingerprint.fold_text(target);
            }
            Self::Component(block) => {
                fingerprint.fold_text("component");
                block.fold_into(fingerprint);
            }
            Self::Surface(block) => {
                fingerprint.fold_text("surface");
                block.fold_into(fingerprint);
            }
            Self::Binding(block) => {
                fingerprint.fold_text("binding");
                block.fold_into(fingerprint);
            }
            Self::Projection {
                declaration,
                view,
                shape,
                selected_fields,
                row_identity,
                native_family,
                lifecycle,
                requires_complete_result,
                permits_continuation,
            } => {
                fingerprint.fold_text("projection");
                fingerprint.fold_text(declaration);
                fingerprint.fold_text(view);
                fingerprint.fold_text(shape);
                fingerprint.fold_texts(selected_fields);
                fingerprint.fold_optional_text(row_identity.as_deref());
                fingerprint.fold_text(native_family);
                fingerprint.fold_text(lifecycle);
                fingerprint.fold_optional_bool(*requires_complete_result);
                fingerprint.fold_optional_bool(*permits_continuation);
            }
            Self::Token {
                name,
                authored_identity,
                value,
            } => {
                fingerprint.fold_text("token");
                fingerprint.fold_text(name);
                fingerprint.fold_optional_text(authored_identity.as_deref());
                fingerprint.fold_text(value);
            }
            Self::SemanticArtifact {
                key,
                family,
                published_aspects,
                consumed_aspects,
                structural_tokens,
                posture_tokens,
                support_tokens,
                intent,
                service,
            } => {
                fingerprint.fold_text("semantic-artifact");
                fingerprint.fold_text(key);
                fingerprint.fold_text(family);
                fingerprint.fold_texts(published_aspects);
                fingerprint.fold_texts(consumed_aspects);
                fingerprint.fold_texts(structural_tokens);
                fingerprint.fold_texts(posture_tokens);
                fingerprint.fold_texts(support_tokens);
                match intent {
                    Some(intent) => {
                        fingerprint.fold_bool(true);
                        intent.fold_into(fingerprint);
                    }
                    None => fingerprint.fold_bool(false),
                }
                fingerprint.fold_optional_text(service.as_deref());
            }
            Self::AppearanceRole(bytes) => {
                fingerprint.fold_text("appearance-role");
                fingerprint.fold_bytes(bytes);
            }
            Self::Backdrop(text) => {
                fingerprint.fold_text("backdrop");
                fingerprint.fold_bytes(text);
            }
        }
    }
}

impl WorthUiIntentDeclarationExactBasis {
    fn from_meaning(meaning: &crate::WorthUiIntentDeclarationMeaning) -> Self {
        Self {
            definition: meaning.definition_reference().to_owned(),
            interaction: meaning.interaction().as_str().to_owned(),
            payload_schema: meaning
                .expected_payload_schema()
                .map(|schema| (schema.identity().to_owned(), schema.version())),
            outcome_schema: meaning
                .expected_outcome_schema()
                .map(|schema| (schema.identity().to_owned(), schema.version())),
            payload_sources: meaning
                .payload_sources()
                .iter()
                .map(crate::WorthUiIntentPayloadSourceSpec::revision_token)
                .collect(),
            operability: meaning.operability().revision_token(),
            confirmation: meaning.confirmation().revision_token(),
            concurrency: meaning.concurrency().canonical_token().to_owned(),
        }
    }

    fn fold_into(&self, fingerprint: &mut Fingerprint) {
        fingerprint.fold_text(&self.definition);
        fingerprint.fold_text(&self.interaction);
        fold_schema(fingerprint, self.payload_schema.as_ref());
        fold_schema(fingerprint, self.outcome_schema.as_ref());
        fingerprint.fold_texts(&self.payload_sources);
        fingerprint.fold_text(&self.operability);
        fingerprint.fold_text(&self.confirmation);
        fingerprint.fold_text(&self.concurrency);
    }
}

fn fold_schema(fingerprint: &mut Fingerprint, schema: Option<&(String, u16)>) {
    match schema {
        Some((identity, version)) => {
            fingerprint.fold_bool(true);
            fingerprint.fold_text(identity);
            fingerprint.fold_u16(*version);
        }
        None => fingerprint.fold_bool(false),
    }
}

fn semantic_texts<T: SemanticText>(values: &[T]) -> Box<[String]> {
    values
        .iter()
        .map(|value| value.semantic_text().to_owned())
        .collect()
}

trait SemanticText {
    fn semantic_text(&self) -> &str;
}

macro_rules! semantic_text {
    ($type:ty) => {
        impl SemanticText for $type {
            fn semantic_text(&self) -> &str {
                self.as_str()
            }
        }
    };
}

semantic_text!(crate::UiDslAspectName);
semantic_text!(crate::UiDslStructuralToken);
semantic_text!(crate::UiDslPostureToken);
semantic_text!(crate::UiDslSupportToken);
