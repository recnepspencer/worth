//! The fixture medium retains real member bytes across checkpoint decisions.
use crate::durability::data::{RelationalCheckpointArtifactPolicy, RelationalCheckpointMember};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::tests::support::*;

#[test]
fn filed_external_members_survive_checkpoint_replacement_and_obey_read_limits() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let runtime = durable_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "first-external-member");
    let capture = || {
        runtime
            .durability_authority()
            .native_checkpoint_with_policy(
                execution.allocation_policy(),
                RelationalCheckpointArtifactPolicy::ExternalMembers,
            )
            .unwrap()
    };
    let first = capture();
    attached_test_medium(&runtime).file_checkpoint(first.clone());
    update_entity_and_release_snapshot(execution, &runtime, entity, "replacement-external-member");
    let second = capture();
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
    attached_test_medium(&runtime).file_checkpoint(second);
    let source = durable_recovery_source(&runtime).into_parts().cold_source;
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
