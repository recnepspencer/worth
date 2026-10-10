use std::sync::{Arc, RwLock};

use crate::harness::fixtures::BridgeHarnessFixture;
use crate::routing::canonicalization::digest_string;
use crate::writeback::{BridgeWritebackEffectClass, BridgeWritebackEffectIntent};
use worth_foundational::facade::{AspectKey, AspectValue};

use super::writeback_certification::{
    AuthorityDenialBoundaryEvidence, AuthorityDenialBoundaryFailureEvidence,
    AuthorityDenialZeroResidueProof, WritebackAdmissionBoundaryMatrix,
    WritebackAdmissionBoundaryMatrixEvidence, WritebackAuthorityDenialMatrix,
    WritebackDuplicateAuthorityMatrix, WritebackDuplicateAuthorityMatrixEvidence,
    WritebackFamilyExtensionMatrix, WritebackFamilyExtensionMatrixEvidence,
    WritebackFeedbackLoopMatrix, WritebackFeedbackLoopMatrixEvidence, WritebackMapperParityMatrix,
    WritebackMapperParityMatrixEvidence, WritebackReplayLoopIsolationMatrix,
    WritebackReplayLoopIsolationMatrixEvidence, WritebackReplayMismatchMatrix,
};
use super::*;

mod authority_denial_certification;
mod counter_snapshot;
mod duplicate_certification;
mod effect_intent;
mod family_certification;
mod feedback_loop_certification;
mod feedback_patch;
mod rejecting_authority;
mod replay_mismatch_certification;
mod runtime_building;
pub(in crate::harness::adapter::adapter_impl) mod terminal_report_export;
#[cfg(test)]
mod typed_certification_tests;

use counter_snapshot::{
    aggregate_runtime_writeback_counters, snapshot_from_counters, WritebackCounterSnapshot,
};
use effect_intent::writeback_effect_intent;
use rejecting_authority::RejectingTruthWritebackAuthority;

pub(super) enum WritebackHarnessTarget {
    DuplicateCertification,
    AuthorityDenialCertification,
    FeedbackLoopCertification,
    ReplayMismatchCertification,
    ExtensibleFamilyCertification,
    MultiFamilyAdmissionBoundaryCertification,
    CrossFamilyReplayLoopIsolationCertification,
    HostMapperParityCertification,
}

pub(super) enum WritebackHarnessExecution {
    DuplicateCertification {
        first_bundle_digest: String,
        repeated_bundle_digest: String,
        replay_bundle_digest: String,
        duplicate_authority_matrix: WritebackDuplicateAuthorityMatrix,
        counter_snapshot: WritebackCounterSnapshot,
    },
    AuthorityDenialCertification {
        failure_digest: String,
        authority_denial: WritebackAuthorityDenialMatrix,
        zero_residue_report: AuthorityDenialZeroResidueProof,
        counter_snapshot: WritebackCounterSnapshot,
    },
    FeedbackLoopCertification {
        feedback_loop_digest: String,
        feedback_route_identity: crate::facade::BridgeRouteIdentity,
        feedback_origin_matrix: WritebackFeedbackLoopMatrix,
        counter_snapshot: WritebackCounterSnapshot,
    },
    ReplayMismatchCertification {
        replay_validation_digest: String,
        replay_mismatch_matrix: WritebackReplayMismatchMatrix,
        counter_snapshot: WritebackCounterSnapshot,
    },
    ExtensibleFamilyCertification {
        family_extension_digest: String,
        family_extension_matrix: WritebackFamilyExtensionMatrix,
        counter_snapshot: WritebackCounterSnapshot,
    },
    MultiFamilyAdmissionBoundaryCertification {
        family_extension_digest: String,
        admission_boundary_matrix: WritebackAdmissionBoundaryMatrix,
        counter_snapshot: WritebackCounterSnapshot,
    },
    CrossFamilyReplayLoopIsolationCertification {
        family_extension_digest: String,
        replay_loop_matrix: WritebackReplayLoopIsolationMatrix,
        counter_snapshot: WritebackCounterSnapshot,
    },
    HostMapperParityCertification {
        family_extension_digest: String,
        mapper_parity_matrix: WritebackMapperParityMatrix,
        counter_snapshot: WritebackCounterSnapshot,
    },
}

pub(super) fn execute_writeback_request(
    runtime: &crate::harness::adapter::BridgeHarnessSession,
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    target: WritebackHarnessTarget,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WritebackHarnessExecution, BridgeHarnessError> {
    match target {
        WritebackHarnessTarget::DuplicateCertification => {
            duplicate_certification::execute_duplicate_certification(
                runtime,
                runtime_bridge,
                fixture,
                resource_request,
            )
        }
        WritebackHarnessTarget::AuthorityDenialCertification => {
            authority_denial_certification::execute_authority_denial_certification(
                runtime,
                runtime_bridge,
                fixture,
                resource_request,
            )
        }
        WritebackHarnessTarget::FeedbackLoopCertification => {
            feedback_loop_certification::execute_feedback_loop_certification(
                runtime,
                runtime_bridge,
                fixture,
                resource_request,
            )
        }
        WritebackHarnessTarget::ReplayMismatchCertification => {
            replay_mismatch_certification::execute_replay_mismatch_certification(
                runtime,
                runtime_bridge,
                fixture,
                resource_request,
            )
        }
        WritebackHarnessTarget::ExtensibleFamilyCertification => {
            family_certification::execute_extensible_family_certification(
                runtime,
                runtime_bridge,
                fixture,
                resource_request,
            )
        }
        WritebackHarnessTarget::MultiFamilyAdmissionBoundaryCertification => {
            family_certification::execute_multi_family_admission_boundary_certification(
                runtime,
                runtime_bridge,
                fixture,
                resource_request,
            )
        }
        WritebackHarnessTarget::CrossFamilyReplayLoopIsolationCertification => {
            family_certification::execute_cross_family_replay_loop_isolation_certification(
                runtime,
                runtime_bridge,
                fixture,
                resource_request,
            )
        }
        WritebackHarnessTarget::HostMapperParityCertification => {
            family_certification::execute_host_mapper_parity_certification(
                runtime,
                runtime_bridge,
                fixture,
                resource_request,
            )
        }
    }
}
fn find_execution_record_for_replay(
    records: &[crate::writeback::BridgeWritebackExecutionRecord],
    replay_bundle_digest: &str,
) -> Option<crate::writeback::BridgeWritebackExecutionRecord> {
    records
        .iter()
        .rev()
        .find(|record| record.replay_bundle_digest() == Some(replay_bundle_digest))
        .cloned()
}

fn find_replay_record(
    records: &[crate::writeback::BridgeWritebackReplayRecord],
    expected_replay_digest: &str,
    replayed_replay_digest: &str,
) -> Option<crate::writeback::BridgeWritebackReplayRecord> {
    records
        .iter()
        .rev()
        .find(|record| {
            record.expected_replay_digest() == expected_replay_digest
                && record.replayed_replay_digest() == replayed_replay_digest
        })
        .cloned()
}

fn lowered_policy(
    runtime_bridge: &crate::facade::RuntimeBridge,
) -> Result<crate::facade::LoweredBridgeExecutionPolicy, BridgeHarnessError> {
    let contract = runtime_bridge
        .admit_policy_declaration(crate::facade::BridgePolicyDeclaration::new(
            crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
                "harness:writeback-policy",
            ),
            crate::facade::BridgeRequestKind::Authoritative,
            crate::facade::BridgeExecutionPolicyClass::DeterministicCanonical,
            crate::facade::BridgeDiagnosticsTier::Standard,
            true,
            true,
        ))
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback harness failed to admit canonical authoritative policy: {error:?}"
            ))
        })?;
    Ok(runtime_bridge.lower_admitted_policy(&contract))
}

fn route_digest_for_first_patch(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<String, BridgeHarnessError> {
    let commit_identity = fixture
        .committed_patches()
        .first()
        .map(|patch| patch.commit_identity().clone())
        .ok_or_else(|| {
            BridgeHarnessError::new("writeback harness fixture requires one committed patch")
        })?;
    let result = runtime_bridge
        .deliver_invalidation(
            runtime_bridge
                .plan_committed_patch(
                    crate::facade::BridgeRouteRequest::for_commit(commit_identity.clone()),
                    execution,
                )
                .map_err(|error| {
                    BridgeHarnessError::new(format!(
                        "writeback harness failed to plan committed patch `{}`: {error}",
                        commit_identity.as_str()
                    ))
                })?,
            execution,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback harness failed to deliver committed patch `{}`: {error}",
                commit_identity.as_str()
            ))
        })?;
    Ok(digest_string(
        "bridge-writeback-route-digest",
        result.result_summary().route_identity().as_str(),
    )
    .to_string())
}

fn route_identity_for_commit(
    runtime_bridge: &crate::facade::RuntimeBridge,
    commit_identity: crate::facade::TruthCommitIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<crate::facade::BridgeRouteIdentity, BridgeHarnessError> {
    let result = runtime_bridge
        .deliver_invalidation(
            runtime_bridge
                .plan_committed_patch(
                    crate::facade::BridgeRouteRequest::for_commit(commit_identity.clone()),
                    execution,
                )
                .map_err(|error| {
                    BridgeHarnessError::new(format!(
                        "writeback harness failed to plan committed patch `{}`: {error}",
                        commit_identity.as_str()
                    ))
                })?,
            execution,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback harness failed to deliver committed patch `{}`: {error}",
                commit_identity.as_str()
            ))
        })?;
    Ok(result.result_summary().route_identity().clone())
}

#[derive(Clone, Copy)]
enum WritebackHarnessErrorDigestDomain {
    AuthorityDenial,
    MergeAuthorityDenial,
    UnsafeFeedbackDenial,
    ContradictoryFeedbackDenial,
}

impl WritebackHarnessErrorDigestDomain {
    fn digest_domain(self) -> &'static str {
        match self {
            Self::AuthorityDenial => "bridge-writeback-harness-authority-denial-failure",
            Self::MergeAuthorityDenial => "bridge-writeback-harness-merge-authority-denial-failure",
            Self::UnsafeFeedbackDenial => "bridge-writeback-harness-unsafe-feedback-denial-failure",
            Self::ContradictoryFeedbackDenial => {
                "bridge-writeback-harness-contradictory-feedback-denial-failure"
            }
        }
    }
}

fn writeback_harness_error_digest(
    domain: WritebackHarnessErrorDigestDomain,
    kind: impl std::fmt::Debug,
    error: impl std::fmt::Display,
) -> String {
    let digest_basis = format!("{kind:?}|{error}");
    digest_string(domain.digest_domain(), &digest_basis).to_string()
}

fn writeback_causality_basis(
    identity: impl Into<String>,
    truth_trigger_basis: impl Into<String>,
    route_basis: impl Into<String>,
    evaluation_basis: impl Into<String>,
    truth_view_basis: impl Into<String>,
) -> crate::facade::BridgeWritebackNativeCausalityInputs {
    crate::facade::BridgeWritebackNativeCausalityInputs::new(
        crate::facade::BridgeWritebackCausalityIdentity::admit_bridge_owned(identity.into()),
        crate::truth_identity_fixtures::truth_commit_fixture(truth_trigger_basis.into()),
        crate::facade::BridgeRouteIdentity::admit_bridge_owned(route_basis.into()),
        crate::truth_identity_fixtures::truth_snapshot_fixture(evaluation_basis.into()),
        crate::truth_identity_fixtures::truth_snapshot_fixture(truth_view_basis.into()),
    )
}

fn authority_denial_causality(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    identity: &'static str,
    commit_identity: crate::facade::TruthCommitIdentity,
    evidence_class: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<crate::facade::BridgeWritebackNativeCausalityInputs, BridgeHarnessError> {
    let route_identity =
        route_identity_for_commit(runtime_bridge, commit_identity.clone(), resource_request)?;
    let truth_view_basis = fixture
        .snapshots()
        .first()
        .map(|snapshot| snapshot.identity().as_str())
        .unwrap_or(evidence_class);
    Ok(writeback_causality_basis(
        identity,
        commit_identity.as_str(),
        route_identity.as_str(),
        evidence_class,
        truth_view_basis,
    ))
}

use feedback_patch::{bridge_feedback_patch, feedback_context_hint};
use runtime_building::{build_writeback_runtime, build_writeback_runtime_with_custom_authority};
