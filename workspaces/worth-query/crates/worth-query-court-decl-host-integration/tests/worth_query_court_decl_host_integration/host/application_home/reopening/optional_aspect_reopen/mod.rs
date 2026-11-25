//! Directory Home courts for optional-aspect absence invalidation and no-authoring reopen projection.
mod ops;
mod program;
mod schema;

use ops::*;
use program::*;
use schema::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandProgress,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::WorthQueryOutputSettlementPosture,
    runtime::ExecutionAllocationPolicy,
};
use worth_query_testkit_document_retention::facade::{
    directory_home::{assert_native_checkpoint_fully_unchanged, unique_directory},
    document_retention_model::operator_identity::{authenticate_operator, request_scope},
};

#[test]
fn tracked_aspect_absence_survives_directory_reopen_then_invalidates_on_other_field() {
    let dir = unique_directory("optional-aspect-absence-reopen");
    let initialized = Arc::new(AtomicUsize::new(0));
    let home = open_directory(dir, initialized.clone());
    assert_eq!(initialized.load(Ordering::SeqCst), 1);

    let born_id = {
        let scope = request_scope();
        let principal = authenticate_operator(home.installed_schema(), &scope);
        let request = home
            .runtime()
            .request(&principal, &scope)
            .on_branch(home.runtime().current_world());
        let mut demand = request
            .demand(Demand("anchor-a".to_owned()))
            .start_in_program::<_, Root>(&home)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.settle(&request).unwrap()
        else {
            panic!("initial optional-aspect producer must settle");
        };
        assert_eq!(settled.producer_contacts_in_this_demand(), 1);
        settled
            .outputs_of::<InitialOutputs>()
            .unwrap()
            .entity::<BornDerived>()
            .unwrap()
            .entity_id()
    };

    let dir = home.checkpoint_and_close_for_reopen();
    let reopened = open_directory(dir, initialized.clone());
    assert_eq!(
        initialized.load(Ordering::SeqCst),
        1,
        "reopen must never re-run initial_state"
    );

    // Reopen must reuse the retained optional-aspect absence fact with 0 contacts.
    {
        let scope = request_scope();
        let principal = authenticate_operator(reopened.installed_schema(), &scope);
        let request = reopened
            .runtime()
            .request(&principal, &scope)
            .on_branch(reopened.runtime().current_world());
        let mut demand = request
            .demand(Demand("anchor-a".to_owned()))
            .start_in_program::<_, Root>(&reopened)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(retained) =
            demand.settle(&request).unwrap()
        else {
            panic!("reopened optional-aspect demand must settle from retained custody");
        };
        assert_eq!(
            retained.producer_contacts_in_this_demand(),
            0,
            "optional-aspect absence fact must survive Directory Home reopen"
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

    // Materialize OptionalSource by writing ONLY SourceContent (the OTHER field).
    // SourceSelection remains None; invalidation must fire because whole-aspect
    // presence changed, proving the absence fact was preserved across reopen.
    {
        let scope = request_scope();
        let principal = authenticate_operator(reopened.installed_schema(), &scope);
        let request = reopened
            .runtime()
            .request(&principal, &scope)
            .on_branch(reopened.runtime().current_world());
        let outcome = request
            .advanced()
            .mutate(MaterializeInput {
                key: "anchor-a".to_owned(),
            })
            .without_source()
            .idempotency(&0x0A01)
            .execute_in_program(&reopened, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "materializing other optional-aspect field must commit: {outcome:?}"
        );
    }

    {
        let scope = request_scope();
        let principal = authenticate_operator(reopened.installed_schema(), &scope);
        let request = reopened
            .runtime()
            .request(&principal, &scope)
            .on_branch(reopened.runtime().current_world());
        let mut demand = request
            .demand(Demand("anchor-a".to_owned()))
            .start_in_program::<_, Root>(&reopened)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(refreshed) =
            demand.settle(&request).unwrap()
        else {
            panic!("invalidated optional-aspect producer must settle via Preserve");
        };
        assert_eq!(
            refreshed.producer_contacts_in_this_demand(),
            1,
            "materializing the other field of an absent optional aspect must invalidate after reopen"
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

#[test]
fn reopened_projection_reads_directory_home_without_authoring_initial_state() {
    let dir = unique_directory("optional-aspect-reopen-projection");
    let initialized = Arc::new(AtomicUsize::new(0));
    let home = open_directory(dir, initialized.clone());
    assert_eq!(initialized.load(Ordering::SeqCst), 1);

    let dir = home.checkpoint_and_close_for_reopen();
    let reopened = open_directory(dir, initialized.clone());
    assert_eq!(
        initialized.load(Ordering::SeqCst),
        1,
        "reopening an existing Directory Home must never run initial_state"
    );

    let before = reopened.capture_native_checkpoint_frame();
    {
        let projection_a = reopened.home().retain_invariant_projection_authority();
        let projection_b = reopened.home().retain_invariant_projection_authority();
        reopened
            .runtime()
            .with_host_advancement(|phase| {
                let selector_a = projection_a
                    .project(
                        &phase,
                        |reader| {
                            let anchor = reader
                                .resolve_entity(AnchorKey::at("AnchorKey"), "anchor-a".to_owned())
                                .expect("reopened Directory Home resolves Anchor through retained projection");
                            assert_eq!(
                                reader.field(&anchor, AnchorKey::at("AnchorKey")),
                                Some("anchor-a".to_owned())
                            );
                            anchor
                        },
                        ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .expect("first retained projection succeeds on reopened Home")
                    .into_parts()
                    .0;
                let (selector_b, foreign_read) = projection_b
                    .project(
                        &phase,
                        |reader| {
                            let anchor = reader
                                .resolve_entity(AnchorKey::at("AnchorKey"), "anchor-a".to_owned())
                                .expect("second retained projection resolves the same Anchor");
                            let foreign = reader.field(&selector_a, AnchorKey::at("AnchorKey"));
                            (anchor, foreign)
                        },
                        ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .expect("second retained projection succeeds on reopened Home")
                    .into_parts()
                    .0;
                assert_eq!(
                    selector_a.entity_id(),
                    selector_b.entity_id(),
                    "both retained projections observe the same reopened Anchor entity"
                );
                assert_ne!(
                    selector_a, selector_b,
                    "each retained invariant projection authority owns a distinct selector namespace"
                );
                assert_eq!(
                    foreign_read, None,
                    "selector from one retained projection authority is refused by another"
                );
            })
            .expect("host advancement runs read-only invariant projections on reopened Home");

        let scope = request_scope();
        let principal = authenticate_operator(reopened.installed_schema(), &scope);
        let rows = reopened
            .runtime()
            .request(&principal, &scope)
            .on_branch(reopened.runtime().current_world())
            .query(ReadAnchor {
                key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("authenticated projection query reads the reopened anchor");
        assert_eq!(rows.rows(), &[KeyRow("anchor-a".to_owned())]);
    }
    let after = reopened.capture_native_checkpoint_frame();
    assert_native_checkpoint_fully_unchanged(&before, &after);
}
