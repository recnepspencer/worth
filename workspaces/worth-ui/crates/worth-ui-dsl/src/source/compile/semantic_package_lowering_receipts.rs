use std::collections::BTreeMap;

use crate::{
    UiDslLoweringReceipt, UiDslPostureToken, UiDslSemanticArtifact, UiDslSemanticArtifactSpec,
    UiDslSemanticFamily, UiDslSemanticKey, UiDslSourceProvenance, UiDslStructuralToken,
    WorthUiArtifactInputProvenance,
};

use super::{
    WorthUiSealedSemanticArtifact, WorthUiSealedSemanticPackage, WorthUiSemanticBlock,
    WorthUiSemanticDeclaration, WorthUiSemanticDeclarationView,
};

pub(super) fn lower(package: &WorthUiSealedSemanticPackage) -> Vec<UiDslLoweringReceipt> {
    let mut inputs = Vec::new();
    for module_id in package.module_ids() {
        let Some(views) = package.declaration_views(module_id) else {
            continue;
        };
        inputs.extend(views.filter_map(lower_declaration));
    }
    let generations = source_generations(&inputs);
    inputs
        .into_iter()
        .map(|input| {
            let generation = generations[&source_artifact_key(&input.provenance)];
            UiDslLoweringReceipt::new(
                input.artifact,
                input.semantic_input_digest,
                generation,
                input.provenance,
            )
        })
        .collect()
}

struct LoweringInput {
    artifact: UiDslSemanticArtifact,
    semantic_input_digest: u64,
    provenance: UiDslSourceProvenance,
}

fn lower_declaration(view: WorthUiSemanticDeclarationView<'_>) -> Option<LoweringInput> {
    let provenance = dsl_provenance(view.provenance());
    let spec = match view.declaration() {
        WorthUiSemanticDeclaration::Component(block) => {
            structural_spec("component", block, provenance.clone())
        }
        WorthUiSemanticDeclaration::Surface(block) => {
            structural_spec("surface", block, provenance.clone())
        }
        WorthUiSemanticDeclaration::Binding(block) => {
            structural_spec("binding", block, provenance.clone())
        }
        WorthUiSemanticDeclaration::SemanticArtifact(artifact) => {
            if artifact.declaration().service_declaration().is_some() {
                return None;
            }
            semantic_artifact_spec(artifact, provenance.clone())
        }
        WorthUiSemanticDeclaration::Import(_)
        | WorthUiSemanticDeclaration::Projection(_)
        | WorthUiSemanticDeclaration::Token(_)
        | WorthUiSemanticDeclaration::AppearanceRole(_)
        | WorthUiSemanticDeclaration::Backdrop(_) => {
            return None;
        }
    };
    let artifact = spec.into_artifact();
    Some(LoweringInput {
        semantic_input_digest: semantic_input_digest(&artifact),
        artifact,
        provenance,
    })
}

fn semantic_artifact_spec(
    artifact: &WorthUiSealedSemanticArtifact,
    provenance: UiDslSourceProvenance,
) -> UiDslSemanticArtifactSpec {
    let declaration = artifact.declaration();
    let mut spec =
        UiDslSemanticArtifactSpec::new(declaration.key().clone(), declaration.family(), provenance);
    for aspect in declaration.published_aspects() {
        spec = spec.with_published_aspect(aspect.clone());
    }
    for aspect in declaration.consumed_aspects() {
        spec = spec.with_consumed_aspect(aspect.clone());
    }
    for token in declaration.structural_tokens() {
        spec = spec.with_structural_token(token.clone());
    }
    for token in declaration.posture_tokens() {
        spec = spec.with_posture_token(token.clone());
    }
    for token in declaration.support_tokens() {
        spec = spec.with_support_token(token.clone());
    }
    if let Some(component) = declaration.component_reference() {
        spec = spec
            .with_component_reference(component.clone())
            .expect("one sealed semantic declaration carries at most one component reference");
    }
    if let Some(attachment) = declaration.appearance_role_attachment() {
        spec = spec
            .with_appearance_role_attachment(attachment.clone())
            .expect("one sealed semantic declaration carries at most one appearance attachment");
    }
    spec
}

fn structural_spec(
    family: &str,
    block: &WorthUiSemanticBlock,
    provenance: UiDslSourceProvenance,
) -> UiDslSemanticArtifactSpec {
    let identity = match block.authored_identity() {
        Some(identity) => format!("{family}:authored:{identity}"),
        None => format!("{family}:identity:{}", block.name_text()),
    };
    let module_path = provenance.module_path().to_owned();
    let mut spec = UiDslSemanticArtifactSpec::new(
        UiDslSemanticKey::new(format!("{family}:{}", block.name_text())),
        UiDslSemanticFamily::Mosaic,
        provenance,
    )
    .with_structural_token(UiDslStructuralToken::new(format!(
        "mosaic:{module_path}|{identity}"
    )));
    if !block.structure().interaction_routes().is_empty() {
        spec = spec.with_posture_token(UiDslPostureToken::new(
            "intent:attached:canonical-route-catalog",
        ));
    }
    if let Some(attachment) = block.appearance_role_attachment() {
        spec = spec
            .with_appearance_role_attachment(attachment.clone())
            .expect("one sealed component carries at most one appearance attachment");
    }
    spec
}

fn dsl_provenance(provenance: &WorthUiArtifactInputProvenance) -> UiDslSourceProvenance {
    match provenance {
        WorthUiArtifactInputProvenance::ParsedSourceDeclaration {
            declaration_span,
            declaration_index,
            ..
        } => UiDslSourceProvenance::file_authored(
            declaration_span.module_id().as_str(),
            *declaration_index,
        ),
        WorthUiArtifactInputProvenance::RustAuthoredDeclaration {
            authored_module_path,
            declaration_index,
        } => UiDslSourceProvenance::rust_authored(authored_module_path, *declaration_index),
    }
}

fn source_generations(inputs: &[LoweringInput]) -> BTreeMap<String, u64> {
    let mut grouped = BTreeMap::<String, Vec<u64>>::new();
    for input in inputs {
        grouped
            .entry(source_artifact_key(&input.provenance))
            .or_default()
            .push(input.semantic_input_digest);
    }
    grouped
        .into_iter()
        .map(|(key, digests)| {
            let generation = digests
                .into_iter()
                .fold(stable_text_digest(&key), |digest, semantic| {
                    digest.rotate_left(9) ^ semantic
                });
            (key, generation)
        })
        .collect()
}

fn source_artifact_key(provenance: &UiDslSourceProvenance) -> String {
    match provenance {
        UiDslSourceProvenance::FileAuthored { module_path, .. } => {
            format!("file:{module_path}")
        }
        UiDslSourceProvenance::RustAuthored { module_path, .. } => {
            format!("rust:{module_path}")
        }
    }
}

fn semantic_input_digest(artifact: &UiDslSemanticArtifact) -> u64 {
    stable_text_digest(artifact.key().as_str())
        ^ stable_text_digest(artifact.family().as_str()).rotate_left(7)
        ^ digest_texts(
            artifact
                .published_aspects()
                .iter()
                .map(|value| value.as_str()),
        )
        .rotate_left(17)
        ^ digest_texts(
            artifact
                .consumed_aspects()
                .iter()
                .map(|value| value.as_str()),
        )
        .rotate_left(23)
        ^ digest_texts(
            artifact
                .structural_tokens()
                .iter()
                .map(|value| value.as_str()),
        )
        .rotate_left(31)
        ^ digest_texts(artifact.posture_tokens().iter().map(|value| value.as_str())).rotate_left(41)
        ^ digest_texts(artifact.support_tokens().iter().map(|value| value.as_str())).rotate_left(53)
        ^ artifact
            .component_reference()
            .map_or(0, |component| stable_text_digest(component.as_str()))
            .rotate_left(47)
        ^ artifact
            .appearance_role_attachment()
            .map_or(
                0,
                crate::UiAppearanceRoleAttachmentDeclaration::semantic_digest,
            )
            .rotate_left(59)
}

fn digest_texts<'a>(values: impl IntoIterator<Item = &'a str>) -> u64 {
    let mut canonical = values.into_iter().collect::<Vec<_>>();
    canonical.sort_unstable();
    canonical.dedup();
    canonical
        .into_iter()
        .fold(0x9E37_79B9_7F4A_7C15, |digest, value| {
            digest.rotate_left(5) ^ stable_text_digest(value)
        })
}

fn stable_text_digest(text: &str) -> u64 {
    text.as_bytes()
        .iter()
        .fold(0xCBF2_9CE4_8422_2325, |digest, byte| {
            digest.wrapping_mul(0x0000_0100_0000_01B3) ^ u64::from(*byte)
        })
}
