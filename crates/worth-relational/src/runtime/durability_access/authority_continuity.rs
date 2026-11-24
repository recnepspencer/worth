use crate::capabilities::RuntimeConfigSource;
use crate::durability::data::{
    RecoveryAuthorityContinuityCheck, RecoveryAuthorityContinuityMismatch, RecoveryAuthorityParity,
    RecoveryVerificationOutcome,
};
use crate::replay::data::ReplayVerificationLayer;
use crate::runtime::durability_access::continuity_issue_mapping::apply_continuity_issue;
use crate::runtime::RelationalRuntime;
use crate::schema::{
    validate_schema_continuity_bundle, SchemaContinuityBundleIssue, ValidatedSchemaContinuityBundle,
};

pub(crate) fn authority_continuity_for_envelopes(
    runtime: &RelationalRuntime,
    checkpoint_envelopes: &[crate::history::data::PositionedCanonicalCommit],
    tail_log: &[crate::durability::migration::ReadmittedCanonicalCommit],
) -> RecoveryAuthorityContinuityCheck {
    authority_continuity_for_canonical_envelopes(
        runtime,
        checkpoint_envelopes
            .iter()
            .map(crate::history::data::PositionedCanonicalCommit::envelope)
            .chain(tail_log.iter().map(|entry| entry.envelope())),
    )
}

fn authority_continuity_for_canonical_envelopes<'a>(
    runtime: &RelationalRuntime,
    envelopes: impl IntoIterator<Item = &'a crate::history::data::CanonicalCommitEnvelope>,
) -> RecoveryAuthorityContinuityCheck {
    let descriptor_policy = runtime
        .runtime_config()
        .schema
        .descriptor_semantics_policy
        .clone();
    let canonical_basis_policy = runtime
        .runtime_config()
        .schema
        .descriptor_canonical_basis_policy
        .clone();
    let expected_descriptor_semantics_version = descriptor_policy.current_write_version();
    let expected_descriptor_canonical_basis_version =
        canonical_basis_policy.current_write_version();
    let mut authority_continuity =
        RecoveryAuthorityContinuityCheck::verified_at(ReplayVerificationLayer::DigestParity);

    for envelope in envelopes {
        if !descriptor_policy.supports(envelope.descriptor_semantics_version) {
            reject_descriptor_semantics_version(
                runtime,
                &mut authority_continuity,
                expected_descriptor_semantics_version,
                envelope.descriptor_semantics_version,
            );
            continue;
        }

        if let Some(found) = unsupported_canonical_basis_version(envelope, &canonical_basis_policy)
        {
            reject_descriptor_canonical_basis_version(
                runtime,
                &mut authority_continuity,
                expected_descriptor_canonical_basis_version,
                found,
            );
            continue;
        }

        match validated_recovery_continuity_envelope(envelope) {
            Ok(validated_bundle) => {
                runtime
                    .performance_access()
                    .count_replay_verification_layer(ReplayVerificationLayer::DigestParity);
                let _ = (
                    validated_bundle.envelope(),
                    validated_bundle.transition(),
                    validated_bundle.continuation(),
                    validated_bundle.reconciliation(),
                );
            }
            Err(issue) => {
                apply_continuity_issue(runtime, &mut authority_continuity, envelope, issue)
            }
        }
    }

    authority_continuity
}

fn reject_descriptor_semantics_version(
    runtime: &RelationalRuntime,
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    expected: crate::schema::data::DescriptorSemanticsVersion,
    found: crate::schema::data::DescriptorSemanticsVersion,
) {
    runtime
        .performance_access()
        .count_descriptor_version_mismatch();
    runtime
        .performance_access()
        .count_replay_verification_layer(ReplayVerificationLayer::DigestParity);
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
    runtime: &RelationalRuntime,
    authority_continuity: &mut RecoveryAuthorityContinuityCheck,
    expected: crate::schema::data::DescriptorCanonicalBasisVersion,
    found: crate::schema::data::DescriptorCanonicalBasisVersion,
) {
    runtime
        .performance_access()
        .count_descriptor_version_mismatch();
    runtime
        .performance_access()
        .count_replay_verification_layer(ReplayVerificationLayer::DigestParity);
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
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    policy: &crate::schema::data::DescriptorCanonicalBasisSupportPolicy,
) -> Option<crate::schema::data::DescriptorCanonicalBasisVersion> {
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

fn validated_recovery_continuity_envelope(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
) -> Result<ValidatedSchemaContinuityBundle<'_>, SchemaContinuityBundleIssue> {
    validate_schema_continuity_bundle(envelope)
}
