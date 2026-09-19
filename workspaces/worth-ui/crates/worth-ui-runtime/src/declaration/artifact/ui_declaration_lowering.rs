use worth_ui_dsl::{
    UiDslAspectName, UiDslLoweringReceipt, UiDslPostureToken, UiDslSemanticArtifact,
    UiDslSemanticArtifactSpec, UiDslSemanticFamily, UiDslSemanticKey, UiDslSourceProvenance,
    UiDslStructuralToken, UiDslSupportToken,
};

use crate::declaration::artifact::ui_declaration_identity::stable_text_digest;
use crate::declaration::declared_posture::admit_declared_posture_contract;
use crate::declaration::family::admit_declaration_family;
use crate::declaration::structural_semantics::admit_declaration_structural_semantics;
use crate::declaration::support::admit_declaration_support_snapshot;
use crate::declaration::{
    UiAspectContract, UiDeclarationArtifact, UiDeclarationArtifactDigest,
    UiDeclarationAspectDigest, UiDeclarationDigestProjection, UiDeclarationFamilyDigest,
    UiDeclarationIdentity, UiDeclarationPostureDigest, UiDeclarationProvenance,
    UiDeclarationStructuralDigest, UiDeclarationSupportDigest,
};

pub(crate) struct UiDeclarationLowering {
    _sealed: (),
}

impl UiDeclarationLowering {
    pub(crate) fn lower(semantic_receipt: UiDslLoweringReceipt) -> UiDeclarationArtifact {
        let semantic_artifact = semantic_receipt.semantic_artifact().clone();
        Self::lower_semantic_input(
            semantic_artifact,
            semantic_receipt.semantic_input_digest(),
            semantic_receipt.source_artifact_generation(),
            semantic_receipt.source_provenance().clone(),
        )
    }

    pub(crate) fn lower_runtime_bootstrap() -> UiDeclarationArtifact {
        let source_provenance =
            UiDslSourceProvenance::rust_authored("worth-ui.runtime.bootstrap", 0);
        let semantic_artifact = UiDslSemanticArtifactSpec::new(
            UiDslSemanticKey::new("worth_ui.runtime.bootstrap.product_root"),
            UiDslSemanticFamily::Page,
            source_provenance.clone(),
        )
        .with_published_aspect(UiDslAspectName::new("structure.product-root"))
        .with_structural_token(UiDslStructuralToken::new("page:product-root"))
        .with_posture_token(UiDslPostureToken::new("world:authoritative"))
        .with_support_token(UiDslSupportToken::new("support:runtime-bootstrap"))
        .into_semantic_artifact();
        let semantic_input_digest = semantic_input_digest(&semantic_artifact);
        let source_artifact_generation = stable_text_digest("rust:worth-ui.runtime.bootstrap")
            .rotate_left(9)
            ^ semantic_input_digest;
        Self::lower_semantic_input(
            semantic_artifact,
            semantic_input_digest,
            source_artifact_generation,
            source_provenance,
        )
    }

    fn lower_semantic_input(
        semantic_artifact: UiDslSemanticArtifact,
        semantic_input_digest: u64,
        source_artifact_generation: u64,
        source_provenance: UiDslSourceProvenance,
    ) -> UiDeclarationArtifact {
        let family_digest =
            UiDeclarationFamilyDigest::new(stable_text_digest(semantic_artifact.family().as_str()));
        let aspect_contract_admission = UiAspectContract::admit(&semantic_artifact);
        let family_admission = admit_declaration_family(&semantic_artifact);
        let declared_posture_admission =
            admit_declared_posture_contract(&semantic_artifact, &family_admission);
        let declaration_support_snapshot_admission =
            admit_declaration_support_snapshot(&declared_posture_admission);
        let structural_semantics_admission =
            admit_declaration_structural_semantics(&semantic_artifact, &family_admission);
        let aspect_digest = UiDeclarationAspectDigest::new(aspect_contract_admission.digest_raw());
        let structural_digest = UiDeclarationStructuralDigest::new(digest_string_slice(
            semantic_artifact.structural_tokens(),
        ));
        let posture_digest = UiDeclarationPostureDigest::new(digest_string_slice(
            semantic_artifact.posture_tokens(),
        ));
        let support_digest = UiDeclarationSupportDigest::new(digest_string_slice(
            semantic_artifact.support_tokens(),
        ));
        let identity = UiDeclarationIdentity::new(
            family_digest,
            aspect_digest,
            structural_digest,
            posture_digest,
            semantic_artifact.key().as_str(),
        );
        let digests = UiDeclarationDigestProjection::new(
            UiDeclarationArtifactDigest::new(
                identity.digest().raw()
                    ^ support_digest.raw().rotate_left(23)
                    ^ semantic_input_digest.rotate_left(41),
            ),
            identity.digest(),
            family_digest,
            aspect_digest,
            structural_digest,
            posture_digest,
            support_digest,
        );
        let provenance = UiDeclarationProvenance::new(
            source_provenance,
            semantic_input_digest,
            source_artifact_generation,
        );
        let authored_appearance_role_attachment =
            semantic_artifact.appearance_role_attachment().cloned();
        let authored_component_reference = semantic_artifact.component_reference().cloned();
        UiDeclarationArtifact::new(crate::declaration::UiDeclarationArtifactInput {
            identity,
            digests,
            aspect_contract_admission,
            declared_posture_admission,
            declaration_support_snapshot_admission,
            structural_semantics_admission,
            family_admission,
            provenance,
            authored_appearance_role_attachment,
            authored_component_reference,
        })
    }
}

fn semantic_input_digest(artifact: &UiDslSemanticArtifact) -> u64 {
    stable_text_digest(artifact.key().as_str())
        ^ stable_text_digest(artifact.family().as_str()).rotate_left(7)
        ^ digest_string_slice(artifact.published_aspects()).rotate_left(17)
        ^ digest_string_slice(artifact.consumed_aspects()).rotate_left(23)
        ^ digest_string_slice(artifact.structural_tokens()).rotate_left(31)
        ^ digest_string_slice(artifact.posture_tokens()).rotate_left(41)
        ^ digest_string_slice(artifact.support_tokens()).rotate_left(53)
}

fn digest_string_slice<T>(values: &[T]) -> u64
where
    T: DeclarationDigestText,
{
    let mut canonical = values
        .iter()
        .map(DeclarationDigestText::digest_text)
        .collect::<Vec<_>>();
    canonical.sort();
    canonical.dedup();

    canonical
        .iter()
        .fold(0x9E37_79B9_7F4A_7C15, |digest, value| {
            digest.rotate_left(5) ^ stable_text_digest(value)
        })
}

trait DeclarationDigestText {
    fn digest_text(&self) -> &str;
}

impl DeclarationDigestText for UiDslStructuralToken {
    fn digest_text(&self) -> &str {
        self.as_str()
    }
}

impl DeclarationDigestText for UiDslPostureToken {
    fn digest_text(&self) -> &str {
        self.as_str()
    }
}

impl DeclarationDigestText for UiDslSupportToken {
    fn digest_text(&self) -> &str {
        self.as_str()
    }
}

impl DeclarationDigestText for UiDslAspectName {
    fn digest_text(&self) -> &str {
        self.as_str()
    }
}
