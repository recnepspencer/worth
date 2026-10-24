//! D11 for the identities only output publication records: a required
//! output's producer basis and a codec retirement's source basis. Two
//! independent worlds, each with its own source and recovered target runtime,
//! publish byte-identical artifacts.
use super::fixture::*;
use crate::mvcc::RelationalOperationControl;
use crate::runtime::RelationalRuntime;

fn artifacts(runtime: &RelationalRuntime) -> Vec<Vec<u8>> {
    runtime
        .durability()
        .canonical_commit_artifacts(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
        .map(|artifact| {
            artifact
                .expect("retained commits encode")
                .as_bytes()
                .to_vec()
        })
        .collect()
}

/// Run the genuine codec transition from `world` into a recovered v2 target.
fn transitioned(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &World,
) -> RelationalRuntime {
    let (mut target, _receipt) = installed(2);
    let (_, mut recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(
            execution,
            &capture(&world.runtime),
        )
        .expect("the source checkpoint restores for transition");
    let (owner, installed) = target
        .durability_recovery()
        .issue_recovered_codec_source_publication_owner(execution, &recovered)
        .expect("the recovered source issues its publication owner");
    let selected = basis(&target);
    let proof = activation_proof(
        execution,
        &mut target,
        &recovered,
        &selected,
        &owner,
        world.activation,
    );
    let transaction = target
        .durability_recovery()
        .begin_recovered_codec_transition(
            execution,
            &mut recovered,
            &selected,
            RelationalOperationControl::uninterrupted(),
            &installed,
            proof,
            activation(world.activation),
        )
        .expect("the transition begins");
    let candidate = target
        .prepare_branch_transaction(execution, transaction)
        .expect("the transition prepares");
    let committed = target
        .durability_recovery()
        .commit_checkpoint_transition(execution, recovered, candidate)
        .expect("the transition commits");
    let (_, recovered) = committed.into_parts();
    assert!(recovered.codec_transition_retires(world.output));
    target
}

#[test]
fn output_origin_and_codec_retirement_artifacts_are_runtime_neutral() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let first = world(execution, true);
    let second = world(execution, true);
    assert_ne!(
        first.runtime.runtime_instance_id(),
        second.runtime.runtime_instance_id()
    );
    let published = artifacts(&first.runtime);
    assert!(first
        .runtime
        .history
        .positioned_canonical_commits_snapshot()
        .iter()
        .any(|positioned| positioned.envelope().required_output_origin.is_some()));
    assert!(
        published == artifacts(&second.runtime),
        "output publication artifacts differ"
    );

    let first_target = transitioned(execution, &first);
    let second_target = transitioned(execution, &second);
    assert!(first_target
        .history
        .positioned_canonical_commits_snapshot()
        .iter()
        .any(|positioned| !positioned
            .envelope()
            .required_output_codec_retirements
            .is_empty()));
    assert!(
        artifacts(&first_target) == artifacts(&second_target),
        "codec retirement artifacts differ"
    );
}
