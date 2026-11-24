use super::HistorySubsystem;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::basis_admission::{
    descriptor_for_cell, issue_admitted_relational_branch_basis_with_retention,
};
use crate::branch::basis_denial::RelationalBranchBasisDenial as Denial;
use crate::branch::basis_retention::retain_component_basis;
use crate::branch::reference::RelationalBranchReferenceCell;
use crate::branch::root::RelationalBranchRoot;
use crate::history::data::coordinates::BranchId;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::{runtime_descriptor_semantics_policy, RelationalSchemaRegistry};

fn history(runtime_id: u64) -> HistorySubsystem {
    let branch = BranchId("main".into());
    let mut history = HistorySubsystem::from_main_cell(
        RelationalBranchReferenceCell::empty(0, branch.clone()).unwrap(),
    );
    history.set_runtime_instance_id(runtime_id).unwrap();
    history
        .install_branch_root(
            &branch,
            RelationalBranchRoot::empty_with_schema(
                &RelationalSchemaRegistry::new(),
                runtime_descriptor_semantics_policy().current_write_version(),
            ),
        )
        .unwrap();
    history
}

fn observe(history: &HistorySubsystem) -> AdmittedRelationalBranchBasis {
    let cell = history.branch_cell(&history.main_branch).unwrap();
    let root = cell.root().unwrap();
    issue_admitted_relational_branch_basis_with_retention(
        descriptor_for_cell(&cell.atomic_snapshot(), &root).unwrap(),
        cell.identity().clone(),
        root,
        cell.publication_cell(),
        &history.retention_binding(),
    )
    .unwrap()
}

#[test]
fn retained_basis_extension_requires_actual_current_history_owner_after_reset() {
    let mut installed = history(1);
    let instrumentation = RuntimeInstrumentation::new();
    let before = observe(&installed);
    // A runtime-id-only check would still be able to acquire through this live old owner.
    let prior_owner = installed.retention_owner.clone();
    let foreign = history(2);
    assert!(matches!(
        retain_component_basis(
            &foreign,
            foreign.runtime_instance_id,
            &instrumentation,
            &before
        ),
        Err(Denial::ForeignRuntime { .. })
    ));
    drop(
        retain_component_basis(
            &installed,
            installed.runtime_instance_id,
            &instrumentation,
            &before,
        )
        .unwrap(),
    );

    installed.reset_retention_owner_with_test_limits(64, 64);
    assert_eq!(
        installed
            .branch_cell(&installed.main_branch)
            .unwrap()
            .identity(),
        before.identity()
    );
    assert_eq!(
        before.identity().runtime_instance_id(),
        installed.runtime_instance_id
    );
    assert!(matches!(
        retain_component_basis(
            &installed,
            installed.runtime_instance_id,
            &instrumentation,
            &before
        ),
        Err(Denial::UnavailableRetainedTarget)
    ));
    let current = observe(&installed);
    assert_eq!(
        current.descriptor(),
        before.descriptor(),
        "matching metadata cannot replace actual owner custody"
    );
    drop(
        retain_component_basis(
            &installed,
            installed.runtime_instance_id,
            &instrumentation,
            &current,
        )
        .unwrap(),
    );
    drop(prior_owner);
}
