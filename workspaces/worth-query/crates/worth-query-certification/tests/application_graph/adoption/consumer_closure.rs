//! Public consumer closure for branch-selected program inspection and commit.

use crate::bounded_dimension_model::host::publish_on_first_program;
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::presented_request::{set_dimension, set_dimension_selected};
use crate::bounded_dimension_model::programs::DimensionProgramP1;
use crate::bounded_dimension_model::settled_verdict::{settle, DimensionVerdict};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::{
    WorthQueryProgramOwner, WorthQuerySelectedProgramOwnerDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionDenial;

#[test]
fn public_entry_inspects_and_executes_each_branch_under_its_carried_program() {
    let host = publish_on_first_program();
    let main = host.current_world();
    let sibling = host
        .runtime()
        .branches()
        .fork(main)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the sibling branch publishes");
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision()
        .clone();
    let source = host.owned_revision().clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);

    for branch in [main, sibling] {
        let inspection = host
            .runtime()
            .request(&principal, &scope)
            .on_branch(branch)
            .programs()
            .inspect()
            .expect("the public entry inspects its exact selected occurrence");
        assert_eq!(inspection.revision(), &source);
    }

    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(main)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("main prepares");
    assert!(matches!(
        prepared.publish(),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));

    let main_selection = host
        .runtime()
        .on_branch(main)
        .select()
        .expect("main selects its exact occurrence");
    let sibling_selection = host
        .runtime()
        .on_branch(sibling)
        .select()
        .expect("sibling selects its exact occurrence");
    let main_owner = host
        .selected_program_owner(&main_selection)
        .expect("main resolves its installed P1 owner");
    let sibling_owner = host
        .selected_program_owner(&sibling_selection)
        .expect("sibling resolves its installed P0 owner");
    assert_eq!(main_owner.owned_revision(), &target);
    assert_eq!(sibling_owner.owned_revision(), &source);
    assert_eq!(
        settle(set_dimension(&main_owner, main, 15, 0x9175_5001)),
        DimensionVerdict::Performed(15)
    );
    assert_eq!(
        settle(set_dimension(&sibling_owner, sibling, 3, 0x9175_5002)),
        DimensionVerdict::Performed(3)
    );
}

#[test]
fn same_typed_first_branch_from_another_live_host_cannot_select_an_owner() {
    let host = publish_on_first_program();
    let foreign = publish_on_first_program();
    let foreign_selection = foreign
        .runtime()
        .on_branch(foreign.current_world())
        .select()
        .expect("the foreign host can select its own branch");
    assert!(matches!(
        host.selected_program_owner(&foreign_selection),
        Err(WorthQuerySelectedProgramOwnerDenial::ProductSelection(
            WorthQueryProductBranchAdmissionDenial::ForeignOwner
        ))
    ));
}

#[test]
fn selected_program_mutation_reuses_its_authorization_selection() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let observer = host.runtime().application_query_basis_observer();
    let before = observer.observe().acquisitions();

    assert_eq!(
        settle(set_dimension_selected(&host, branch, 3, 0x9175_5003)),
        DimensionVerdict::Performed(3)
    );

    let contacts = observer.observe().acquisitions() - before;
    assert_eq!(
        contacts, 5,
        "one selected mutation retains one authorization selection plus four fresh security bases"
    );
}
