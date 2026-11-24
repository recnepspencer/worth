//! Verify durable schema continuity against the receiver's declared support.
use crate::durability::data::RecoveryAuthorityContinuityCheck;
use crate::durability::data::RecoveryAuthorityParity;
use crate::durability::data::RecoveryVerificationOutcome;
use crate::replay::data::ReplayVerificationLayer;
use branch::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use branch::schema::continuity::bundle_admission::validate_schema_continuity_bundle;
use contracts::durability::data::recovery_errors::RecoveryAuthorityContinuityMismatch;
use contracts::performance::data::runtime_complexity_counters::{
    RuntimeComplexityCounters, RuntimeComplexityCountersOwnerPort as _,
};
use contracts::schema::data::continuity::descriptor_versions::{
    DescriptorCanonicalBasisSupportPolicy, DescriptorSemanticsSupportPolicy,
};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

pub mod issue_mapping;

pub fn verify<'a>(
    descriptor_policy: &DescriptorSemanticsSupportPolicy,
    canonical_basis_policy: &DescriptorCanonicalBasisSupportPolicy,
    instrumentation: &RuntimeInstrumentation,
    envelopes: impl IntoIterator<
        Item = &'a branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope,
    >,
) -> RecoveryAuthorityContinuityCheck {
    let expected_descriptor_semantics_version = descriptor_policy.current_write_version();
    let expected_descriptor_canonical_basis_version =
        canonical_basis_policy.current_write_version();
    let mut authority_continuity =
        RecoveryAuthorityContinuityCheck::verified_at(ReplayVerificationLayer::DigestParity);

    for envelope in envelopes {
        if !descriptor_policy.supports(envelope.descriptor_semantics_version) {
            reject_descriptor_semantics_version(
                instrumentation,
                &mut authority_continuity,
                expected_descriptor_semantics_version,
                envelope.descriptor_semantics_version,
            );
            continue;
        }

        if let Some(found) = unsupported_canonical_basis_version(envelope, canonical_basis_policy) {
            reject_descriptor_canonical_basis_version(
                instrumentation,
                &mut authority_continuity,
                expected_descriptor_canonical_basis_version,
                found,
            );
            continue;
        }

        match validate_schema_continuity_bundle(envelope) {
            Ok(validated_bundle) => {
                instrumentation.count(RuntimeComplexityCounters::count_replay_digest_parity);
                let _ = (
                    validated_bundle.envelope(),
                    validated_bundle.transition(),
                    validated_bundle.continuation(),
                    validated_bundle.reconciliation(),
                );
            }
            Err(issue) => issue_mapping::apply_continuity_issue(
                instrumentation,
                &mut authority_continuity,
                envelope,
                issue,
            ),
        }
    }

    authority_continuity
}

fn reject_descriptor_semantics_version(
    instrumentation: &RuntimeInstrumentation,
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    expected: contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion,
    found: contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion,
) {
    instrumentation.count_descriptor_version_mismatch();
    instrumentation.count(RuntimeComplexityCounters::count_replay_digest_parity);
    authority_continuity.descriptor_version_parity = RecoveryAuthorityParity::drift();
    authority_continuity.first_mismatch.get_or_insert(
        RecoveryAuthorityContinuityMismatch::DescriptorSemanticsVersion { expected, found },
    );
    authority_continuity.verification_outcome = RecoveryVerificationOutcome::Rejected {
        layer: ReplayVerificationLayer::DigestParity,
        detail: "descriptor semantics version mismatch".to_string(),
    };
}

fn reject_descriptor_canonical_basis_version(
    instrumentation: &RuntimeInstrumentation,
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    expected: contracts::schema::data::continuity::descriptor_versions::DescriptorCanonicalBasisVersion,
    found: contracts::schema::data::continuity::descriptor_versions::DescriptorCanonicalBasisVersion,
) {
    instrumentation.count_descriptor_version_mismatch();
    instrumentation.count(RuntimeComplexityCounters::count_replay_digest_parity);
    authority_continuity.descriptor_version_parity = RecoveryAuthorityParity::drift();
    authority_continuity.first_mismatch.get_or_insert(
        RecoveryAuthorityContinuityMismatch::DescriptorCanonicalBasisVersion { expected, found },
    );
    authority_continuity.verification_outcome = RecoveryVerificationOutcome::Rejected {
        layer: ReplayVerificationLayer::DigestParity,
        detail: "descriptor canonical basis version mismatch".to_string(),
    };
}

fn unsupported_canonical_basis_version(
    envelope: &branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope,
    policy: &contracts::schema::data::continuity::descriptor_versions::DescriptorCanonicalBasisSupportPolicy,
) -> Option<contracts::schema::data::continuity::descriptor_versions::DescriptorCanonicalBasisVersion>
{
    let continuation = envelope
        .schema_continuation_descriptor
        .as_ref()
        .map(|descriptor| descriptor.bridge.canonical_basis_version);
    let reconciliation = envelope
        .schema_reconciliation_descriptor
        .as_ref()
        .map(|descriptor| descriptor.canonical_basis_version);
    continuation
        .into_iter()
        .chain(reconciliation)
        .find(|version| !policy.supports(*version))
}
