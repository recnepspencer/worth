use super::*;
use crate::application_schema::ApplicationFieldPresence;

pub(super) fn members(contract: ErasedContract) -> Vec<ApplicationSchemaMember> {
    let mut members = vec![
        ApplicationSchemaMember::Entity {
            entity: "Grant".to_string(),
        },
        ApplicationSchemaMember::Entity {
            entity: "Resource".to_string(),
        },
        ApplicationSchemaMember::Entity {
            entity: "Principal".to_string(),
        },
        ApplicationSchemaMember::Aspect {
            entity: "Grant".to_string(),
            aspect: "Facts".to_string(),
            identity: worth_foundational::facade::AspectIdentity(0x91613002),
            revision: worth_foundational::facade::AspectContractRevision(1),
        },
        ApplicationSchemaMember::Aspect {
            entity: "Resource".to_string(),
            aspect: "ResourceFacts".to_string(),
            identity: worth_foundational::facade::AspectIdentity(0x91613003),
            revision: worth_foundational::facade::AspectContractRevision(1),
        },
        ApplicationSchemaMember::PrincipalBinding {
            binding: "PrincipalBinding".to_string(),
            mapping_entity: "Grant".to_string(),
            identity_aspect: "Facts".to_string(),
            identity_field: "Action".to_string(),
            status_aspect: "Facts".to_string(),
            status_field: "Purpose".to_string(),
            target_relation: "Grantor".to_string(),
            principal_entity: "Principal".to_string(),
            principal_identity_aspect: "Facts".to_string(),
            principal_identity_field: "Field".to_string(),
            principal_identity_scalar_family: ScalarAspectType::UInt64,
            principal_identity_value_type:
                <u64 as crate::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY
                    .as_str()
                    .to_string(),
        },
        ApplicationSchemaMember::Operation {
            operation: "Operation".to_string(),
            input_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                "worth.rust.unit",
            ),
        },
        ApplicationSchemaMember::ApplicationCapabilityContext {
            context: "Context".to_string(),
            context_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                "Context",
            ),
        },
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
            context: "Context".to_string(),
            context_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                "Context",
            ),
            slot: "ResourceSlot".to_string(),
            slot_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                "ResourceSlot",
            ),
            entity: "Resource".to_string(),
        },
        ApplicationSchemaMember::ApplicationCapabilityProvenance {
            provenance: "Provenance".to_string(),
            provenance_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                "Provenance",
            ),
        },
        relation_member("ResourceRelation", "Grant", "Resource"),
        relation_member("WrongResourceRelation", "Principal", "Resource"),
        relation_member("ScopedRelation", "Grant", "Resource"),
        relation_member("PrincipalResource", "Principal", "Resource"),
        relation_member("Parent", "Grant", "Grant"),
        relation_member("Grantor", "Principal", "Grant"),
        relation_member("Grantee", "Principal", "Grant"),
    ];
    for field in [
        "Action",
        "Purpose",
        "Field",
        "Amount",
        "Workflow",
        "Status",
        "ValidFrom",
        "ValidThrough",
        "DelegationLimit",
    ] {
        members.push(field_member(field));
    }
    members.push(resource_field_member("ResourceWorkflow"));
    members.push(ApplicationSchemaMember::ApplicationCapability { contract });
    members
}

pub(super) fn field_member(field: &str) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Field {
        entity: "Grant".to_string(),
        aspect: "Facts".to_string(),
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

pub(super) fn resource_field_member(field: &str) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Field {
        entity: "Resource".to_string(),
        aspect: "ResourceFacts".to_string(),
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

pub(super) fn relation_member(relation: &str, from: &str, to: &str) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Relation {
        relation: relation.to_string(),
        from: from.to_string(),
        to: to.to_string(),
        integrity: crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    }
}
