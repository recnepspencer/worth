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
    assert_eq!(settle!(root, request).0, 1);
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
    if !leave_dirty {
        // The upstream's prepared input changes from 1 to 2, but ceil(y / 2) stays 1.
        let mut refreshed = request
            .demand(Demand(OUTPUT))
            .start_in_program::<installation::Program, installation::Root>(&app)
            .unwrap();
        assert_eq!(settle!(refreshed, request).0, 1, "U really decides again");
        assert_eq!(
            read_value(OUTPUT),
            length(1),
            "U republishes an equal value"
        );
    }
    dependent_publication::take_calls();
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
    if add_match {
        assert!(calls > 0 && contacts > 1,
            "changed indexed membership reruns this original consumer: {calls} decisions, {contacts} lifetime contacts");
    } else {
        assert_eq!(
            (calls, contacts),
            (0, 1),
            "unchanged indexed membership reuses the equal upstream without another decision"
        );
    }
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
