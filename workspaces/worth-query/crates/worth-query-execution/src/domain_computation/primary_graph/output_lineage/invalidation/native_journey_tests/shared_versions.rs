//! The retained versions of a branch share their rows. A row is reserved
//! once, for as long as any retained version holds it, and a version adds
//! only what its own edits copied. The oldest retained version shares no
//! older one, so its reservation and the root's cover its whole index.

use super::*;

#[test]
fn versions_sharing_a_row_reserve_it_once_and_release_it_with_the_last() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: TypeId::of::<StatusOutput>(),
    };
    let row = RecordedSettlementIdentity::retain(&source, coordinate, 0);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let window = owner.resources.installation().maximum_retained_positions;
    let retained = || owner.resources.retained_capacity_bytes();
    let covers_oldest = |runtime: &mut RelationalRuntime| {
        let (basis_handle, basis) = snapshot(runtime);
        let cell = owner
            .cell_for_read(&basis, &mut owner.edit_admission())
            .unwrap()
            .unwrap();
        let image = cell.read_image();
        let root = image.payload();
        let oldest = root
            .past
            .get_min()
            .map_or(&root.current, |(_, past)| &past.state);
        let whole = super::super::retention::state_bound(oldest).unwrap();
        let own = oldest
            .retained_capacity
            .as_ref()
            .map_or(0, |own| own.bytes());
        let inherited = root
            .inherited_capacity
            .as_ref()
            .map_or(0, |inherited| inherited.bytes());
        runtime.snapshots().release_snapshot(&basis_handle).unwrap();
        own + inherited >= whole
    };

    handle.with_open_runtime_mut(|runtime| {
        // Each delivery touches the label, which the row never reads: its
        // version copies no index node.
        let mut flip = false;
        let mut deliver = |runtime: &mut RelationalRuntime| {
            flip = !flip;
            write_field(
                runtime,
                entity,
                label.clone(),
                if flip { "aa" } else { "bb" },
            );
        };
        for _ in 0..=window {
            deliver(runtime);
        }
        let vacant = retained();
        deliver(runtime);
        assert_eq!(
            retained(),
            vacant,
            "a full window of versions that hold no row is steady"
        );

        let (basis_handle, basis) = snapshot(runtime);
        register(
            owner,
            Arc::clone(&row),
            Arc::from([field_fact(runtime, &basis_handle, entity, status)]),
            &basis,
            OrdSet::new(),
        );
        runtime.snapshots().release_snapshot(&basis_handle).unwrap();
        assert!(retained() > vacant, "the registered row is reserved");
        // The registration's image at the delivered position leaves with the
        // next delivery; the version it registered stays for one window.
        deliver(runtime);
        let registered = retained();
        assert!(registered > vacant);

        // Every version after the registering one holds the same row and
        // copies nothing.
        for delivery in 2..window {
            deliver(runtime);
            assert_eq!(
                retained(),
                registered,
                "delivery {delivery} shares the row its predecessors reserved"
            );
            assert!(covers_oldest(runtime));
        }
        // The registering version becomes the oldest retained one, then
        // leaves: the root keeps what the versions after it still share.
        deliver(runtime);
        deliver(runtime);
        let shared = retained();
        assert!(
            vacant < shared && shared <= registered,
            "the window still holds the row once after the version that \
             registered it left: {vacant} < {shared} <= {registered}"
        );
        assert!(
            covers_oldest(runtime),
            "the oldest version's row was registered by a version that left"
        );
        deliver(runtime);
        assert_eq!(retained(), shared);

        assert_eq!(
            owner.retire_settlements(&[Arc::clone(&row)], &mut owner.edit_admission()),
            [Arc::clone(&row)]
        );
        assert!(
            retained() > vacant,
            "retained versions still hold the retired row"
        );
        assert!(covers_oldest(runtime));
        for _ in 0..=window {
            deliver(runtime);
        }
        assert_eq!(
            retained(),
            vacant,
            "the row's reservation is released with the last version that held it"
        );
    });
}
