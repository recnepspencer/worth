//! Actual filed region members remain readable after a later selection replaces them.
use super::branch_history_publications::{create_entity, update_entity};
use super::*;
use worth_relational::facade::{
    durability::{RelationalCheckpointArtifactPolicy, RelationalCheckpointMember},
    storage::RelationalStorageUnavailableCause as Cause,
};

#[test]
fn filed_external_members_survive_checkpoint_replacement_and_obey_read_limits() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let entity = create_entity(request, &runtime, "first-external-member");
    let first = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    medium.file_checkpoint(first.clone());
    let updated = update_entity(request, &runtime, entity, "replacement-external-member");
    release_test_commit_snapshot(&runtime, &updated);
    let selected_members = first
        .members()
        .iter()
        .map(|member| match member {
            RelationalCheckpointMember::New { locator, .. }
            | RelationalCheckpointMember::Carried { locator } => *locator,
        })
        .collect::<Vec<_>>();
    let second = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &selected_members,
                source: Some(medium.clone()),
            },
        )
        .unwrap();
    let (locator, expected) = first
        .members()
        .iter()
        .find_map(|member| {
            let RelationalCheckpointMember::New { locator, bytes } = member else {
                return None;
            };
            (!second.members().iter().any(|member| match member {
                RelationalCheckpointMember::New { locator: other, .. }
                | RelationalCheckpointMember::Carried { locator: other } => locator == other,
            }))
            .then_some((*locator, bytes.clone()))
        })
        .expect("the update must replace at least one actual member");
    medium.file_checkpoint(second);
    let source: std::sync::Arc<
        dyn worth_relational::facade::durability::RelationalDurableColdSource,
    > = medium.clone();
    assert_eq!(
        source.read_member(locator, 0),
        Err(Cause::LimitExceeded {
            length: expected.len() as u64,
            limit: 0
        })
    );
    assert_eq!(
        source.read_member(locator, locator.length).unwrap(),
        expected.as_ref()
    );
}
