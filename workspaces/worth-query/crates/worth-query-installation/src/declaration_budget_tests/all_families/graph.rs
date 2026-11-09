//! Graph, scalar, format and context-slot declarations in the complete schema.
use super::*;
use worth_foundational::facade::{
    AbsenceLaw, AspectContractRevision, AspectIdentity, ScalarAspectType,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryRootPathMeaning, WorthQueryPortableApplicationQueryRootPathParts,
};

pub(super) fn extend(members: &mut Vec<ApplicationSchemaMember>) {
    let derived = members
        .iter()
        .find_map(|m| match m {
            ApplicationSchemaMember::DerivedRelation { definition } => Some(definition.parts()),
            _ => None,
        })
        .unwrap()
        .clone();
    let context = members
        .iter()
        .find_map(|m| match m {
            ApplicationSchemaMember::ApplicationCapabilityContext {
                context,
                context_type,
            } => Some((context.clone(), context_type.clone())),
            _ => None,
        })
        .unwrap();
    let mutation = members
        .iter()
        .find_map(|m| match m {
            ApplicationSchemaMember::ApplicationMutation { description } => Some(description),
            _ => None,
        })
        .expect("an actual mutation supplies the component-read operation");
    assert_eq!(mutation.scope().entity, "TestEntity");
    let operation = mutation.operation().to_owned();
    members.extend([
        ApplicationSchemaMember::ContentFormat {
            declaration: ApplicationContentFormatDeclaration {
                identity: [0x76; 32],
                name: "budget.u64-frame".into(),
                version: 1,
                maximum_frame_bytes: 8,
            },
        },
        ApplicationSchemaMember::Unit {
            unit: "Millimetres".into(),
        },
        ApplicationSchemaMember::Entity {
            entity: "BudgetScalar".into(),
        },
        ApplicationSchemaMember::Aspect {
            absence: AbsenceLaw::Required,
            entity: "BudgetScalar".into(),
            aspect: "Value".into(),
            identity: AspectIdentity(0x7699),
            revision: AspectContractRevision(1),
        },
        ApplicationSchemaMember::Field {
            entity: "BudgetScalar".into(),
            aspect: "Value".into(),
            field: "value".into(),
            presence: ApplicationFieldPresence::Required,
            scalar_family: ScalarAspectType::UInt64,
            value_type: "u64".into(),
            canonical: None,
            content_format: None,
            unit: None,
            frame: None,
            writable: true,
            equality_queryable: false,
            unique: false,
        },
        ApplicationSchemaMember::ScalarAspectField {
            entity: "BudgetScalar".into(),
            aspect: "Value".into(),
            field: "value".into(),
        },
        ApplicationSchemaMember::RelationAspect {
            definition: ErasedApplicationRelationAspectDefinition::from_untrusted_parts(
                WorthQueryPortableRelationAspectParts {
                    relation: "MappingTarget".into(),
                    from: "TestEntity".into(),
                    to: "TestEntity".into(),
                    aspect: "Weight".into(),
                    identity: AspectIdentity(0x769a),
                    revision: AspectContractRevision(1),
                    absence: AbsenceLaw::Required,
                    fields: vec![ApplicationRelationFieldDefinition {
                        field: "weight".into(),
                        presence: ApplicationFieldPresence::Required,
                        scalar_family: ScalarAspectType::UInt64,
                        value_type: "u64".into(),
                        canonical: None,
                        unit: None,
                        frame: None,
                        writable: false,
                    }],
                },
            ),
        },
        ApplicationSchemaMember::ComponentSet {
            definition: ApplicationComponentSetMeaning::from_untrusted_parts(
                WorthQueryPortableApplicationComponentSetParts {
                    name: "BudgetComponents".into(),
                    member: "TestEntity".into(),
                    relations: vec!["MappingTarget".into()],
                },
            ),
        },
        ApplicationSchemaMember::DerivedComponentSet {
            definition: ApplicationDerivedComponentSetMeaning {
                name: "BudgetDerivedComponents".into(),
                member: derived.from.clone(),
                relation: derived.relation.clone(),
            },
        },
        ApplicationSchemaMember::OperationComponentRead {
            definition: ApplicationOperationComponentReadMeaning::from_untrusted_parts(
                WorthQueryPortableOperationComponentReadParts {
                    operation,
                    set: "BudgetComponents".into(),
                    root: "TestEntity".into(),
                    member: "TestEntity".into(),
                    paths: vec![ApplicationQueryRootPathMeaning::from_untrusted_parts(
                        WorthQueryPortableApplicationQueryRootPathParts {
                            start_entity: "TestEntity".into(),
                            terminal_entity: "TestEntity".into(),
                            steps: Vec::new(),
                            guards: Vec::new(),
                        },
                    )],
                },
            ),
        },
        ApplicationSchemaMember::ApplicationCapabilityContextEntitySlot {
            context: context.0,
            context_type: context.1,
            slot: "BudgetTarget".into(),
            slot_type: portable("worth.budget.target-slot"),
            entity: "TestEntity".into(),
        },
    ]);
}
