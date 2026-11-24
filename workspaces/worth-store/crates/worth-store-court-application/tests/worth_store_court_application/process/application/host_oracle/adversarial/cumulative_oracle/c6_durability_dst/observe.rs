//! Real Store Home observation, branch payload parity, and exact retention /
//! ancestry-integrity assertions for the C6 DST.
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;
use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::transactions::TransactionCommitError;
use worth_runtime_world::facade::*;
use worth_store_testkit_application_oracle::world_home;

#[derive(Clone, Debug)]
pub(super) struct ExpectedBranch {
    pub ordinal: u64,
    pub payloads: Vec<String>,
}

pub(super) fn observe_root_branch(
    fixture: &world_home::WorldHomeFixture,
    ctx: &str,
) -> ProductBranchObservation {
    fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap_or_else(|e| panic!("{ctx}: observe_root_product_branch failed: {e:?}"))
}

pub(super) fn observe_branch_entity_payloads(
    fixture: &world_home::WorldHomeFixture,
    branch: &ProductBranchObservation,
    exec: &ExecutionRequest<'_, '_>,
    ctx: &str,
) -> Vec<String> {
    let native = fixture.home.relational().runtime();
    let snap = native
        .snapshots()
        .snapshot_for_observation(&branch.basis().relational_basis().observation())
        .unwrap_or_else(|e| panic!("{ctx}: snapshot_for_observation failed: {e:?}"));
    let view = native
        .read_truth()
        .read_snapshot(&snap, exec)
        .unwrap_or_else(|e| panic!("{ctx}: read_snapshot outer failed: {e:?}"))
        .unwrap_or_else(|| panic!("{ctx}: read_snapshot returned None"));
    let mut payloads: Vec<String> = view
        .entities()
        .iter()
        .map(|e| format!("{:?}|{:?}", e.lifecycle, e.authoritative_aspect_state))
        .collect();
    payloads.sort();
    native.snapshots().release_snapshot(&snap).unwrap();
    payloads
}

pub(super) fn observe_root_entity_payloads(
    fixture: &world_home::WorldHomeFixture,
    exec: &ExecutionRequest<'_, '_>,
    ctx: &str,
) -> Vec<String> {
    let root = observe_root_branch(fixture, ctx);
    observe_branch_entity_payloads(fixture, &root, exec, ctx)
}

pub(super) fn observe_ref_entity_payloads(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    exec: &ExecutionRequest<'_, '_>,
) -> Vec<String> {
    let main = BranchId("main".to_owned());
    let handle = runtime.branch_identity(&main).unwrap();
    let (_, basis) = runtime.observe_branch(&handle).unwrap();
    let snap = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let view = runtime
        .read_truth()
        .read_snapshot(&snap, exec)
        .unwrap()
        .unwrap();
    let mut payloads: Vec<String> = view
        .entities()
        .iter()
        .map(|e| format!("{:?}|{:?}", e.lifecycle, e.authoritative_aspect_state))
        .collect();
    payloads.sort();
    runtime.snapshots().release_snapshot(&snap).unwrap();
    payloads
}

pub(super) fn observe_root_retained_ordinals(
    fixture: &world_home::WorldHomeFixture,
    ctx: &str,
) -> Vec<u64> {
    let root = observe_root_branch(fixture, ctx);
    let trace = fixture
        .home
        .world()
        .inspection_port()
        .trace_ancestry(
            root.selected_commit().clone(),
            NonZeroUsize::new(256).unwrap(),
        )
        .unwrap_or_else(|e| panic!("{ctx}: trace_ancestry on root failed: {e:?}"));
    let mut ordinals: Vec<u64> = trace.commits().map(|c| c.identity().ordinal()).collect();
    ordinals.reverse();
    ordinals
}

pub(super) fn verify_branches_and_ancestry(
    fixture: &world_home::WorldHomeFixture,
    exec: &ExecutionRequest<'_, '_>,
    expected_root_retained_ordinals: &[u64],
    any_retired: bool,
    expected_branches: &BTreeMap<String, ExpectedBranch>,
    ctx: &str,
) {
    let installed = fixture
        .home
        .world()
        .observation_port()
        .observe_installed_product_branches(exec)
        .unwrap_or_else(|e| panic!("{ctx}: observe_installed_product_branches failed: {e:?}"));
    let actual_names: BTreeSet<String> = installed
        .branches()
        .iter()
        .map(|b| b.branch_identity().name().as_str().to_owned())
        .collect();
    let expected_names: BTreeSet<String> = expected_branches.keys().cloned().collect();
    assert_eq!(
        actual_names, expected_names,
        "{ctx}: exact installed product branch set mismatch"
    );

    let recovery = fixture.home.world().recovery_port();
    let limit = NonZeroUsize::new(256).unwrap();
    for branch in installed.branches() {
        let name = branch.branch_identity().name().as_str();
        let expected = expected_branches
            .get(name)
            .unwrap_or_else(|| panic!("{ctx}: unexpected branch {name:?}"));
        assert_eq!(
            branch.selected_commit().ordinal(),
            expected.ordinal,
            "{ctx}: branch {name:?} selected commit ordinal mismatch"
        );
        let actual_payloads = observe_branch_entity_payloads(fixture, branch, exec, ctx);
        assert_eq!(
            actual_payloads, expected.payloads,
            "{ctx}: branch {name:?} Relational entity payload parity mismatch"
        );

        let trace = fixture
            .home
            .world()
            .inspection_port()
            .trace_ancestry(branch.selected_commit().clone(), limit)
            .unwrap_or_else(|e| {
                panic!("{ctx}: trace_ancestry failed on branch {name:?} (RetiredHistoryGap): {e:?}")
            });
        let walked: Vec<CompositeCommitIdentity> =
            trace.commits().map(|c| c.identity().clone()).collect();
        assert_eq!(
            walked.first(),
            Some(branch.selected_commit()),
            "{ctx}: ancestry trace on {name:?} did not start at branch head"
        );
        if !any_retired {
            assert!(
                trace.is_complete() && !trace.crossed_retired_history(),
                "{ctx}: ancestry trace on {name:?} truncated/retired unexpectedly before retirement"
            );
        }
        let walked_retained = recovery
            .retained_commits(&walked)
            .unwrap_or_else(|e| panic!("{ctx}: retained_commits(walked) failed: {e:?}"));
        assert_eq!(
            walked_retained, walked,
            "{ctx}: ancestry trace on {name:?} included non-retained commit"
        );
        if name == "Original World Root" {
            let mut walked_ords: Vec<u64> = walked.iter().map(|c| c.ordinal()).collect();
            walked_ords.reverse();
            assert_eq!(
                walked_ords, expected_root_retained_ordinals,
                "{ctx}: root ancestry trace ordinals diverged from exact retained boundary"
            );
        }
    }
}

pub(super) fn assert_fork_head_ordinal(
    fork_ord: u64,
    source_ord: u64,
    native_fork: bool,
    signal_fork: bool,
    ctx: &str,
) {
    if !native_fork && !signal_fork {
        assert_eq!(
            fork_ord, source_ord,
            "{ctx}: double-ReuseExact fork must select exact source commit ordinal {source_ord}, got {fork_ord}"
        );
    } else {
        assert!(
            fork_ord > source_ord,
            "{ctx}: forked component branch must allocate fresh commit ordinal > {source_ord}, got {fork_ord}"
        );
    }
}

pub(super) fn assert_fault_prepare_refusal(
    cause: &WorldNativePreparationDenial,
    indeterminate_only: bool,
    ctx: &str,
) {
    let ok = match cause {
        WorldNativePreparationDenial::Durable(_) => true,
        WorldNativePreparationDenial::Native(TransactionCommitError::DurabilityUnresolved {
            ..
        }) => true,
        WorldNativePreparationDenial::Native(TransactionCommitError::PublicationFailed {
            ..
        }) if !indeterminate_only => true,
        _ => false,
    };
    assert!(ok, "{ctx}: unexpected prepare_native refusal: {cause:?}");
}

pub(super) fn assert_fault_publish_outcome(out: &WorldDurablePublicationOutcome, ctx: &str) {
    assert!(
        matches!(
            out,
            WorldDurablePublicationOutcome::RecoveryRequired {
                cause: WorldDurablePublicationDenial::Indeterminate
                    | WorldDurablePublicationDenial::Medium(_),
                ..
            } | WorldDurablePublicationOutcome::ProductUnpublished(_)
        ),
        "{ctx}: expected typed durable fault publish outcome, got {out:?}"
    );
}
