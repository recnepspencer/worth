//! Actual cold recovery, Head readmission, member capture and mirror maintenance.
use super::*;
use crate::durability::data::{
    RelationalCheckpointManifest, RelationalHistoryMemberDirectory, RelationalNativeCheckpoint,
};
use crate::storage::residency::history_capture_bound::{
    HISTORY_CAPTURE_INPUT_TARGET, HISTORY_CAPTURE_MAX_LAYERS,
};

#[derive(serde::Deserialize)]
struct Frame {
    manifest: RelationalCheckpointManifest,
}

fn manifest(native: &RelationalNativeCheckpoint) -> RelationalCheckpointManifest {
    let frame: Frame = rmp_serde::from_slice(
        native
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap();
    frame.manifest
}

#[test]
fn checkpoint_restart_cycles_preserve_full_reads_and_retention() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let (mut recovery, mut archive, entity) = fixture_with_history(execution, false, 3);
    archive.damaged.store(false, Ordering::SeqCst);
    let mut consolidations = 0;
    let mut previous_depth = 0;
    for _ in 0..(HISTORY_CAPTURE_MAX_LAYERS * 4) {
        let runtime = recover(execution, recovery.clone());
        let mut reader = test_owner_begin_transaction_for_main(&runtime);
        let expected = reader
            .read_entity(execution, entity)
            .unwrap()
            .effective()
            .cloned()
            .unwrap();
        // Mirror Head is a separate carrier; warming it must not demand history.
        runtime
            .acquire_partition_edition()
            .demand_partition_head(execution, entity.partition_id)
            .unwrap();
        let old_manifest = manifest(recovery.checkpoint().unwrap().checkpoint());
        let heads = old_manifest
            .roots
            .iter()
            .flat_map(|root| root.regions.iter())
            .chain(old_manifest.mirror.iter())
            .map(|leaf| leaf.head)
            .collect::<std::collections::BTreeSet<_>>();
        // A destination with history custody but no prior Head must encode the
        // admitted Head. Ordinary commits can already warm Full, so they cannot
        // expose the restart-only empty-prefix-layer regression.
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
        let captured = manifest(&native);
        let roots = captured
            .roots
            .iter()
            .flat_map(|root| root.regions.iter())
            .chain(captured.mirror.iter())
            .map(|leaf| leaf.history);
        let directory =
            RelationalHistoryMemberDirectory::readmit(&captured.history_dependencies, roots)
                .unwrap();
        let mut depth = 0;
        for leaf in captured
            .roots
            .iter()
            .flat_map(|root| root.regions.iter())
            .chain(captured.mirror.iter())
        {
            let (layers, input) = directory.fold_span(leaf.history);
            assert!(layers < HISTORY_CAPTURE_MAX_LAYERS + 1);
            assert!(input < HISTORY_CAPTURE_INPUT_TARGET);
            depth = depth.max(layers);
        }
        consolidations += usize::from(depth < previous_depth);
        previous_depth = depth;
        archive = place(&native, &archive);
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
        let maintenance = recover(execution, recovery.clone());
        maintenance.retention().run_pass(execution).unwrap();
        let mut reader = test_owner_begin_transaction_for_main(&maintenance);
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

fn recover(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: RelationalDurableRecoverySource,
) -> crate::runtime::RelationalRuntime {
    let mut runtime = durable_runtime_with_test_schema();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    runtime
}

fn place(native: &RelationalNativeCheckpoint, prior: &MemberSource) -> Arc<MemberSource> {
    Arc::new(MemberSource {
        members: native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
                RelationalCheckpointMember::Carried { locator } => {
                    (*locator, prior.members[locator].clone())
                }
            })
            .collect(),
        commits: prior.commits.clone(),
        damaged: AtomicBool::new(false),
        reads: AtomicUsize::new(0),
        member_reads: Default::default(),
        commit_reads: Default::default(),
        commits_damaged: AtomicBool::new(false),
        failed_member: Default::default(),
        failures: Default::default(),
    })
}
