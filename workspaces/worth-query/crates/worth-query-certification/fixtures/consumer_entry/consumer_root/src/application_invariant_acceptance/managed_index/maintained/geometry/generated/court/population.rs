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
    assert_eq!(settlement.root_producer_contacts_in_this_demand(), 1);
    assert!(settlement.root_receipt().is_some());
    let roles = settlement
        .root_settlement()
        .outputs_of::<operation::Outputs>()
        .unwrap();
    assert!(roles.entity::<operation::EdgeOutput>().is_ok());
    assert!(roles.entity::<operation::StartOutput>().is_ok());
    assert!(roles.entity::<operation::EndOutput>().is_ok());
}
