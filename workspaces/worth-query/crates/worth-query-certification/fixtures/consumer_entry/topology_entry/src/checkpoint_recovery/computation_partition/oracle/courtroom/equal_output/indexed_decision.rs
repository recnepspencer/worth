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
    // An equal upstream left Dirty stales nothing by itself: without a new
    // match the consumer settles in its one advance below.
    if leave_dirty && add_match {
        let stopped = consumer.advance(&request);
        assert!(
            matches!(&stopped,
            Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::PublicationStale
                    && denial.recovery_posture() == primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable),
            "publishing U while the decision reads it requires a fresh publication basis: {:?}",
            stopped.as_ref().err()
        );
        assert_eq!(
            dependent_publication::take_calls(),
            1,
            "changed indexed membership contacts the decision while U is Dirty"
        );
        assert_eq!(
            read_value(DEPENDENT),
            length(1),
            "the stale decision does not publish"
        );
    }
    let contacts = settle!(consumer, request).0;
    assert_eq!(
        dependent_publication::take_calls(),
        usize::from(add_match),
        "only changed decision membership contacts the consumer"
    );
    assert_eq!(
        contacts,
        1 + usize::from(add_match) * (1 + usize::from(leave_dirty)),
        "the stale contact and the published one both follow the new match"
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
