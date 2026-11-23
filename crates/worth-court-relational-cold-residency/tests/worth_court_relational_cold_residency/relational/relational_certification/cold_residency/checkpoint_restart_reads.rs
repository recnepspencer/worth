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
    let initial_bounds = recovery
        .checkpoint()
        .unwrap()
        .checkpoint()
        .inspect_history_fold_bounds(execution, Some(archive.clone()))
        .unwrap()
        .expect("external-member checkpoint carries a directory");
    let cycles = initial_bounds.max_layers_limit * 4;
    let mut consolidations = 0usize;
    let mut previous_depth = 0usize;
    for _ in 0..cycles {
        let runtime = recover(execution, copy_transport(&recovery, archive.clone()));
        let mut reader = begin_main_transaction(&runtime);
        let expected = reader
            .read_entity(execution, entity)
            .unwrap()
            .effective()
            .cloned()
            .unwrap();
        // Mirror Head is a separate carrier; warming it via public lineage
        // record-history resolution must admit mirror Head without admitting
        // branch or mirror Full history.
        let main_branch = worth_relational::facade::history::BranchId("main".into());
        let _ = runtime
            .inspect_what_happened()
            .resolve_lineage_record_history(execution, &main_branch, entity)
            .unwrap();
        let branch_residency = runtime
            .observe_branch_region_residency(&main_branch, entity.partition_id)
            .unwrap();
        assert_eq!(
            branch_residency.head,
            RelationalRegionProbeStatus::Resident,
            "branch Head must be resident before capturing with prior Heads excluded"
        );
        assert_eq!(
            branch_residency.full,
            RelationalRegionProbeStatus::Cold,
            "warming branch and mirror Heads must not demand branch history"
        );
        let mirror_residency = runtime.observe_mirror_region_residency(entity.partition_id);
        assert_eq!(
            mirror_residency.head,
            RelationalRegionProbeStatus::Resident,
            "mirror Head must be resident before capturing with prior Heads excluded"
        );
        assert_eq!(
            mirror_residency.full,
            RelationalRegionProbeStatus::Cold,
            "warming mirror Head must not demand mirror history"
        );
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
        let bounds = native
            .inspect_history_fold_bounds(execution, Some(archive.clone()))
            .unwrap()
            .expect("external-member checkpoint carries a directory");
        assert!(bounds.all_within_bounds);
        assert!(bounds.max_layers < bounds.max_layers_limit + 1);
        assert!(bounds.max_input_bytes < bounds.input_target_bytes);
        consolidations += usize::from(bounds.max_layers < previous_depth);
        previous_depth = bounds.max_layers;
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
    assert!(
        consolidations >= 2,
        "real runtime capture must repeatedly consolidate"
    );
}
