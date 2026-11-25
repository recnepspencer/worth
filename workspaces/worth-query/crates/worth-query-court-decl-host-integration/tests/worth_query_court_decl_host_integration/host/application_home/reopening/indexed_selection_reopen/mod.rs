//! Directory Home court proving rebased indexed-selection facts reuse and invalidate after reopen.
mod ops;
mod program;
mod schema;

use ops::*;
use program::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandProgress,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::WorthQueryOutputSettlementPosture,
    runtime::ExecutionAllocationPolicy,
};
use worth_query_testkit_document_retention::facade::{
    directory_home::unique_directory,
    document_retention_model::operator_identity::{authenticate_operator, request_scope},
};

#[test]
fn handler_predicate_rebases_and_reuses_then_invalidates_across_directory_reopen() {
    let dir = unique_directory("indexed-selection-reopen");
    let home = open_directory(dir);

    let born_id = {
        let scope = request_scope();
        let principal = authenticate_operator(home.installed_schema(), &scope);
        let request = home
            .runtime()
            .request(&principal, &scope)
            .on_branch(home.runtime().current_world());
        let mut demand = request
            .demand(Demand("acct-1".to_owned()))
            .start_in_program::<_, Root>(&home)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.settle(&request).unwrap()
        else {
            panic!("indexed-selection producer must settle initially");
        };
        assert_eq!(settled.producer_contacts_in_this_demand(), 1);
        settled
            .outputs_of::<InitialOutputs>()
            .unwrap()
            .entity::<BornDerived>()
            .unwrap()
            .entity_id()
    };

    // Close and reopen the SAME Directory Home so the indexed selection compares
    // against the cold-rebuilt index on the reopened World.
    let dir = home.checkpoint_and_close_for_reopen();
    let reopened = open_directory(dir);

    {
        let scope = request_scope();
        let principal = authenticate_operator(reopened.installed_schema(), &scope);
        let request = reopened
            .runtime()
            .request(&principal, &scope)
            .on_branch(reopened.runtime().current_world());
        let mut demand = request
            .demand(Demand("acct-1".to_owned()))
            .start_in_program::<_, Root>(&reopened)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(retained) =
            demand.settle(&request).unwrap()
        else {
            panic!("reopened indexed-selection demand must reuse from cold index");
        };
        assert_eq!(
            retained.producer_contacts_in_this_demand(),
            0,
            "rebased indexed selection must reuse against the reopened Directory Home's rebuilt index"
        );
        assert_eq!(
            retained.posture(),
            WorthQueryOutputSettlementPosture::RecoveredPerformed
        );
        assert_eq!(
            retained
                .outputs_of::<InitialOutputs>()
                .unwrap()
                .entity::<BornDerived>()
                .unwrap()
                .entity_id(),
            born_id
        );
    }

    // Mutate AccountStatus away from "pending-membership" on the reopened Home.
    {
        let scope = request_scope();
        let principal = authenticate_operator(reopened.installed_schema(), &scope);
        let request = reopened
            .runtime()
            .request(&principal, &scope)
            .on_branch(reopened.runtime().current_world());
        let outcome = request
            .advanced()
            .mutate(UpdateStatusInput {
                key: "acct-member".to_owned(),
                status: "left-membership".to_owned(),
            })
            .without_source()
            .idempotency(&0x0B01)
            .execute_in_program(&reopened, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "changing AccountStatus on reopened Home must commit: {outcome:?}"
        );
    }

    // Re-demanding must now invalidate (FreshRequired -> Preserve runs once).
    {
        let scope = request_scope();
        let principal = authenticate_operator(reopened.installed_schema(), &scope);
        let request = reopened
            .runtime()
            .request(&principal, &scope)
            .on_branch(reopened.runtime().current_world());
        let mut demand = request
            .demand(Demand("acct-1".to_owned()))
            .start_in_program::<_, Root>(&reopened)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(refreshed) =
            demand.settle(&request).unwrap()
        else {
            panic!("changed indexed membership must invalidate and settle via Preserve");
        };
        assert_eq!(
            refreshed.producer_contacts_in_this_demand(),
            1,
            "leaving pending-membership must invalidate the rebased indexed selection after reopen"
        );
        assert_eq!(
            refreshed.posture(),
            WorthQueryOutputSettlementPosture::Performed
        );
        assert_eq!(
            refreshed
                .outputs_of::<PreserveOutputs>()
                .unwrap()
                .entity::<KeptDerived>()
                .unwrap()
                .entity_id(),
            born_id
        );
    }
}
