//! Fork/retirement exclusion courts for branch-carried program truth.

use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::domain::WorthQueryProgramSupportRetirementDenial;
use worth_query_host::facade::product::{
    WorthQueryProductBranchCreateError, WorthQueryProductBranchCreationDenial,
};

use super::support_retirement::adopt;
use crate::bounded_dimension_model::host::publish_on_first_program;
use crate::bounded_dimension_model::programs::DimensionProgramP1;

#[test]
fn retirement_barrier_refuses_a_new_fork_of_that_program() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let source = host.owned_revision().clone();

    let result = host
        .runtime()
        .with_program_retirement_barrier_for_test(&source, || {
            host.runtime()
                .branches()
                .fork(branch)
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create()
        });
    assert!(matches!(
        result,
        Err(WorthQueryProductBranchCreateError::Creation(
            WorthQueryProductBranchCreationDenial::ProgramSupportUnavailable
        ))
    ));
}

#[test]
fn in_flight_fork_reservation_blocks_retirement_until_its_terminal() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let source = host.owned_revision().clone();

    host.runtime()
        .with_program_fork_reservation_for_test(&source, || {
            assert!(matches!(
                host.retire_program_support(&source),
                Err(WorthQueryProgramSupportRetirementDenial::InventoryUnavailable(_))
            ));
        });
    host.runtime()
        .branches()
        .fork(branch)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the admitted fork publishes after the in-flight reservation clears");

    let denial = host
        .retire_program_support(&source)
        .expect_err("both live P0 branches remain truthful retirement users");
    let WorthQueryProgramSupportRetirementDenial::CurrentBranches(inventory) = denial else {
        panic!("the completed fork must become ordinary branch inventory")
    };
    assert_eq!(inventory.current_branches(), 2);
}

#[test]
fn adopted_source_forks_with_world_carried_program_after_old_support_retires() {
    let host = publish_on_first_program();
    let source_branch = host.current_world();
    let source_program = host.owned_revision().clone();
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    adopt(&host, source_branch, &target);
    let child = host
        .runtime()
        .branches()
        .fork(source_branch)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the adopted source forks under P1");

    host.retire_program_support(&source_program)
        .expect("no branch or retained interpretation still uses P0");
    let selected = host
        .runtime()
        .on_branch(child)
        .select()
        .expect("the child remains selectable after P0 retirement");
    assert_eq!(
        selected.inspect_selected_program().unwrap().revision(),
        &target
    );
    drop(selected);
    let grandchild = host
        .runtime()
        .branches()
        .fork(child)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the P1 child still forks after P0 retirement");
    let grandchild = host
        .runtime()
        .on_branch(grandchild)
        .select()
        .expect("the grandchild carries a selectable P1 occurrence");
    assert_eq!(
        grandchild.inspect_selected_program().unwrap().revision(),
        &target
    );
}
