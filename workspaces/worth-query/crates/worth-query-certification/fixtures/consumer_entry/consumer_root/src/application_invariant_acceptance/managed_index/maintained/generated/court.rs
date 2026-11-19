//! Actual producer custody reaches both scopes through generated memberships.
use super::super::super::super::authentication;
use super::super::super::lifecycle;
use super::super::{bounds, host, prefix, query, rule, schema::*};
use super::{operation, producer, program, source};
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::sync::{atomic::Ordering, Arc};
use worth_query_host::facade::{application_entry::*, primary_graph::*};
mod cancellation;
mod fallback;
mod reconstruction;

fn find(directory: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: directory.into(),
        prefix: "taken".into(),
    }
}
fn values(rows: &[query::ItemRow]) -> Vec<(String, String)> {
    let mut rows: Vec<_> = rows
        .iter()
        .map(|row| (row.identity.clone(), row.name.clone()))
        .collect();
    rows.sort();
    rows
}
// A literal authored universe independent of provider nodes and Source lookup.
fn expected(directory: &str, name: &str) -> Vec<(String, String)> {
    let mut rows = vec![("generated-item".into(), name.into())];
    if directory == "B" {
        rows.push(("y".into(), "taken".into()));
    }
    rows.sort();
    rows
}
#[test]
fn generated_catalog_suspension_preserves_logical_source_and_restoration_judges_real_candidate() {
    lifecycle::isolated_allocation_case(run, concat!(module_path!(), "::generated_catalog_suspension_preserves_logical_source_and_restoration_judges_real_candidate"));
}
fn run() {
    journey();
    let region = stats_alloc::Region::new(lifecycle::ALLOCATOR);
    journey();
    let released = region.change();
    assert_eq!(
        released.bytes_allocated, released.bytes_deallocated,
        "all actual producer, suspension, reconstruction, Source and reader owners close"
    );
}
fn journey() {
    let provider = Arc::new(prefix_observation::Calls::default());
    let rules = Arc::new(rule::Calls::new(Arc::clone(&provider)));
    let world = host::install(provider.clone(), rules.clone());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let a = request
        .query(find("A"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let performed = request
        .advanced()
        .mutate(source::operation::RequestItem {
            directory: "A".into(),
            name: "taken-generated".into(),
        })
        .without_source()
        .idempotency(&0x919700u64)
        .execute_performed::<host::Program, program::Root>(
            &world,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
        panic!("the genuine generated source action must freshly publish")
    };
    let publication = performed
        .receipt()
        .committed_product_publication()
        .composite_commit()
        .clone();
    assert_eq!(publication.owner_identity(), before.owner_identity());
    assert_eq!(
        publication.ordinal(),
        before.ordinal() + 1,
        "source and generated output co-commit"
    );
    let authored = request
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(authored.rows().len(), 1);
    assert_eq!(
        (&*authored.rows()[0].directory, &*authored.rows()[0].note),
        ("A", "taken-generated")
    );
    let produced = request.query(find("A")).execute().unwrap();
    assert_eq!(
        values(produced.rows()),
        expected("A", "taken-generated"),
        "the generated item exists before delivery"
    );
    let coordinate = produced
        .rows()
        .iter()
        .find(|row| row.identity == "generated-item")
        .unwrap()
        .coordinate;
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &publication
    );
    let handler_contacts = world.runtime().producer_contacts_on_this_thread_for_test();
    let mut output = performed
        .start_required_outputs(&world, &request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|failure| {
            panic!(
                "the actual fixed source starts its required outputs: {:?}",
                failure.denial()
            )
        });
    let settlement = (0..64)
        .find_map(|_| {
            match output
                .required_output_mut()
                .advance(&world, &request)
                .unwrap()
            {
                WorthQueryApplicationProgramOutputProgress::Settled(settlement) => Some(settlement),
                _ => None,
            }
        })
        .expect("the one real Catalog producer settles within the declared bound");
    // The delivered output is the performing writer's own co-commit. Delivery
    // hands back that original receipt and enters no handler; the work counts
    // are the receipt's own account of the one execution inside the edit.
    assert_eq!(settlement.root_producer_contacts_in_this_demand(), 0);
    assert_eq!(
        world.runtime().producer_contacts_on_this_thread_for_test(),
        handler_contacts,
        "delivery enters no producer handler"
    );
    assert_eq!(
        settlement
            .root_receipt()
            .expect("the reused committed output keeps its original receipt")
            .committed_product_publication()
            .composite_commit(),
        &publication
    );
    assert_eq!(
        (
            settlement.work().producer_contact_count(),
            settlement.work().derived_publication_count()
        ),
        (1, 1),
        "the settlement's work is the original co-commit's own explanation"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &publication
    );
    assert_eq!(
        output
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        &publication
    );
    let original = settlement
        .performed_source()
        .expect("the actual source owner remains retained");
    assert_eq!(
        original
            .receipt()
            .retained_receipt()
            .unwrap()
            .committed_product_publication()
            .composite_commit(),
        &publication
    );
    assert_eq!(original.root_fulfillment().source_commit(), &publication);
    assert_eq!(original.root_fulfillment().output_commit(), &publication);
    assert_eq!(settlement.observation().selected_commit(), &publication);
    let roles = settlement
        .root_settlement()
        .outputs_of::<operation::Outputs>()
        .unwrap();
    let item = roles.entity::<operation::ItemOutput>().unwrap();
    let id = item.entity_id();
    assert_eq!(
        (
            id.partition_value(),
            id.local_slot_value(),
            id.generation_value()
        ),
        coordinate
    );
    assert_eq!(
        request.query(find("A")).execute().unwrap().rows(),
        produced.rows()
    );
    drop((settlement, output));
    let original = request.query(find("A")).execute().unwrap();
    assert_eq!(values(original.rows()), expected("A", "taken-generated"));
    rules.records.lock().unwrap().clear();
    let a_source = request
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let source = a_source.observed_sources()[0].clone();
    let a_scope = a_source.rows()[0].coordinate;
    let b_source = request
        .query(source::ReadDirectory {
            directory: "B".into(),
        })
        .execute()
        .unwrap();
    let b_scope = b_source.rows()[0].coordinate;
    assert_ne!(a_scope, b_scope);
    let selected = world
        .with_host_advancement(|phase| {
            world
                .on_branch(world.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    rules.lent.lock().unwrap().clear();
    let suspended = selected
        .suspend_current_generated_output::<producer::Producer>(&scope, source)
        .unwrap_or_else(|failure| panic!("real generated Catalog suspension failed: {failure:?}"));
    {
        let records = rules.records.lock().unwrap();
        assert_eq!(
            records.len(),
            2,
            "both actual scopes reach the rule despite B never opening"
        );
        for (scope, directory) in [(a_scope, "A"), (b_scope, "B")] {
            let row = records
                .iter()
                .find(|record| record.source.scope == scope)
                .expect("logical proposed Source preserves the real generated row");
            assert_eq!(
                values(&row.source.after),
                expected(directory, "taken-generated")
            );
            assert_eq!(
                values(
                    row.source
                        .before
                        .as_ref()
                        .expect("suspension reads the real available logical before")
                ),
                expected(directory, "taken-generated")
            );
        }
    }
    {
        // The logical observation is unchanged: Native lends no changed row
        // on either side, though the physical Source is suspended.
        let lent = rules.lent.lock().unwrap();
        for scope in [a_scope, b_scope] {
            let (_, sides) = lent
                .iter()
                .find(|(at, _)| *at == scope)
                .expect("each invoked scope takes the loan");
            assert_eq!(sides, &[Vec::new(), Vec::new()], "suspension lends nothing");
        }
    }
    let queries = provider.probe_validations.load(Ordering::SeqCst);
    assert_suspended(request.query(find("A")).execute());
    let suspended_position = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    assert_eq!(
        provider.probe_validations.load(Ordering::SeqCst),
        queries,
        "current suspension refusal precedes provider query"
    );
    assert_eq!(values(original.rows()), expected("A", "taken-generated"));
    rules.records.lock().unwrap().clear();
    rules.lent.lock().unwrap().clear();
    let completed = reconstruction::complete(&world, suspended, "taken");
    let failure = match world.restore_generated_output(completed, &scope) {
        Err(failure) => failure,
        Ok(_) => panic!("the exact reconstructed membership must expose B's duplicate"),
    };
    let WorthQueryGeneratedOutputRestorationFailure::Rejected { suspended, cause } = failure else {
        panic!("the actual candidate must reach the installed invariant")
    };
    assert!(
        matches!(cause, WorthQueryGeneratedOutputRestorationFailureCause::PreparationInvariantRejected { identifier, .. } if identifier == "AtMostOneTakenName")
    );
    {
        let records = rules.records.lock().unwrap();
        let b = records
            .iter()
            .find(|record| record.source.scope == b_scope)
            .expect("unopened B sees the actual invalid reconstructed name");
        assert_eq!(values(&b.source.after), expected("B", "taken"));
        assert!(b.source.before.is_none(), "the committed physical Source is genuinely suspended, never empty success or logical fallback");
        // Restoration changes B's real rows: with its committed physical
        // Source suspended, they are lent on the proposed side only.
        let lent = rules.lent.lock().unwrap();
        let (_, [committed, proposed]) = lent
            .iter()
            .find(|(at, _)| *at == b_scope)
            .expect("B takes the loan");
        assert!(
            committed.is_empty() && !proposed.is_empty(),
            "{committed:?} {proposed:?}"
        );
    }
    assert_suspended(request.query(find("A")).execute());
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &suspended_position,
        "the real rejected candidate leaves Native/World selection unchanged"
    );
    let suspended = cancellation::reject(&world, &provider, &rules, suspended);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &suspended_position
    );
    assert_suspended(request.query(find("A")).execute());
    rules.records.lock().unwrap().clear();
    let builds_before_restore = provider.builds.load(Ordering::SeqCst);
    let updates_before_restore = provider.updates.load(Ordering::SeqCst);
    let completed = reconstruction::complete(&world, suspended, "taken-generated");
    world
        .restore_generated_output(completed, &scope)
        .unwrap_or_else(|failure| {
            panic!(
                "the actual lawful reconstruction restores: {:?}",
                failure.cause()
            )
        });
    assert_eq!(
        provider.builds.load(Ordering::SeqCst),
        builds_before_restore,
        "exact actual restoration shares canonical payload without a provider build"
    );
    assert_eq!(
        provider.updates.load(Ordering::SeqCst),
        updates_before_restore,
        "exact actual restoration shares canonical payload without provider maintenance"
    );
    {
        let records = rules.records.lock().unwrap();
        assert_eq!(
            records.len(),
            2,
            "both required rules run with corresponding payloads"
        );
        for (scope, directory) in [(a_scope, "A"), (b_scope, "B")] {
            let row = records
                .iter()
                .find(|row| row.source.scope == scope)
                .unwrap();
            assert_eq!(
                values(&row.source.after),
                expected(directory, "taken-generated")
            );
            assert!(
                row.source.before.is_none(),
                "correspondence never opens committed suspension Source"
            );
        }
    }
    let builds = provider.builds.load(Ordering::SeqCst);
    for directory in ["A", "B"] {
        assert_eq!(
            values(request.query(find(directory)).execute().unwrap().rows()),
            expected(directory, "taken-generated")
        );
    }
    assert_eq!(values(original.rows()), expected("A", "taken-generated"));
    assert_eq!(provider.builds.load(Ordering::SeqCst), builds, "fresh resumed public reads use the judged canonical material without another reconstruction");
    fallback::restore_changed_unprojected_field(&world, &provider, &scope);
    a.close();
}
fn assert_suspended<T>(result: Result<T, WorthQueryApplicationRequestQueryDenial>) {
    match result {
        Err(WorthQueryApplicationRequestQueryDenial::PrincipalResolution(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryPrincipalResolutionDenialKind::Refused(
                WorthQueryPrincipalResolutionRefusalKind::BranchMaterializationSuspended
            )
        ),
        Err(denial) => panic!("wrong current read cause: {denial:?}"),
        Ok(_) => panic!("a current physical read cannot expose retained logical material"),
    }
}
