use worth_foundational::facade::{AspectContractRevision, AspectIdentity, ScalarAspectType};
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

mod capability;
mod operation;
mod query;

use capability::{application_capability, authorization_path};
use operation::{aftermath_contract, external_effect, inverse_aftermath_contract};
use query::application_query;

pub(super) const EXPECTED_MEMBER_COUNT: usize = 25;

pub(super) fn complete_untrusted_schema_record() -> WorthQueryPortableApplicationSchemaRecord {
    WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: "archive.tests".to_owned(),
            name: "CompleteUntrustedSchema".to_owned(),
            major: 4,
            minor: 2,
            members: members(),
            contributions: contributions(),
        },
    )
}

fn contributions() -> Vec<ApplicationSchemaContributionProvenance> {
    vec![
        ApplicationSchemaContributionProvenance::from_untrusted_parts(
            "worth.archive.tests.foundation.v1".to_owned(),
            (0..13).collect(),
        ),
        ApplicationSchemaContributionProvenance::from_untrusted_parts(
            "worth.archive.tests.operations.v1".to_owned(),
            (13..EXPECTED_MEMBER_COUNT as u32).collect(),
        ),
    ]
}

fn members() -> Vec<ApplicationSchemaMember> {
    vec![
        ApplicationSchemaMember::Entity {
            entity: text("Entity"),
        },
        ApplicationSchemaMember::Aspect {
            entity: text("Entity"),
            aspect: text("Aspect"),
            identity: AspectIdentity(17),
            revision: AspectContractRevision(3),
        },
        ApplicationSchemaMember::Field {
            entity: text("Entity"),
            aspect: text("Aspect"),
            field: text("field"),
            presence: ApplicationFieldPresence::Optional,
            scalar_family: ScalarAspectType::UInt64,
            value_type: text("worth.rust.u64"),
            unit: Some(text("count")),
            frame: Some(text("worth.tests.frame.v1")),
            writable: true,
            equality_queryable: true,
        },
        ApplicationSchemaMember::Relation {
            relation: text("relates"),
            from: text("Entity"),
            to: text("Other"),
            integrity: worth_query_installation::facade::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        },
        principal_binding(),
        ApplicationSchemaMember::ApplicationQuery {
            definition: application_query(),
        },
        ApplicationSchemaMember::ApplicationCapability {
            contract: application_capability(),
        },
        ApplicationSchemaMember::ApplicationCapabilityContext {
            context: text("Context"),
            context_type: type_id("context"),
        },
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
            context: text("Context"),
            context_type: type_id("context"),
            slot: text("ResourceSlot"),
            slot_type: type_id("resource-slot"),
            entity: text("Entity"),
        },
        ApplicationSchemaMember::ApplicationCapabilityProvenance {
            provenance: text("Provenance"),
            provenance_type: type_id("provenance"),
        },
        ApplicationSchemaMember::Operation {
            operation: text("Apply"),
            input_type: type_id("input"),
        },
        ApplicationSchemaMember::OperationProgram {
            operation: text("Apply"),
            target: ApplicationOperationProgramTarget::Write {
                entity: text("Entity"),
                aspect: text("Aspect"),
                field: text("field"),
            },
        },
        ApplicationSchemaMember::OperationDecisionRead {
            operation: text("Apply"),
            target: ApplicationOperationDecisionReadTarget::Relation {
                relation: text("relates"),
                from: text("Entity"),
                to: text("Other"),
            },
        },
        ApplicationSchemaMember::OperationMutationPrecondition {
            operation: text("Apply"),
            target: ApplicationMutationPreconditionTarget::from_untrusted_fields(
                ApplicationMutationPreconditionFamily::ExpectedFact,
                text("Entity"),
                text("Aspect"),
                text("field"),
            ),
        },
        ApplicationSchemaMember::OperationDecisionFactBudget {
            operation: text("Apply"),
            maximum_fact_count: 9,
        },
        ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation: text("Apply"),
            maximum_work_units: 11,
        },
        external_effect(),
        ApplicationSchemaMember::OperationAftermath {
            operation: text("Apply"),
            contract: aftermath_contract(),
        },
        ApplicationSchemaMember::Policy {
            policy: text("Policy"),
        },
        ApplicationSchemaMember::Ability {
            ability: text("Read"),
            scope_entity: text("Entity"),
        },
        ApplicationSchemaMember::OperationAbility {
            operation: text("Apply"),
            ability: text("Read"),
            scope_entity: text("Entity"),
        },
        ApplicationSchemaMember::AbilityPolicy {
            ability: text("Read"),
            scope_entity: text("Entity"),
            policy: text("Policy"),
            paths: vec![authorization_path()],
        },
        ApplicationSchemaMember::Unit {
            unit: text("count"),
        },
        ApplicationSchemaMember::Effect {
            effect: text("Changed"),
            payload_type: type_id("effect-payload"),
        },
        ApplicationSchemaMember::OperationAftermath {
            operation: text("Reverse"),
            contract: inverse_aftermath_contract(),
        },
    ]
}

fn principal_binding() -> ApplicationSchemaMember {
    ApplicationSchemaMember::PrincipalBinding {
        binding: text("PrincipalBinding"),
        mapping_entity: text("Mapping"),
        identity_aspect: text("Identity"),
        identity_field: text("external"),
        status_aspect: text("Status"),
        status_field: text("enabled"),
        target_relation: text("maps-to"),
        principal_entity: text("Principal"),
        principal_identity_aspect: text("Identity"),
        principal_identity_field: text("principal-id"),
        principal_identity_scalar_family: ScalarAspectType::String,
        principal_identity_value_type: text("worth.rust.string"),
    }
}

fn type_id(name: &str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::from_untrusted(format!("worth.tests.{name}.v1"))
}

fn text(value: &str) -> String {
    value.to_owned()
}
