use super::*;

#[test]
fn capability_marker_identity_survives_a_rust_module_move() {
    mod original_location {
        crate::worth_query_capability!(
            pub(super) Capability in super::Schema,
            identity "worth.query.test.module-move-capability.v1"
        );
    }
    mod moved_location {
        crate::worth_query_capability!(
            pub(super) Capability in super::Schema,
            identity "worth.query.test.module-move-capability.v1"
        );
    }

    assert_eq!(
        original_location::Capability::reference().marker_identity(),
        moved_location::Capability::reference().marker_identity()
    );
    assert_ne!(
        std::any::type_name::<original_location::Capability>(),
        std::any::type_name::<moved_location::Capability>()
    );
}

#[test]
fn capability_dimension_schema_identity_survives_rust_module_moves() {
    mod original_location {
        pub(super) struct Entity;
        impl crate::application_schema::fragment::Entity for Entity {
            const NAME: &'static str = "Entity";
            type Aspects = ();
            type Relations = ();
        }
        pub(super) struct Records;
        impl crate::application_schema::fragment::Fragment for Records {
            const NAME: &'static str = "relocated";
            type Bindings = crate::application_schema::fragment::NoBindings;
            type Members = (Entity,);
        }

        crate::worth_query_capability_context!(
            pub(super) Context in super::Schema,
            identity "worth.query.test.moved-context.v1"
        );
        crate::worth_query_capability_context_entity_slot!(
            pub(super) Slot in super::Schema,
            Context => Entity,
            identity "worth.query.test.moved-slot.v1"
        );
        crate::worth_query_capability_provenance!(
            pub(super) Provenance in super::Schema,
            identity "worth.query.test.moved-provenance.v1"
        );
    }
    mod moved_location {
        pub(super) struct Entity;
        impl crate::application_schema::fragment::Entity for Entity {
            const NAME: &'static str = "Entity";
            type Aspects = ();
            type Relations = ();
        }
        pub(super) struct Records;
        impl crate::application_schema::fragment::Fragment for Records {
            const NAME: &'static str = "relocated";
            type Bindings = crate::application_schema::fragment::NoBindings;
            type Members = (Entity,);
        }

        crate::worth_query_capability_context!(
            pub(super) Context in super::Schema,
            identity "worth.query.test.moved-context.v1"
        );
        crate::worth_query_capability_context_entity_slot!(
            pub(super) Slot in super::Schema,
            Context => Entity,
            identity "worth.query.test.moved-slot.v1"
        );
        crate::worth_query_capability_provenance!(
            pub(super) Provenance in super::Schema,
            identity "worth.query.test.moved-provenance.v1"
        );
    }

    let original =
        ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(original_location::Records)
            .unwrap()
            .capability_context(original_location::Context::reference())
            .capability_context_entity_slot(original_location::Slot::reference())
            .capability_provenance(original_location::Provenance::reference())
            .build()
            .unwrap();
    let moved =
        ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(moved_location::Records)
            .unwrap()
            .capability_context(moved_location::Context::reference())
            .capability_context_entity_slot(moved_location::Slot::reference())
            .capability_provenance(moved_location::Provenance::reference())
            .build()
            .unwrap();

    assert_eq!(original.identity(), moved.identity());
}
