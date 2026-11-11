//! Actual repeated recapture, restart and retention preserve original Full read meaning.
use super::{
    immutable_recovery_input::{copy_input, copy_transport},
    selected_directory,
};
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{durability::*, runtime::RelationalRuntime};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction, cold_recovery::source_with_history,
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};
fn recover(
    request: &ExecutionRequest<'_, '_>,
    input: RelationalDurableRecoverySource,
) -> RelationalRuntime {
    let medium = TestDurableMedium::new();
    let mut runtime = durable_runtime(&medium);
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    runtime
}
#[test]
fn checkpoint_restart_cycles_preserve_full_reads_and_retention() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = source_with_history(execution, 3, false);
    let entity = original.entity;
    let (mut recovery, mut archive) = copy_input(&original);
    // Original loop cardinality is eight policy layers times four, without a cost claim.
    for _ in 0..32 {
        let runtime = recover(execution, copy_transport(&recovery, archive.clone()));
        let mut reader = begin_main_transaction(&runtime);
        let expected = reader
            .read_entity(execution, entity)
            .unwrap()
            .effective()
            .cloned()
            .unwrap();
        let directory = selected_directory::read(&recovery);
        let heads = directory
            .roots
            .iter()
            .flat_map(|r| &r.regions)
            .chain(&directory.mirror)
            .map(|r| r.head)
            .collect::<BTreeSet<_>>();
        let destination = archive
            .members
            .keys()
            .copied()
            .filter(|member| !heads.contains(member))
            .collect::<Vec<_>>();
        let native = runtime
            .durability_authority()
            .native_checkpoint_with_policy(
                execution,
                execution.allocation_policy(),
                RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &destination,
                    source: Some(archive.clone()),
                },
            )
            .unwrap();
        archive = archive.with_selected_members(native.members());
        recovery = RelationalDurableRecoverySource::new(
            Some(RelationalDurableCheckpointSource::new(
                native.covered_position().unwrap(),
                native.covered_branch_transitions().unwrap(),
                native,
            )),
            Vec::new(),
            Vec::new(),
            archive.clone(),
        );
        let maintenance = recover(execution, copy_transport(&recovery, archive.clone()));
        maintenance.retention().run_pass(execution).unwrap();
        let mut reader = begin_main_transaction(&maintenance);
        assert_eq!(
            reader.read_entity(execution, entity).unwrap().effective(),
            Some(&expected)
        );
    }
}
