//! Indexed decision membership remains evidence when an upstream republishes equally.
use super::*;
use worth_query_consumer_values::{PlanarAdjustment, PlanarOperation};

#[test]
fn a_new_indexed_match_prevents_equal_upstream_reuse() {
    journey(true, false);
}

#[test]
fn unchanged_indexed_membership_reuses_an_equal_upstream() {
    journey(false, false);
}

#[test]
fn a_new_indexed_match_contacts_the_consumer_with_a_dirty_upstream() {
    journey(true, true);
}

#[test]
fn unchanged_indexed_membership_does_not_contact_over_a_dirty_upstream() {
    journey(false, true);
}

fn journey(add_match: bool, leave_dirty: bool) {
    let _guard = checkpoint_recovery_test_guard();
    let app = installation::install_with_indexed_decision();
    let (scope, principal) = installation::authentication_fixture::authenticate(&app);
    let request = app.request(&principal, &scope);
    let mut root = request
        .demand(Demand(OUTPUT))
        .start_in_program::<installation::Program, installation::Root>(&app)
        .unwrap();
    let mut consumer = request
        .demand(counted_producer::Demand)
        .start_in_program::<installation::Program, installation::DependentRoot>(&app)
        .unwrap();
    let upstream = loop {
        let (fresh_scope, fresh_principal) =
            installation::authentication_fixture::authenticate(&app);
        let fresh = app.request(&fresh_principal, &fresh_scope);
        match root.advance(&fresh).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Progressed => continue,
            WorthQueryApplicationOutputDemandProgress::Settled(settlement) => break settlement,
            WorthQueryApplicationOutputDemandProgress::Pending => {
                panic!("the initial synchronous upstream must not leave external work pending")
            }
        }
    };
    assert_eq!(upstream.producer_contacts_in_this_demand(), 1);
    assert_eq!(settle!(consumer, request).0, 1);
    let read_value = |key: &str| {
        request
            .query(PlanarOutputRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value
    };
    assert_eq!(read_value(OUTPUT), length(1));
    assert_eq!(
        read_value(DEPENDENT),
        length(1),
        "only anchor-c initially matches y = 10"
    );
    dependent_publication::take_calls();
    assert_eq!(
        dependent_publication::take_indexed_selections(),
        [1],
        "the first consumer decision sees only the seeded indexed member"
    );
    let source = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .unwrap();
    let mut edits = vec![PlanarAdjustment {
        body_key: "anchor-b".to_owned(),
        replacement_y: length(2),
    }];
    if add_match {
        // The same commit adds anchor-atoll to the complete equality result.
        edits.push(PlanarAdjustment {
            body_key: "anchor-atoll".to_owned(),
            replacement_y: length(10),
        });
    }
    let changed = request
        .advanced()
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-b".to_owned(),
            operation: PlanarOperation::Adjust(edits),
        }))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_ffee_u64)
        .execute_in_program::<installation::Program>(
            &app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(changed, worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { .. }), "one real edit commits both changes");
    let upstream = if leave_dirty {
        upstream
    } else {
        // The upstream's prepared input changes from 1 to 2, but ceil(y / 2) stays 1.
        let mut refreshed = request
            .demand(Demand(OUTPUT))
            .start_in_program::<installation::Program, installation::Root>(&app)
            .unwrap();
        let settlement = loop {
            let (fresh_scope, fresh_principal) =
                installation::authentication_fixture::authenticate(&app);
            let fresh = app.request(&fresh_principal, &fresh_scope);
            match refreshed.advance(&fresh).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Progressed => continue,
                WorthQueryApplicationOutputDemandProgress::Settled(settlement) => break settlement,
                WorthQueryApplicationOutputDemandProgress::Pending => {
                    panic!("the synchronous upstream refresh must not leave external work pending")
                }
            }
        };
        assert_eq!(
            settlement.producer_contacts_in_this_demand(),
            1,
            "U really decides again"
        );
        assert_eq!(
            read_value(OUTPUT),
            length(1),
            "U republishes an equal value"
        );
        settlement
    };
    let before_consumer = request.retain_current_read().unwrap();
    let upstream_currentness = request
        .at(&before_consumer)
        .require_current_output_demand(&upstream);
    if leave_dirty {
        assert!(
            matches!(
                &upstream_currentness,
                Err(worth_query_host::facade::application_entry::WorthQueryOutputCurrentnessDenial::Output(cause))
                    if cause.kind() == primary_graph::WorthQueryOutputDemandDenialKind::Superseded
            ),
            "the original upstream still carries the edited source: {upstream_currentness:?}"
        );
    } else {
        upstream_currentness.expect("the explicitly refreshed equal upstream is already current");
    }
    // Native requested-output progression may refresh U and retry the caller
    // internally. A stage handoff keeps this original demand under fresh authority.
    let settlement = loop {
        let (fresh_scope, fresh_principal) =
            installation::authentication_fixture::authenticate(&app);
        let fresh = app.request(&fresh_principal, &fresh_scope);
        match consumer.advance(&fresh) {
            Ok(WorthQueryApplicationOutputDemandProgress::Progressed) => continue,
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(settlement)) => break settlement,
            Ok(WorthQueryApplicationOutputDemandProgress::Pending) => {
                panic!("the synchronous native upstream must not leave external work pending")
            }
            Err(cause) => {
                panic!("the indexed decision must complete on a current basis: {cause:?}")
            }
        }
    };
    let current = request.retain_current_read().unwrap();
    request
        .at(&current)
        .require_current_output_demand(&settlement)
        .expect("the completed decision carries its current upstream and indexed membership");
    let calls = dependent_publication::take_calls();
    let contacts = settlement.producer_contacts_in_this_demand();
    // Changed indexed evidence enters C. If U is dirty, current_output stops
    // that entry before indexed selection; the native owner refreshes U and
    // resumes C with the same demand. Only the resumed entry completes a decision.
    let entries = usize::from(add_match) * (1 + usize::from(leave_dirty));
    assert_eq!(
        (calls, contacts),
        (entries, 1 + entries),
        "only changed indexed membership contacts this original consumer; dirty upstream: {leave_dirty}"
    );
    assert_eq!(
        dependent_publication::take_indexed_selections(),
        if add_match { vec![2] } else { vec![] },
        "exactly one indexed decision completes after U is current; reuse makes no decision"
    );
    assert_eq!(
        read_value(DEPENDENT),
        length(if add_match { 2 } else { 1 }),
        "the published output includes every matching entity"
    );
}

pub(super) fn declare_edit(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = EditPlanar::reference::<Schema>();
    builder
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 8192)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, PositionX::reference())
        .operation_read_field(operation, PositionY::reference())
        .operation_read_field(operation, Length::reference())
        .operation_read_relation(operation, PlanarSuccessor::reference())
        .operation_create(operation, Body::reference())
        .operation_write(operation, BodyKey::reference())
        .operation_write(operation, PositionX::reference())
        .operation_write(operation, PositionY::reference())
        .operation_write(operation, Length::reference())
        .operation_link(operation, PlanarSuccessor::reference())
        .operation_unlink(operation, PlanarSuccessor::reference())
        .application_mutation_binding::<PlanarEditBinding<Schema>>()
}
