use super::*;

pub(super) fn entity_member(entity: &str) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Entity {
        entity: entity.to_string(),
    }
}

pub(super) fn aspect_member(entity: &str, aspect: &str) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Aspect {
        entity: entity.to_string(),
        aspect: aspect.to_string(),
        identity: worth_foundational::facade::AspectIdentity(0x91613004),
        revision: worth_foundational::facade::AspectContractRevision(1),
    }
}

pub(super) fn elevation_field_member(field: &str) -> ApplicationSchemaMember {
    typed_field_member("Elevation", "ElevationFacts", field)
}

pub(super) fn review_field_member(field: &str) -> ApplicationSchemaMember {
    typed_field_member("Review", "ReviewFacts", field)
}

fn typed_field_member(entity: &str, aspect: &str, field: &str) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Field {
        entity: entity.to_string(),
        aspect: aspect.to_string(),
        field: field.to_string(),
        presence: ApplicationFieldPresence::Required,
        scalar_family: ScalarAspectType::UInt64,
        value_type:
            <u64 as crate::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY
                .as_str()
                .to_string(),
        unit: None,
        frame: None,
        writable: false,
        equality_queryable: true,
    }
}

pub(super) fn context_slot_member(
    slot: &str,
    slot_type: &'static str,
    entity: &str,
) -> ApplicationSchemaMember {
    ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
        context: "Context".to_string(),
        context_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared("Context"),
        slot: slot.to_string(),
        slot_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(slot_type),
        entity: entity.to_string(),
    }
}

pub(super) fn operation_member(operation: &str) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Operation {
        operation: operation.to_string(),
        input_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
            "worth.rust.unit",
        ),
    }
}
