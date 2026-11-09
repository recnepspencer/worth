//! Corrupt one recorded header fact after genuine Native admission and publication.
mod cascade;
mod journey;
mod restoration;
mod scope;
use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    history::OutputOriginHeaderCorruptionForTest as Corruption,
    identity::{EntityId, KindId, PartitionId},
    publication::AuthoredRetirementRefusal,
    symbols::ClientKey,
    transactions::{CreatedEntityRef, RecordRef, TransactionCommitError},
};

fn check(restoration: bool, detail: &str, corruption: Corruption) {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    let world = journey::run(execution);
    let runtime = &world.runtime;
    let basis = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let retired = &world.retired.publication().envelope;
    let json = serde_json::from_str::<serde_json::Value>(
        &serde_json::to_string(retired.as_ref()).unwrap(),
    )
    .unwrap();
    let co_anchors = json["authored_cascade_retirement"]["co_anchors"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, pair)| {
            (
                serde_json::from_value::<EntityId>(pair[0].clone()).unwrap(),
                target(&format!("inverse-co-{i}")),
            )
        })
        .collect::<Vec<_>>();
    let admit = || {
        runtime.prepare_authored_source_restoration_with_co_anchors(
            execution,
            &world.installed,
            &basis,
            world.retired.commit.commit_id,
            (world.source, target("inverse-anchor")),
            &co_anchors,
        )
    };
    // A fresh lawful twin for every individual corruption; setup failures are distinct.
    assert!(admit().expect("lawful inverse admission").is_some());
    let envelope = if restoration {
        &world.restored.publication().envelope
    } else {
        retired
    };
    let header = serde_json::from_str::<serde_json::Value>(
        &serde_json::to_string(envelope.as_ref()).unwrap(),
    )
    .unwrap();
    let event = &header[if restoration {
        "authored_cascade_restoration"
    } else {
        "authored_cascade_retirement"
    }];
    // Historical validation names the first publication's source, including
    // the source fact deliberately corrupted by these four source courts.
    let subject = match corruption {
        Corruption::ForeignSource => {
            serde_json::from_value(event["original_anchor"].clone()).unwrap()
        }
        Corruption::CyclicSource | Corruption::CascadeCyclicSource => {
            serde_json::from_value(event["publications"][0]["outputs"][0].clone()).unwrap()
        }
        Corruption::CascadeForeignSource => {
            let id: EntityId =
                serde_json::from_value(event["publications"][0]["members"][0]["id"].clone())
                    .unwrap();
            EntityId::new(
                id.partition_id,
                id.local_slot_value(),
                id.generation_value() + 1024,
            )
        }
        _ => world.source,
    };
    runtime
        .history_authority()
        .with_corrupt_commit_envelope_for_test(
            execution,
            envelope.commit.commit_id,
            corruption,
            || {
                let error =
                    admit().expect_err("corrupt header must refuse before issuing inverse custody");
                let TransactionCommitError::Publication { error, .. } = error else {
                    panic!("wrong refusal: {error:?}")
                };
                assert_eq!(
                    error.authored_retirement_refusal(),
                    Some(AuthoredRetirementRefusal::CorruptPublication)
                );
                assert_eq!(error.context.affected_records, [RecordRef::Entity(subject)]);
                assert_eq!(error.detail.rsplit(": ").next().unwrap(), detail);
                assert_eq!(
                    runtime
                        .observe_branch(&runtime.main_branch_identity())
                        .unwrap()
                        .1
                        .descriptor(),
                    basis.descriptor(),
                    "refusal must not publish any effect"
                );
            },
        );
    assert!(admit().expect("original history restored").is_some());
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}
fn target(key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
    }
}
