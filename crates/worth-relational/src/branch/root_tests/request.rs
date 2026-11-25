//! Owner-level fork preflight.
use super::*;
use crate::tests::support::{create_entity_outcome, runtime_with_test_schema};
use worth_execution::{ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;

#[test]
fn fork_target_materialization_overflow_refuses_before_installing_target() {
    crate::branch::root::capture::with_reconstruction_fixture_for_test(|fixture_lane| {
        use crate::identity::data::PartitionId;
        use crate::storage::overlay::RelationalPartitionAllocationInventory as Inv;
        let host_request = worth_harness::facade::execution_fixture::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        let runtime = runtime_with_test_schema();
        create_entity_outcome(execution, &runtime, "branch-root-axis-binding");
        let source_root = runtime
            .history
            .branch_cell(&BranchId("main".into()))
            .unwrap()
            .root()
            .unwrap();
        let target_branch = BranchId("overflow-refused-fork".to_owned());

        let mut overflowing_root = source_root.as_ref().clone();
        let base_region = source_root.regions_for_test().values().next().unwrap();
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
        overflowing_root.replace_regions_for_test(
            crate::branch::root_regions::RelationalPersistentRegionSet::initial(
                fixture_lane,
                77_001,
                regions,
                &super::RelationalBranchRootIdentityIssuer::default(),
            )
            .unwrap(),
        );
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
            crate::branch::fork::RelationalForkDenial::AllocationAccountingOverflow
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
    })
}
