use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod batch_construction;

pub use batch_construction::WorthQueryBatchMutationEvidence;

use worth_runtime_bridge::facade::BridgeBatchMutationAuthorityBundle;

use super::batch_digest_helpers::{
    batch_continuity_mutation_digest, batch_existing_truth_assertion_digest,
    batch_existing_truth_binding_digest, batch_naming_mutation_digest,
    batch_symbolic_resolution_digest, batch_symbolic_target_reference_digest, batch_target_digest,
};
use super::{
    binding::WorthQueryExistingTruthBindingEvidence, target::WorthQueryMutationTargetClass,
    WorthQueryExistingTruthAssertionEvidence, WorthQueryMutationTargetEvidence,
};
use crate::runtime::WorthQueryMutationEvidenceDigest;

fn summarize_existing_truth_modes(
    mutation_families: &[crate::runtime::WorthQueryMutationFamily],
    existing_truth_assertions: &[Option<WorthQueryExistingTruthAssertionEvidence>],
) -> Result<
    (
        usize,
        usize,
        usize,
        usize,
        Option<WorthQueryMutationEvidenceDigest>,
    ),
    IdentityDerivationDenial,
> {
    let mut retained_authoritative_assertion_count = 0;
    let mut backend_verified_assertion_count = 0;
    let mut backend_verified_update_count = 0;
    let mut backend_verified_delete_count = 0;
    let mode_parts = mutation_families
        .iter()
        .zip(existing_truth_assertions.iter())
        .filter_map(|(family, assertion)| {
            let assertion = assertion.as_ref()?;
            match (family, assertion.mode()) {
                (
                    crate::runtime::WorthQueryMutationFamily::Assertion,
                    crate::runtime::WorthQueryExistingTruthAssertionMode::RetainedAuthoritativeAssertion,
                ) => retained_authoritative_assertion_count += 1,
                (
                    crate::runtime::WorthQueryMutationFamily::Assertion,
                    crate::runtime::WorthQueryExistingTruthAssertionMode::BackendVerifiedAssertion,
                ) => backend_verified_assertion_count += 1,
                (
                    crate::runtime::WorthQueryMutationFamily::Update,
                    crate::runtime::WorthQueryExistingTruthAssertionMode::BackendVerifiedAssertion,
                ) => backend_verified_update_count += 1,
                (
                    crate::runtime::WorthQueryMutationFamily::Delete,
                    crate::runtime::WorthQueryExistingTruthAssertionMode::BackendVerifiedAssertion,
                ) => backend_verified_delete_count += 1,
                (
                    family,
                    mode,
                ) => {
                    panic!(
                        "invalid existing-truth assertion mode `{mode}` for mutation family `{family}`"
                    )
                }
            }
            Some(
                crate::evidence_identity::worth_query_evidence_identity(
                    crate::evidence_identity::WorthQueryEvidenceScope::MutationEvidenceAggregateDigest,
                )
                .field_shape(
                    crate::evidence_identity::WorthQueryEvidenceTag::new("role"),
                    "batch-existing-truth-mode-entry",
                )
                .field_shape(
                    crate::evidence_identity::WorthQueryEvidenceTag::new("family"),
                    family.as_str(),
                )
                .field_shape(
                    crate::evidence_identity::WorthQueryEvidenceTag::new("mode"),
                    assertion.mode().as_str(),
                )
                .field_evidence_identity(
                    crate::evidence_identity::WorthQueryEvidenceTag::new("verification"),
                    assertion.verification_evidence_identity(),
                )
                .seal(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let digest = (!mode_parts.is_empty()).then(|| {
        let identity = crate::evidence_identity::worth_query_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceScope::MutationEvidenceAggregateDigest,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("role"),
            "batch-existing-truth-mode",
        )
        .field_evidence_identity_sequence(
            crate::evidence_identity::WorthQueryEvidenceTag::new("entry"),
            mode_parts.iter(),
        )
        .seal();
        WorthQueryMutationEvidenceDigest::aggregate("batch-existing-truth-mode", identity)
    })?;
    Ok((
        retained_authoritative_assertion_count,
        backend_verified_assertion_count,
        backend_verified_update_count,
        backend_verified_delete_count,
        digest,
    ))
}

#[cfg(test)]
mod batch_tests;
