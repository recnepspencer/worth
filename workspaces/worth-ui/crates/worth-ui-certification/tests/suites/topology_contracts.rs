//! Semantic integration suite. Individual responsibilities remain in named child modules.

use std::path::Path;
use std::sync::LazyLock;

use worth_ui_certification::topology::WorkspaceSourceInventory;

static WORKSPACE_SOURCE_INVENTORY: LazyLock<WorkspaceSourceInventory> = LazyLock::new(|| {
    WorkspaceSourceInventory::capture(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate parent")
            .parent()
            .expect("workspace root"),
    )
});

pub(crate) fn workspace_source_inventory() -> &'static WorkspaceSourceInventory {
    &WORKSPACE_SOURCE_INVENTORY
}

pub(crate) fn repository_document(relative_path: &str) -> String {
    let repository_root = workspace_source_inventory()
        .root()
        .parent()
        .and_then(Path::parent)
        .expect("workspace sits below the repository root");
    std::fs::read_to_string(repository_root.join(relative_path))
        .unwrap_or_else(|error| panic!("{relative_path} should be readable: {error}"))
}

#[test]
fn topology_suite_uses_one_stable_workspace_source_inventory() {
    let first = workspace_source_inventory();
    let second = workspace_source_inventory();

    assert!(std::ptr::eq(first, second));
    assert!(first.rust_file_count() > 0);
}

#[test]
fn hp05_certifies_the_exact_post_retirement_repository() {
    let repository_root = workspace_source_inventory()
        .root()
        .parent()
        .and_then(Path::parent)
        .expect("Worth UI workspace sits below the repository root");
    let world =
        worth_ui_certification::topology::HostRetirementTopologyWorld::capture(repository_root);
    world
        .certify()
        .unwrap_or_else(|failure| panic!("{failure}"));
    worth_ui_certification::topology::HostRetirementTopologyWorld::assert_hiding_mutations_rejected(
    );
}

#[path = "../admission_boundary_bypass.rs"]
mod admission_boundary_bypass;
#[path = "../admission_topology_audit.rs"]
mod admission_topology_audit;
#[path = "../allocation_planning_boundary_certification.rs"]
mod allocation_planning_boundary_certification;
#[path = "../appearance_owner_topology.rs"]
mod appearance_owner_topology;
#[path = "../application_authority_topology.rs"]
mod application_authority_topology;
#[path = "../declaration_residue_audit.rs"]
mod declaration_residue_audit;
#[path = "../declaration_topology_audit.rs"]
mod declaration_topology_audit;
#[path = "../executable_equivalence_topology_audit.rs"]
mod executable_equivalence_topology_audit;
#[path = "../graph_mutation_boundary_audit.rs"]
mod graph_mutation_boundary_audit;
#[path = "../graph_residue_audit.rs"]
mod graph_residue_audit;
#[path = "../graph_topology_authority_runtime.rs"]
mod graph_topology_authority_runtime;
#[path = "../graph_topology_runtime.rs"]
mod graph_topology_runtime;
#[path = "../inspection_boundary_bypass.rs"]
mod inspection_boundary_bypass;
#[path = "../inspection_contract_audit.rs"]
mod inspection_contract_audit;
#[path = "../inspection_growth_posture_audit.rs"]
mod inspection_growth_posture_audit;
#[path = "../lane_extension_authority_topology_audit.rs"]
mod lane_extension_authority_topology_audit;
#[path = "../measurement_boundary_purity_audit.rs"]
mod measurement_boundary_purity_audit;
#[path = "../measurement_growth_posture_audit.rs"]
mod measurement_growth_posture_audit;
#[path = "../milestone_3141_phase1_topology.rs"]
mod milestone_3141_phase1_topology;
#[path = "../obligation_boundary_bypass.rs"]
mod obligation_boundary_bypass;
#[path = "../obligation_boundary_residue_audit.rs"]
mod obligation_boundary_residue_audit;
#[path = "../obligation_selection_topology_runtime.rs"]
mod obligation_selection_topology_runtime;
#[path = "../query_reporting_projection_boundary.rs"]
mod query_reporting_projection_boundary;
#[path = "../responsibility_directory_topology.rs"]
mod responsibility_directory_topology;
#[path = "../runtime_diagnostic_family_mapping_audit.rs"]
mod runtime_diagnostic_family_mapping_audit;
#[path = "../topology_audit.rs"]
mod topology_audit;
