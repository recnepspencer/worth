//! Real native aggregate guard courts; ordinary owning B delivery is separate.
use crate::application_invariant_acceptance::membership::{
    FanoutRoot, MembershipProgram, MembershipRoot, MembershipSchema,
};
use crate::application_invariant_acceptance::proof::settle;
use crate::application_invariant_acceptance::{authentication, membership, seed::length};
mod unrelated;
use worth_query_consumer_values::{PlanarAdjustment, PlanarOperation};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationDiscoveredMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryDiscoveredProgramOutputProgress, WorthQueryOutputDemandControls,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryOutputCurrentnessDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_topology_entry::{PlanarDiscoveryLink, PlanarDiscoveryRead};
use worth_query_topology_entry::{
    PlanarEdit, PlanarFinalToSummaryConnection, PlanarMutation, PlanarOutputRead,
    PlanarOutputToMembershipConnection, PlanarRead, PlanarSourceAdjustment,
};

macro_rules! world {
    ($world:ident, $request:ident) => {
        world!($world, $request, membership::install());
    };
    ($world:ident, $request:ident, $installed:expr) => {
        let $world = $installed;
        let scope = authentication::request_scope();
        let adapter = authentication::admit($world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let $request = $world.application.request(&principal, &scope);
    };
}
macro_rules! outputs {
    ($world:ident, $request:ident, $body:expr, $key:expr) => {{
        let source = $request
            .query(PlanarRead {
                body_key: $body.to_owned(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone();
        let outcome = $request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: $body.to_owned(),
                replacement_y: length(2),
            })
            .expect_source(source)
            .idempotency(&$key)
            .execute_performed_discovered::<MembershipProgram, FanoutRoot>(
                &$world.application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
            panic!("real source must publish once")
        };
        performed
            .start_required_outputs(
                &$world.application,
                &$request,
                WorthQueryOutputDemandControls::default(),
            )
            .unwrap_or_else(|failure| panic!("native source setup: {:?}", failure.denial()))
    }};
}
macro_rules! settled {
    ($world:ident, $request:ident, $started:ident) => {
        settle(|| {
            match $started
                .required_output_mut()
                .advance(&$world.application, &$request)
                .unwrap()
            {
                WorthQueryDiscoveredProgramOutputProgress::Pending
                | WorthQueryDiscoveredProgramOutputProgress::Progressed => None,
                WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => Some(settled),
            }
        })
    };
}
macro_rules! link {
    ($world:ident, $request:ident, $body:expr, $target:expr, $key:expr) => {
        assert!(
            matches!(
                $request
                    .advanced()
                    .mutate(PlanarDiscoveryLink {
                        scope_key: $body.to_owned(),
                        target_key: $target.to_owned()
                    })
                    .idempotency(&$key)
                    .execute_in_program(
                        &$world.application,
                        ExecutionAllocationPolicy::SystemAllocation
                    )
                    .unwrap(),
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "genuine membership-only mutation must land"
        )
    };
}

fn stale(result: Result<(), WorthQueryOutputCurrentnessDenial>) {
    assert!(
        matches!(result, Err(WorthQueryOutputCurrentnessDenial::Output(ref cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded),
        "only moved native facts justify stale: {result:?}"
    );
}

#[test]
fn discovered_currentness_original_membership_and_empty_negative_facts() {
    let _observations = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for (body, count, key) in [("anchor-a", 3, 10_200u64), ("sibling-a", 0, 10_202u64)] {
        world!(world, request, membership::install_with_isolated_cycle());
        // Empty capture must precede anchor preparation: sibling-a is part of
        // the later nonempty tree's genuine successor read closure.
        let mut empty_started = outputs!(world, request, "sibling-a", key + 100);
        let empty = settled!(world, request, empty_started);
        assert_eq!(empty.root_outputs().count(), 0);
        let mut full_started = outputs!(world, request, "anchor-a", key);
        let full = settled!(world, request, full_started);
        assert_eq!(full.root_outputs().count(), 3);
        let (aggregate, other) = if body == "anchor-a" {
            (full, empty)
        } else {
            (empty, full)
        };
        assert_eq!(aggregate.root_outputs().count(), count);
        let historical = aggregate.observation();
        assert_ne!(
            historical.selected_commit(),
            other.observation().selected_commit()
        );
        let common = request.retain_current_read().unwrap();
        let _ =
            worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test();
        for tree in [&aggregate, &other] {
            request
                .at(&common)
                .require_current_discovered_program_outputs(tree)
                .unwrap();
        }
        let clean =
            worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test();
        assert_eq!(
            clean.source_currentness_visits, 0,
            "clean original discovery and both complete trees: {clean:?}"
        );
        println!("clean discovery body={body} roots={count}: {clean:?}");

        unrelated::assert_initial(&request);
        for (step, next_value) in [62, 61, 62].into_iter().enumerate() {
            unrelated::publish(
                &world.application,
                &request,
                key + 1000 + step as u64,
                next_value,
            );
            let selected = request.retain_current_read().unwrap();
            let _ =
                worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test(
                );
            for _ in 0..2 {
                for tree in [&aggregate, &other] {
                    request
                        .at(&selected)
                        .require_current_discovered_program_outputs(tree)
                        .unwrap();
                }
            }
            let facts =
                worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test(
                );
            assert_eq!(
                facts.source_currentness_visits, 0,
                "clean continuity step={step}: {facts:?}"
            );
        }
        let before = request
            .query(PlanarDiscoveryRead {
                body_key: body.to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .source_body_keys
            .clone();
        let sibling = request
            .query(PlanarRead {
                body_key: "sibling-a".to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y;
        link!(world, request, body, "anchor-b", key + 1);
        let after = request
            .query(PlanarDiscoveryRead {
                body_key: body.to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .source_body_keys
            .clone();
        assert_eq!(after.len(), before.len() + 1);
        assert!(after.iter().any(|key| key == "anchor-b"));
        assert_eq!(
            request
                .query(PlanarRead {
                    body_key: "sibling-a".to_owned()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .y,
            sibling
        );
        let current = request.retain_current_read().unwrap();
        // Surviving root lineage alone still passes. Only original membership moved.
        for (_, root) in aggregate.root_outputs() {
            request
                .at(&current)
                .require_current_output_demand(root)
                .unwrap();
        }
        let _ =
            worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test();
        stale(
            request
                .at(&current)
                .require_current_discovered_program_outputs(&aggregate),
        );
        let dirty =
            worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test();
        assert!(
            dirty.source_currentness_visits > 0,
            "matching discovery touch takes genuine full comparison: {dirty:?}"
        );
        println!("dirty discovery body={body}: {dirty:?}");
        request
            .at(historical)
            .require_current_discovered_program_outputs(&aggregate)
            .unwrap();
        // An unrelated native publication preserves the original trees' coverage.
        unrelated::publish(&world.application, &request, key + 10, 63);
        let full_image = request.retain_current_read().unwrap();
        let _ =
            worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test();
        request
            .at(&full_image)
            .require_current_discovered_program_outputs(
                &other)
            .expect(
                "unchanged second original tree and discovery remain current without resource estimates",
            );
        let full =
            worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test();
        assert_eq!(
            full.source_currentness_visits, 0,
            "unrelated publication preserves native coverage: {full:?}"
        );
        assert_eq!(
            world.application.retained_source_custody_count_for_test(),
            0
        );
    }
}

#[test]
fn discovered_currentness_foreign_runtime_preserves_owner() {
    world!(world, request);
    let mut started = outputs!(world, request, "anchor-a", 10_204u64);
    let aggregate = settled!(world, request, started);
    let current = request.retain_current_read().unwrap();
    request
        .at(&current)
        .require_current_discovered_program_outputs(&aggregate)
        .unwrap();
    let other = membership::install();
    let adapter = authentication::admit(other.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let other_request = other.application.request(&principal, &scope);
    let other_read = other_request.retain_current_read().unwrap();
    let refused = other_request
        .at(&other_read)
        .require_current_discovered_program_outputs(&aggregate);
    assert!(
        matches!(refused, Err(WorthQueryOutputCurrentnessDenial::Output(ref cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::ForeignSettlement),
        "foreign runtime cannot borrow original settlement authority: {refused:?}"
    );
    request
        .at(&current)
        .require_current_discovered_program_outputs(&aggregate)
        .unwrap();
    for (key, expected) in [("sibling-b", 22), ("sibling-c", 31), ("remote-b", 42)] {
        assert_eq!(
            request
                .query(PlanarOutputRead {
                    body_key: key.to_owned()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            length(expected)
        );
    }
}

mod dependent;
mod retired;

mod deep_output;
