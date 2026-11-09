use super::super::{WorthQueryMemoryWorkspace, WorthQueryWorkspaceErrorKind};
use worth_foundational::facade::CanonicalDigestDerivationDenial;

#[test]
fn oversized_collection_identity_is_refused_when_the_workspace_is_admitted() {
    let kind = "x".repeat(4 * 1024 * 1024);
    let error = match WorthQueryMemoryWorkspace::collection(kind, []) {
        Err(error) => error,
        Ok(_) => panic!("oversized identity must refuse before a workspace can accept writes"),
    };
    let WorthQueryWorkspaceErrorKind::EvidenceIdentityDerivationDenied(refusal) = error.kind()
    else {
        panic!("workspace admission must preserve the digest refusal: {error:?}");
    };
    assert!(matches!(
        refusal.cause,
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { .. }
    ));
}
