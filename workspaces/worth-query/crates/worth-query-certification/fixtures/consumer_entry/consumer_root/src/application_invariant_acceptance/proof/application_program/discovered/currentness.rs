//! Real native aggregate guard courts; ordinary owning B delivery is separate.
use crate::application_invariant_acceptance::membership::{
    FanoutRoot, MembershipProgram, MembershipRoot, MembershipSchema,
};
use crate::application_invariant_acceptance::proof::settle;
use crate::application_invariant_acceptance::{authentication, membership, seed::length};
use std::num::NonZeroUsize;
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
        let $world = membership::install();
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
                WorthQueryOutputDemandControls::new(
                    NonZeroUsize::new(4096).unwrap(),
                    NonZeroUsize::new(8192).unwrap(),
                ),
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
                WorthQueryDiscoveredProgramOutputProgress::Pending => None,
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
fn comparison_work(
    application: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        MembershipSchema,
        MembershipProgram,
    >,
) -> NonZeroUsize {
    NonZeroUsize::new(
        application
            .runtime()
            .output_demand_resource_profile()
            .limits()
            .source_currentness_work(),
    )
    .unwrap()
}

fn retained_bytes() -> NonZeroUsize {
    NonZeroUsize::new(8192).unwrap()
}
fn stale(result: Result<(), WorthQueryOutputCurrentnessDenial>) {
    assert!(
        matches!(result, Err(WorthQueryOutputCurrentnessDenial::Output(ref cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded),
        "only moved native facts justify stale: {result:?}"
    );
}

#[test]
fn discovered_currentness_original_membership_and_empty_negative_facts() {
    for (body, count, key) in [("anchor-a", 3, 10_200u64), ("sibling-a", 0, 10_202u64)] {
        world!(world, request);
        let mut started = outputs!(world, request, body, key);
        let aggregate = settled!(world, request, started);
        assert_eq!(aggregate.root_outputs().count(), count);
        let historical = aggregate.observation();
        request
            .at(historical)
            .require_current_discovered_program_outputs(
                &aggregate,
                comparison_work(&world.application),
            )
            .unwrap();
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
                .require_current_output_demand(root, comparison_work(&world.application))
                .unwrap();
        }
        stale(
            request
                .at(&current)
                .require_current_discovered_program_outputs(
                    &aggregate,
                    comparison_work(&world.application),
                ),
        );
        request
            .at(historical)
            .require_current_discovered_program_outputs(
                &aggregate,
                comparison_work(&world.application),
            )
            .unwrap();
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
        .require_current_discovered_program_outputs(&aggregate, comparison_work(&world.application))
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
        .require_current_discovered_program_outputs(
            &aggregate,
            comparison_work(&other.application),
        );
    assert!(
        matches!(refused, Err(WorthQueryOutputCurrentnessDenial::Output(ref cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::ForeignSettlement),
        "foreign runtime cannot borrow original settlement authority: {refused:?}"
    );
    request
        .at(&current)
        .require_current_discovered_program_outputs(&aggregate, comparison_work(&world.application))
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
