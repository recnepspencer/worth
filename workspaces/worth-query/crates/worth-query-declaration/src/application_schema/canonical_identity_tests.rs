use worth_foundational::facade::{
    BoundaryProtocolIdentity, BoundaryProtocolVersion, ScalarAspectType,
};

use super::{
    canonical_identity, ApplicationSchemaCanonicalHeader, ApplicationSchemaIdentity,
    ApplicationSchemaMember,
};
use crate::application_schema::{
    ApplicationExternalEffectProtocol, ApplicationOperationProgramTarget,
    WorthQueryExternalEffectCorrelationFamily,
};
use crate::application_schema::{
    ApplicationMutationPreconditionFamily, ApplicationMutationPreconditionTarget,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
#[path = "canonical_identity_query_fixture.rs"]
mod query_fixture;
use query_fixture::{application_query, QueryEntity, QueryMarker, QueryResult, QuerySchema};

#[path = "canonical_identity_application_query_tests.rs"]
mod application_query;

#[test]
fn every_application_schema_member_family_changes_identity() {
    let empty = identity(&[]);
    for member in [
        ApplicationSchemaMember::Entity {
            entity: "Entity".to_string(),
        },
        ApplicationSchemaMember::Aspect {
            entity: "Entity".to_string(),
            aspect: "Aspect".to_string(),
            identity: worth_foundational::facade::AspectIdentity(0x91613001),
            revision: worth_foundational::facade::AspectContractRevision(1),
        },
        field((ScalarAspectType::UInt64, "u64", None, false, false)),
        ApplicationSchemaMember::Relation {
            relation: "Relation".to_string(),
            from: "From".to_string(),
            to: "To".to_string(),
            integrity: crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        },
        principal_binding("PrincipalBinding"),
        application_query("value"),
        ApplicationSchemaMember::ApplicationCapabilityContext {
            context: "Context".to_string(),
            context_type: portable("ContextType"),
        },
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
            context: "Context".to_string(),
            context_type: portable("ContextType"),
            slot: "Slot".to_string(),
            slot_type: portable("SlotType"),
            entity: "Entity".to_string(),
        },
        ApplicationSchemaMember::ApplicationCapabilityProvenance {
            provenance: "Provenance".to_string(),
            provenance_type: portable("ProvenanceType"),
        },
        ApplicationSchemaMember::Operation {
            operation: "Operation".to_string(),
            input_type: portable("Input"),
        },
        ApplicationSchemaMember::OperationProgram {
            operation: "Operation".to_string(),
            target: ApplicationOperationProgramTarget::Create {
                entity: "Entity".to_string(),
            },
        },
        mutation_precondition(ApplicationMutationPreconditionFamily::ExpectedFact, "Field"),
        ApplicationSchemaMember::Policy {
            policy: "Policy".to_string(),
        },
        ApplicationSchemaMember::Unit {
            unit: "USD".to_string(),
        },
        ApplicationSchemaMember::Effect {
            effect: "Effect".to_string(),
            payload_type: portable("Payload"),
        },
    ] {
        assert_ne!(identity(&[member]), empty);
    }
}

#[test]
fn capability_context_slot_and_provenance_axes_change_schema_identity() {
    let context =
        |name: &str, marker: &'static str| ApplicationSchemaMember::ApplicationCapabilityContext {
            context: name.to_string(),
            context_type: portable(marker),
        };
    assert_ne!(
        identity(&[context("Context", "ContextType")]),
        identity(&[context("OtherContext", "ContextType")])
    );
    assert_ne!(
        identity(&[context("Context", "ContextType")]),
        identity(&[context("Context", "OtherContextType")])
    );

    let slot = |name: &str, marker: &'static str, entity: &str| {
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
            context: "Context".to_string(),
            context_type: portable("ContextType"),
            slot: name.to_string(),
            slot_type: portable(marker),
            entity: entity.to_string(),
        }
    };
    let baseline = identity(&[slot("Slot", "SlotType", "Entity")]);
    for changed in [
        slot("OtherSlot", "SlotType", "Entity"),
        slot("Slot", "OtherSlotType", "Entity"),
        slot("Slot", "SlotType", "OtherEntity"),
    ] {
        assert_ne!(baseline, identity(&[changed]));
    }

    let provenance = |name: &str, marker: &'static str| {
        ApplicationSchemaMember::ApplicationCapabilityProvenance {
            provenance: name.to_string(),
            provenance_type: portable(marker),
        }
    };
    assert_ne!(
        identity(&[provenance("Provenance", "ProvenanceType")]),
        identity(&[provenance("OtherProvenance", "ProvenanceType")])
    );
    assert_ne!(
        identity(&[provenance("Provenance", "ProvenanceType")]),
        identity(&[provenance("Provenance", "OtherProvenanceType")])
    );
}

#[test]
fn mutation_precondition_family_and_target_change_schema_identity() {
    let expected_fact =
        mutation_precondition(ApplicationMutationPreconditionFamily::ExpectedFact, "Field");
    assert_ne!(
        identity(std::slice::from_ref(&expected_fact)),
        identity(&[mutation_precondition(
            ApplicationMutationPreconditionFamily::ExpectedVersion,
            "Field",
        )])
    );
    assert_ne!(
        identity(&[expected_fact]),
        identity(&[mutation_precondition(
            ApplicationMutationPreconditionFamily::ExpectedFact,
            "OtherField",
        )])
    );
}

#[test]
fn application_query_meaning_changes_schema_identity() {
    assert_ne!(
        identity(&[application_query("value")]),
        identity(&[application_query("renamed_value")])
    );
}

#[test]
fn every_field_capability_and_type_dimension_changes_identity() {
    let base = identity(&[field((ScalarAspectType::UInt64, "u64", None, false, false))]);
    for changed in [
        field((ScalarAspectType::Int64, "u64", None, false, false)),
        field((ScalarAspectType::UInt64, "Other", None, false, false)),
        field((ScalarAspectType::UInt64, "u64", Some("USD"), false, false)),
        field((ScalarAspectType::UInt64, "u64", None, true, false)),
        field((ScalarAspectType::UInt64, "u64", None, false, true)),
    ] {
        assert_ne!(identity(&[changed]), base);
    }
}

#[test]
fn every_principal_binding_dimension_changes_identity() {
    let base = principal_binding("PrincipalBinding");
    let base_identity = identity(std::slice::from_ref(&base));
    macro_rules! changed {
        ($field:ident, $value:literal) => {{
            let mut member = base.clone();
            let ApplicationSchemaMember::PrincipalBinding { $field, .. } = &mut member else {
                unreachable!("principal binding fixture changed member family")
            };
            *$field = $value.to_string();
            member
        }};
    }
    for member in [
        changed!(binding, "OtherBinding"),
        changed!(mapping_entity, "OtherMapping"),
        changed!(identity_aspect, "OtherIdentityAspect"),
        changed!(identity_field, "OtherIdentityField"),
        changed!(status_aspect, "OtherStatusAspect"),
        changed!(status_field, "OtherStatusField"),
        changed!(target_relation, "OtherTargetRelation"),
        changed!(principal_entity, "OtherPrincipal"),
        changed!(principal_identity_aspect, "OtherPrincipalIdentityAspect"),
        changed!(principal_identity_field, "OtherPrincipalIdentityField"),
        changed!(principal_identity_value_type, "OtherPrincipalIdentityValue"),
    ] {
        assert_ne!(identity(&[member]), base_identity);
    }
    let mut changed_family = base;
    let ApplicationSchemaMember::PrincipalBinding {
        principal_identity_scalar_family,
        ..
    } = &mut changed_family
    else {
        unreachable!("principal binding fixture changed member family")
    };
    *principal_identity_scalar_family = ScalarAspectType::Bool;
    assert_ne!(identity(&[changed_family]), base_identity);
}

#[test]
fn operation_input_effect_payload_and_schema_version_change_identity() {
    let operation = ApplicationSchemaMember::Operation {
        operation: "Operation".to_string(),
        input_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared("Input"),
    };
    let changed_operation = ApplicationSchemaMember::Operation {
        operation: "Operation".to_string(),
        input_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
            "OtherInput",
        ),
    };
    assert_ne!(identity(&[operation]), identity(&[changed_operation]));

    let effect = ApplicationSchemaMember::Effect {
        effect: "Effect".to_string(),
        payload_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared("Payload"),
    };
    let changed_effect = ApplicationSchemaMember::Effect {
        effect: "Effect".to_string(),
        payload_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
            "OtherPayload",
        ),
    };
    assert_ne!(identity(&[effect]), identity(&[changed_effect]));

    let initial = canonical_identity(header(0), &[], &[]);
    let successor = canonical_identity(header(1), &[], &[]);
    assert_ne!(initial, successor);
}

#[test]
fn every_external_effect_contract_dimension_changes_identity() {
    let base = ApplicationSchemaMember::OperationExternalEffect {
        operation: "Operation".to_string(),
        effect: "ExternalEffect".to_string(),
        rust_payload_type: crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
            "Payload",
        ),
        protocol: external_protocol(1),
        maximum_payload_bytes: 64,
        correlation_family: WorthQueryExternalEffectCorrelationFamily::new("external-family")
            .unwrap(),
    };
    let base_identity = identity(std::slice::from_ref(&base));

    macro_rules! changed {
        ($field:ident, $value:expr) => {{
            let mut member = base.clone();
            let ApplicationSchemaMember::OperationExternalEffect { $field, .. } = &mut member
            else {
                unreachable!("external-effect fixture changed member family")
            };
            *$field = $value;
            member
        }};
    }

    for member in [
        changed!(operation, "OtherOperation".to_string()),
        changed!(effect, "OtherEffect".to_string()),
        changed!(
            rust_payload_type,
            crate::portable_identity::WorthQueryPortableTypeIdentity::declared("OtherPayload")
        ),
        changed!(protocol, external_protocol(2)),
        changed!(maximum_payload_bytes, 65),
        changed!(
            correlation_family,
            WorthQueryExternalEffectCorrelationFamily::new("other-family").unwrap()
        ),
    ] {
        assert_ne!(identity(&[member]), base_identity);
    }
}

fn external_protocol(version: u32) -> ApplicationExternalEffectProtocol {
    ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("test.external-payload"),
        BoundaryProtocolVersion::new(version),
    )
}

#[test]
fn every_operation_program_action_changes_identity() {
    let base = identity(&[]);
    for target in [
        ApplicationOperationProgramTarget::Create {
            entity: "Entity".to_string(),
        },
        ApplicationOperationProgramTarget::Delete {
            entity: "Entity".to_string(),
        },
        ApplicationOperationProgramTarget::Write {
            entity: "Entity".to_string(),
            aspect: "Aspect".to_string(),
            field: "Field".to_string(),
        },
        ApplicationOperationProgramTarget::Link {
            relation: "Relation".to_string(),
            from: "From".to_string(),
            to: "To".to_string(),
        },
        ApplicationOperationProgramTarget::Unlink {
            relation: "Relation".to_string(),
            from: "From".to_string(),
            to: "To".to_string(),
        },
        ApplicationOperationProgramTarget::Emit {
            effect: "Effect".to_string(),
        },
    ] {
        let program = ApplicationSchemaMember::OperationProgram {
            operation: "Operation".to_string(),
            target,
        };
        assert_ne!(identity(&[program]), base);
    }
}

fn identity(members: &[ApplicationSchemaMember]) -> ApplicationSchemaIdentity {
    canonical_identity(header(0), members, &[])
}

fn mutation_precondition(
    family: ApplicationMutationPreconditionFamily,
    field: &str,
) -> ApplicationSchemaMember {
    ApplicationSchemaMember::OperationMutationPrecondition {
        operation: "Operation".to_string(),
        target: ApplicationMutationPreconditionTarget::field(family, "Entity", "Aspect", field),
    }
}

const fn portable(identity: &'static str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::declared(identity)
}

fn header(minor: u32) -> ApplicationSchemaCanonicalHeader<'static> {
    ApplicationSchemaCanonicalHeader {
        owner: "owner",
        name: "Schema",
        major: 1,
        minor,
    }
}

fn field(
    dimensions: (ScalarAspectType, &str, Option<&str>, bool, bool),
) -> ApplicationSchemaMember {
    let (scalar_family, value_type, unit, writable, equality_queryable) = dimensions;
    ApplicationSchemaMember::Field {
        entity: "Entity".to_string(),
        aspect: "Aspect".to_string(),
        field: "Field".to_string(),
        presence: crate::application_schema::ApplicationFieldPresence::Required,
        scalar_family,
        value_type: value_type.to_string(),
        unit: unit.map(str::to_string),
        frame: None,
        writable,
        equality_queryable,
    }
}

fn principal_binding(binding: &str) -> ApplicationSchemaMember {
    ApplicationSchemaMember::PrincipalBinding {
        binding: binding.to_string(),
        mapping_entity: "ExternalPrincipalMapping".to_string(),
        identity_aspect: "ExternalIdentity".to_string(),
        identity_field: "ExternalIdentityField".to_string(),
        status_aspect: "ExternalIdentity".to_string(),
        status_field: "ExternalMappingStatus".to_string(),
        target_relation: "ExternalPrincipalTarget".to_string(),
        principal_entity: "Principal".to_string(),
        principal_identity_aspect: "PrincipalIdentity".to_string(),
        principal_identity_field: "PrincipalIdentityField".to_string(),
        principal_identity_scalar_family: ScalarAspectType::UInt64,
        principal_identity_value_type: "u64".to_string(),
    }
}
