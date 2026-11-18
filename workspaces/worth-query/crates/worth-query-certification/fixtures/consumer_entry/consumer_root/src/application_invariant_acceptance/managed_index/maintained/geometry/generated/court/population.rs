//! Independent native B universe and one actual declared producer publication.
use super::*;
pub(super) fn native_b(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
) {
    let vertices = [
        ("native-start", 50),
        ("native-end2", 60),
        ("native-end3", 70),
    ]
    .map(|(identity, x)| authoring::Edit::Vertex {
        directory: "catalog".into(),
        identity: identity.into(),
        x,
        y: 0,
    });
    let edges =
        [("native-e2", "native-end2"), ("native-e3", "native-end3")].map(|(identity, end)| {
            authoring::Edit::Edge {
                directory: "B".into(),
                identity: identity.into(),
                start: "native-start".into(),
                end: end.into(),
            }
        });
    for (index, edit) in vertices.into_iter().chain(edges).enumerate() {
        super::super::super::court::commit(world, principal, scope, edit, 0x919810 + index as u64);
    }
}
pub(super) fn produce(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
    scope: &WorthQueryRequestScope,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let performed = request
        .advanced()
        .mutate(action::RequestEdge {
            directory: "A".into(),
        })
        .without_source()
        .idempotency(&0x919820u64)
        .execute_performed::<host::Program, program::Root>(
            world,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
        panic!("geometry source action must freshly publish")
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
        "source and geometry co-commit"
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
        ("A", "generated-geometry")
    );
    let produced = request
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        values(produced.rows()),
        expected("A", 10),
        "real endpoints exist before delivery"
    );
    let coordinate = produced.rows()[0].coordinate;
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &publication
    );
    let mut output = performed
        .start_required_outputs(world, &request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|failure| panic!("geometry demand starts: {:?}", failure.denial()));
    let settlement = (0..64)
        .find_map(|_| {
            match output
                .required_output_mut()
                .advance(world, &request)
                .unwrap()
            {
                WorthQueryApplicationProgramOutputProgress::Settled(settlement) => Some(settlement),
                _ => None,
            }
        })
        .expect("one actual geometry producer settles inside its bound");
    assert_eq!(settlement.root_producer_contacts_in_this_demand(), 0);
    assert_eq!(settlement.work().producer_contact_count(), 0);
    assert_eq!(settlement.work().derived_publication_count(), 0);
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
    let edge = roles.entity::<operation::EdgeOutput>().unwrap();
    let id = edge.entity_id();
    assert_eq!(
        (
            id.partition_value(),
            id.local_slot_value(),
            id.generation_value()
        ),
        coordinate
    );
    assert!(roles.entity::<operation::StartOutput>().is_ok());
    assert!(roles.entity::<operation::EndOutput>().is_ok());
    assert_eq!(
        request
            .query(query::ReadSource {
                directory: "A".into()
            })
            .execute()
            .unwrap()
            .rows(),
        produced.rows()
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &publication
    );
}
