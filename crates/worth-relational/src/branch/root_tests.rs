use std::sync::Arc;

use super::{RelationalBranchRootCaptureDenial, RelationalBranchVisibilityCommitment};
use crate::history::data::{BranchId, CommitId};
use crate::schema::data::SchemaId;
use crate::tests::support::{create_entity_outcome, runtime_with_test_schema};

#[path = "root_tests/request.rs"]
mod request;

fn committed_root_and_envelope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::runtime::RelationalRuntime,
    Arc<super::RelationalBranchRoot>,
    crate::history::data::CanonicalCommitEnvelope,
) {
    let runtime = runtime_with_test_schema();
    let performed = create_entity_outcome(execution, &runtime, "branch-root-axis-binding");
    let root = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("performed commit installs one complete branch root");
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, performed.commit.commit_id)
        .unwrap()
        .expect("performed commit owns one canonical envelope")
        .copy_original_for_test(execution);
    (runtime, root, envelope)
}

fn prepare_main_candidate(
    runtime: &crate::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    key: &str,
) -> crate::facade::mvcc::PreparedRelationalCommitCandidate {
    let services = runtime.owner_component_services();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut txn = services
        .transaction_admission_port()
        .begin_branch_transaction(
            &basis,
            crate::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    txn.push_batch(
        crate::tests::support::batch_create(key),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    services
        .preparation_port()
        .prepare_branch_transaction(execution, txn)
        .unwrap()
}

#[test]
fn publication_cost_distinguishes_new_and_reused_schema_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "schema-authority-allocation-owner");
    let first = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("first commit installs a root");
    create_entity_outcome(execution, &runtime, "schema-authority-reuse");
    let second = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("second commit installs a root");

    assert_eq!(first.publication_cost().new_schema_authorities, 1);
    assert_eq!(first.publication_cost().reused_schema_authorities, 0);
    assert_eq!(second.publication_cost().new_schema_authorities, 0);
    assert_eq!(second.publication_cost().reused_schema_authorities, 1);
    assert_eq!(
        first.schema_authority().allocation_id(),
        second.schema_authority().allocation_id()
    );
}

#[test]
fn relink_rejects_schema_authority_mutation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, root, mut envelope) = committed_root_and_envelope(execution);
    let mut parts = (*envelope.schema_authority).clone();
    parts.primary_schema_id = Some(SchemaId("mutated-schema".to_owned()));
    envelope.body_mut_for_test().schema_authority = parts.into();

    let denial = root
        .relink_canonical_envelope(
            execution,
            crate::branch::RelationalOriginalRootCommitEvidence::prepare(
                execution,
                Arc::new(envelope),
            )
            .unwrap(),
            &runtime.services.symbols.interner_snapshot(),
        )
        .expect_err("canonical schema authority mutation cannot be readmitted");

    assert!(matches!(
        denial,
        RelationalBranchRootCaptureDenial::SchemaRootMismatch { .. }
    ));
}

#[test]
fn relink_rejects_visibility_tuple_mutation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, root, mut envelope) = committed_root_and_envelope(execution);
    envelope
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .contains_opaque_aspect ^= true;

    let denial = root
        .relink_canonical_envelope(
            execution,
            crate::branch::RelationalOriginalRootCommitEvidence::prepare(
                execution,
                Arc::new(envelope),
            )
            .unwrap(),
            &runtime.services.symbols.interner_snapshot(),
        )
        .expect_err("visibility tuple mutation cannot replace the committed envelope");

    assert!(matches!(
        denial,
        RelationalBranchRootCaptureDenial::VisibilityCommitmentMismatch { .. }
    ));
}

#[test]
fn completeness_recomputes_visibility_commitment() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, root, mut altered_envelope) = committed_root_and_envelope(execution);
    assert!(root.is_complete(execution, &runtime.services.symbols.interner_snapshot()));
    altered_envelope
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .contains_opaque_aspect ^= true;
    let mut corrupted = root.as_ref().clone();
    let committed = corrupted
        .committed
        .as_mut()
        .expect("performed root owns committed axes");
    committed.axes.visibility = RelationalBranchVisibilityCommitment::for_root(
        &altered_envelope,
        committed.axes.storage_root,
        committed.axes.schema_root,
        committed.axes.correctness_index,
    );

    assert!(
        !corrupted.is_complete(execution, &runtime.services.symbols.interner_snapshot()),
        "completeness must reject a commitment to any different visible tuple"
    );
}

#[test]
fn visibility_commitment_binds_every_visible_tuple_axis() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, root, envelope) = committed_root_and_envelope(execution);
    let storage_root = [17; 32];
    let schema_root = [29; 32];
    let baseline = RelationalBranchVisibilityCommitment::for_root(
        &envelope,
        storage_root,
        schema_root,
        super::RelationalRootCorrectnessIndex::AuthoritativeFallback,
    )
    .digest();
    assert_ne!(
        production_visibility(&envelope, [18; 32], schema_root),
        baseline
    );
    assert_ne!(
        production_visibility(&envelope, storage_root, [30; 32]),
        baseline
    );

    let mut mutants = Vec::new();
    let mut commit = envelope.copy_original_for_test(execution);
    commit.body_mut_for_test().commit.commit_id =
        CommitId(commit.body_mut_for_test().commit.commit_id.0 + 1);
    mutants.push(("commit", commit));
    let mut version = envelope.copy_original_for_test(execution);
    version.body_mut_for_test().commit.version_id.0 += 1;
    mutants.push(("version", version));
    let mut parents = envelope.copy_original_for_test(execution);
    parents.body_mut_for_test().commit.parents = vec![CommitId(41), CommitId(43)];
    mutants.push(("ordered parents", parents));
    let mut parents = envelope.copy_original_for_test(execution);
    parents.body_mut_for_test().commit.parents = vec![CommitId(43), CommitId(41)];
    mutants.push(("parent order", parents));
    let mut branch = envelope.copy_original_for_test(execution);
    branch.body_mut_for_test().branch_context = BranchId("visibility-mutant".to_owned());
    mutants.push(("branch", branch));
    let mut receipt_branch = envelope.copy_original_for_test(execution);
    receipt_branch.body_mut_for_test().commit.branch_id =
        BranchId("receipt-branch-mutant".to_owned());
    mutants.push(("receipt branch", receipt_branch));
    let mut checkpoint = envelope.copy_original_for_test(execution);
    checkpoint.body_mut_for_test().branch_cell_checkpoint = None;
    mutants.push(("branch checkpoint", checkpoint));
    let mut authority_kind = envelope.copy_original_for_test(execution);
    authority_kind.body_mut_for_test().authority_kind =
        crate::history::data::CanonicalCommitAuthorityKind::BranchReferenceMovement;
    mutants.push(("authority kind", authority_kind));
    let mut merge_parents = envelope.copy_original_for_test(execution);
    merge_parents
        .body_mut_for_test()
        .merge_parent_branches
        .push(BranchId("merge-mutant".to_owned()));
    mutants.push(("merge branches", merge_parents));
    let mut merge_bases = envelope.copy_original_for_test(execution);
    merge_bases
        .body_mut_for_test()
        .merge_base_commits
        .push(CommitId(47));
    mutants.push(("merge bases", merge_bases));
    let mut schema_version = envelope.copy_original_for_test(execution);
    schema_version.body_mut_for_test().schema_version.0 += 1;
    mutants.push(("schema version", schema_version));
    let mut schema_authority = envelope.copy_original_for_test(execution);
    let mut parts = (*schema_authority.schema_authority).clone();
    parts.primary_schema_id = Some(SchemaId("visibility-mutant".to_owned()));
    schema_authority.body_mut_for_test().schema_authority = parts.into();
    mutants.push(("schema authority", schema_authority));
    let mut plan = envelope.copy_original_for_test(execution);
    plan.body_mut_for_test().merged_plan.transaction_id.0 += 1;
    mutants.push(("merged plan", plan));
    let mut allocations = envelope.copy_original_for_test(execution);
    assert!(!allocations.record_allocations().is_empty());
    allocations
        .body_mut_for_test()
        .install_record_allocations(Vec::new());
    mutants.push(("record allocations", allocations));
    let mut patch = envelope.copy_original_for_test(execution);
    patch.body_mut_for_test().patch.authoritative_record_patches[0].contains_opaque_aspect ^= true;
    mutants.push(("patch", patch));
    let mut lineage = envelope.copy_original_for_test(execution);
    lineage
        .body_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .event_id += 1;
    mutants.push(("lineage", lineage));
    let mut descriptor_semantics = envelope.copy_original_for_test(execution);
    descriptor_semantics
        .body_mut_for_test()
        .descriptor_semantics_version
        .0 += 1;
    mutants.push(("descriptor semantics", descriptor_semantics));

    for (axis, mutant) in mutants {
        assert_ne!(
            production_visibility(&mutant, storage_root, schema_root),
            baseline,
            "{axis} must change the visibility commitment"
        );
        assert!(
            root.relink_canonical_envelope(
                execution,
                crate::branch::RelationalOriginalRootCommitEvidence::prepare(
                    execution,
                    Arc::new(mutant)
                )
                .unwrap(),
                &runtime.services.symbols.interner_snapshot(),
            )
            .is_err(),
            "{axis} mutation must not relink to the committed root"
        );
    }
}

fn production_visibility(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    storage_root: [u8; 32],
    schema_root: [u8; 32],
) -> [u8; 32] {
    RelationalBranchVisibilityCommitment::for_root(
        envelope,
        storage_root,
        schema_root,
        super::RelationalRootCorrectnessIndex::AuthoritativeFallback,
    )
    .digest()
}

#[test]
fn fork_target_materialization_overflow_refuses_before_installing_target() {
    use crate::identity::data::PartitionId;
    use crate::storage::overlay::RelationalPartitionAllocationInventory as Inv;
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let (runtime, source_root, _) = committed_root_and_envelope(execution);
    let target_branch = BranchId("overflow-refused-fork".to_owned());

    let mut overflowing_root = source_root.as_ref().clone();
    let base_region = source_root.regions.values().next().unwrap();
    let inventory = |authoritative_bytes| Inv {
        authoritative_bytes,
        ..Default::default()
    };
    let mut regions = std::collections::BTreeMap::new();
    regions.insert(
        PartitionId(101),
        Arc::new(base_region.clone_with_inventory_for_test(
            99_001,
            PartitionId(101),
            inventory(u64::MAX - 8),
        )),
    );
    regions.insert(
        PartitionId(102),
        Arc::new(base_region.clone_with_inventory_for_test(
            99_002,
            PartitionId(102),
            inventory(16),
        )),
    );
    overflowing_root.regions = super::RelationalPersistentRegionSet::initial(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        77_001,
        regions,
        &super::RelationalBranchRootIdentityIssuer::default(),
    )
    .unwrap();
    let overflowing_root = Arc::new(overflowing_root);
    assert!(!overflowing_root.shares_authoritative_regions_with(&source_root));
    assert_eq!(
        overflowing_root.logical_partition_payload_bytes(),
        Err(crate::storage::substrate::StorageAllocationOverflow)
    );

    let fork_port = runtime.owner_component_services().fork_port();
    let reservation = runtime
        .history
        .fork_binding()
        .reserve_target(target_branch.clone())
        .expect("target reservation succeeds");
    let main = BranchId("main".to_owned());
    let source_cell = runtime.history.branch_cell(&main).unwrap();
    let mut target_cell = source_cell.publication_cell().enter_state().snapshot_cell();
    target_cell.install_root(overflowing_root);
    let prepared = crate::branch::fork::target_preparation::PreparedForkTarget {
        target_cell,
        source_root,
        target_observation: source_cell.observation(),
        fork_provenance: source_cell.observation(),
        target_truth_version: source_cell.truth_version(),
        shared_commit_id: None,
        source_head_version: None,
        reservation,
        indexes: None,
        managed_source: None,
    };
    let head_installs_before = runtime.retention_cost_counters().head_installs;
    let denial = fork_port
        .prepare_installation(prepared, true)
        .err()
        .expect("overflowing materialized fork must refuse before installation");
    assert_eq!(
        denial,
        crate::branch::RelationalForkDenial::AllocationAccountingOverflow
    );
    assert!(runtime.history.branch_cell(&target_branch).is_none());
    assert_eq!(
        runtime.retention_cost_counters().head_installs,
        head_installs_before,
        "refused fork must not install a head retention obligation"
    );
    runtime
        .history_authority()
        .fork_branch_from(execution, target_branch.clone(), &main)
        .expect("dropped reservation allows a clean subsequent fork");
    assert!(runtime.history.branch_cell(&target_branch).is_some());
}
