//! Authored bytes are independent of runtime identities and derived execution.
use crate::conditional_evaluations::{source_free_builder, Requested, VersionCompute};
use std::sync::Arc;
use worth_execution::{CancellationToken, ExecutionRequest, SerialRequest};
use worth_foundational::facade::*;
use worth_runtime_bridge::facade::*;
use worth_signal::facade::durability::SignalDefinitionTransitionArtifact;
use worth_signal::facade::{SignalConditionalArtifactReuse, SignalConditionalVersionComparator};

fn branch() -> DurableSignalBranchIdentity {
    DurableSignalBranchIdentity::from_recorded(
        DurableSignalOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([0x54; 16]).unwrap(),
            SignalOwnerSlotIdentity::from_recorded(7),
        ),
        3,
    )
}
mod cold_readmission;
mod node_capability_backing;
fn capture(execution: &ExecutionRequest<'_, '_>) -> SignalDefinitionTransitionArtifact {
    capture_for(execution, branch(), 11)
}
fn capture_for(
    execution: &ExecutionRequest<'_, '_>,
    branch: DurableSignalBranchIdentity,
    transition: u64,
) -> SignalDefinitionTransitionArtifact {
    let mut builder = Box::new(source_free_builder());
    let mut occurrences = Vec::new();
    for (ordinal, identity) in [(1, "first-definition"), (2, "second-definition")] {
        let lowering = builder
            .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
                contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
                    identity: Arc::from(identity),
                    dependency_count: 0,
                    condition_dependency_ordinals: Vec::new(),
                    condition: BridgeConditionalCondition::OnDemand,
                    dependency_comparator: SignalConditionalVersionComparator::Exact,
                    output_comparator: SignalConditionalVersionComparator::Exact,
                    artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
                }),
                location: BridgeConditionalLocation::workflow_stage("authored-stage", identity),
                dependencies: Vec::new(),
                providers: BridgeConditionalProviderSet::new()
                    .authored_trigger(Requested)
                    .authored_compute(VersionCompute),
            })
            .unwrap();
        let node = lowering.signal_node();
        occurrences.push(
            DurableSignalConditionalDefinitionOccurrence::from_recorded(
                DurableSignalNodeIdentity::from_recorded(branch, node.index(), node.generation()),
                DurableSignalConditionalOccurrence::from_recorded(
                    branch.owner(),
                    branch.ordinal() * 2 + ordinal,
                ),
            )
            .unwrap(),
        );
    }
    builder
        .capture_authored_definition(
            execution,
            SignalDefinitionTransitionIdentity::from_recorded(branch.owner(), transition),
            branch,
            &occurrences,
        )
        .unwrap()
}
fn request() -> SerialRequest {
    SerialRequest::from_policy(
        &BridgeExecutionPolicyBaseline::operational().request_policy(),
        CancellationToken::new(),
        None,
    )
}
#[test]
fn authored_conditional_bytes_match_across_fresh_runtime_and_lowering_identities() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let first = capture(&execution);
    let second = capture(&execution);
    assert_eq!(first.as_bytes(), second.as_bytes());
    let section =
        BridgeAuthoredSection::readmit_bytes(&execution, first.bridge_bytes(), branch()).unwrap();
    assert_eq!(section.conditional_definition_count(), 2);
    assert_eq!(section.branch(), branch());
}
#[test]
fn authored_bridge_codec_refuses_unknown_version_with_a_typed_protocol_reason() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let artifact = capture(&execution);
    let mut bytes = artifact.bridge_bytes().to_vec();
    bytes[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        BridgeAuthoredSection::readmit_bytes(&execution, &bytes, branch()),
        Err(BridgeAuthoredSectionDenial::UnsupportedVersion(_))
    ));
}
#[test]
fn authored_bridge_codec_refuses_the_retired_providerless_version() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let artifact = capture(&execution);
    let mut bytes = artifact.bridge_bytes().to_vec();
    bytes[8..12].copy_from_slice(&1u32.to_le_bytes());
    assert!(matches!(
        BridgeAuthoredSection::readmit_bytes(&execution, &bytes, branch()),
        Err(BridgeAuthoredSectionDenial::UnsupportedVersion(_))
    ));
}
#[test]
fn conditional_successor_capture_refuses_a_callback_without_portable_parameters() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let predecessor = capture(&execution);
    let occurrence = predecessor.conditional_occurrences().next().unwrap();
    let contract = BridgeConditionalContract::new(BridgeConditionalContractParts {
        identity: Arc::from("first-definition"),
        dependency_count: 0,
        condition_dependency_ordinals: Vec::new(),
        condition: BridgeConditionalCondition::OnDemand,
        dependency_comparator: SignalConditionalVersionComparator::Exact,
        output_comparator: SignalConditionalVersionComparator::Exact,
        artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
    });
    let location = BridgeConditionalLocation::workflow_stage("authored-stage", "first-definition");
    let callbacks = BridgeConditionalProviderSet::new()
        .authored_trigger(Requested)
        .compute(VersionCompute);
    assert!(matches!(
        BridgeAuthoredSection::capture_definition_successor(
            &execution,
            &predecessor,
            occurrence,
            &contract,
            &location,
            &callbacks,
        ),
        Err(BridgeAuthoredSectionDenial::Provider(
            BridgeAuthoredProviderDenial::MemoryOnlyRole(BridgeConditionalProviderRole::Compute)
        ))
    ));
}
#[test]
fn authored_bridge_codec_checksum_covers_the_definition_payload() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let artifact = capture(&execution);
    let mut bytes = artifact.bridge_bytes().to_vec();
    *bytes.last_mut().unwrap() ^= 1;
    assert!(matches!(
        BridgeAuthoredSection::readmit_bytes(&execution, &bytes, branch()),
        Err(BridgeAuthoredSectionDenial::ChecksumMismatch)
    ));
    let foreign = DurableSignalBranchIdentity::from_recorded(
        DurableSignalOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([0x55; 16]).unwrap(),
            branch().owner().slot(),
        ),
        branch().ordinal(),
    );
    assert!(matches!(
        BridgeAuthoredSection::readmit_bytes(&execution, artifact.bridge_bytes(), foreign),
        Err(BridgeAuthoredSectionDenial::ForeignBranch)
    ));
}
